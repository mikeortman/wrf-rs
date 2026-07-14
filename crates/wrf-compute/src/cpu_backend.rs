use std::num::NonZeroUsize;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::thread;

use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::{
    BackendKind, ComputeBackend, ComputeError, ComputeResult, CpuField, FieldValue, GridShape,
    LinearChunk, ParallelExecutionError,
};

const TARGET_CHUNKS_PER_WORKER: usize = 4;

/// Default multithreaded CPU field allocator and persistent work scheduler.
///
/// Cloning a backend only clones the `Arc` that owns its persistent thread pool;
/// it never duplicates field data or creates more worker threads.
#[derive(Clone, Debug)]
pub struct CpuBackend {
    thread_pool: Arc<ThreadPool>,
    worker_count: NonZeroUsize,
}

impl CpuBackend {
    /// Creates the standard backend using all parallelism reported by the host.
    pub fn try_new() -> ComputeResult<Self> {
        let worker_count = thread::available_parallelism().unwrap_or(NonZeroUsize::MIN);
        Self::try_with_non_zero_worker_count(worker_count)
    }

    /// Creates a backend with an explicit non-zero worker count.
    ///
    /// This is intended for resource limits and deterministic parity tests; the
    /// standard [`Self::try_new`] path remains automatically multithreaded.
    pub fn try_with_worker_count(worker_count: usize) -> ComputeResult<Self> {
        let worker_count = NonZeroUsize::new(worker_count).ok_or(ComputeError::ZeroWorkerCount)?;
        Self::try_with_non_zero_worker_count(worker_count)
    }

    /// Returns the number of persistent CPU workers.
    pub const fn worker_count(&self) -> usize {
        self.worker_count.get()
    }

    /// Executes a fallible CPU-specific parallel operation on this backend's pool.
    ///
    /// Numerical crates use this scheduling facility when one kernel must borrow
    /// several disjoint output fields at once. It is intentionally absent from
    /// [`ComputeBackend`]; future device backends implement the numerical
    /// capability itself rather than accepting host closures.
    pub fn try_execute_parallel<Output, KernelError, Operation>(
        &self,
        operation: Operation,
    ) -> Result<Output, ParallelExecutionError<KernelError>>
    where
        Output: Send,
        KernelError: Send,
        Operation: FnOnce() -> Result<Output, KernelError> + Send,
    {
        let parallel_result =
            catch_unwind(AssertUnwindSafe(|| self.thread_pool.install(operation)));

        match parallel_result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(error)) => Err(ParallelExecutionError::Kernel(error)),
            Err(_) => Err(ParallelExecutionError::WorkerPanicked),
        }
    }

    /// Runs a closure over work-stealing, disjoint mutable output chunks.
    ///
    /// Shared input fields can be captured immutably by `operation`. No worker
    /// threads or per-point buffers are allocated by this call.
    pub fn try_for_each_output_chunk<Value, KernelError, Operation>(
        &self,
        output_values: &mut [Value],
        operation: Operation,
    ) -> Result<(), ParallelExecutionError<KernelError>>
    where
        Value: Send,
        KernelError: Send,
        Operation: Fn(LinearChunk, &mut [Value]) -> Result<(), KernelError> + Send + Sync,
    {
        if output_values.is_empty() {
            return Ok(());
        }

        let target_chunk_count = self.worker_count() * TARGET_CHUNKS_PER_WORKER;
        let chunk_size = output_values.len().div_ceil(target_chunk_count).max(1);
        let parallel_result = catch_unwind(AssertUnwindSafe(|| {
            self.thread_pool.install(|| {
                output_values
                    .par_chunks_mut(chunk_size)
                    .enumerate()
                    .try_for_each(|(chunk_index, output_chunk)| {
                        let start = chunk_index * chunk_size;
                        let linear_chunk = LinearChunk::new(start, start + output_chunk.len());
                        operation(linear_chunk, output_chunk)
                    })
            })
        }));

        match parallel_result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(ParallelExecutionError::Kernel(error)),
            Err(_) => Err(ParallelExecutionError::WorkerPanicked),
        }
    }

    /// Runs a closure over disjoint mutable output blocks of an exact length.
    ///
    /// This is the scheduling primitive for kernels whose independent work
    /// units are complete contiguous columns, rows, or vertical profiles. The
    /// operation runs directly on field storage without per-block allocation.
    pub fn try_for_each_output_block<Value, KernelError, Operation>(
        &self,
        output_values: &mut [Value],
        block_length: usize,
        operation: Operation,
    ) -> Result<(), ParallelExecutionError<KernelError>>
    where
        Value: Send,
        KernelError: Send,
        Operation: Fn(usize, &mut [Value]) -> Result<(), KernelError> + Send + Sync,
    {
        if block_length == 0 {
            return Err(ParallelExecutionError::ZeroBlockLength);
        }
        if output_values.len() % block_length != 0 {
            return Err(ParallelExecutionError::IncompleteOutputBlock {
                output_value_count: output_values.len(),
                block_length,
            });
        }

        let parallel_result = catch_unwind(AssertUnwindSafe(|| {
            self.thread_pool.install(|| {
                output_values
                    .par_chunks_mut(block_length)
                    .enumerate()
                    .try_for_each(|(block_index, output_block)| {
                        operation(block_index, output_block)
                    })
            })
        }));

        match parallel_result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(ParallelExecutionError::Kernel(error)),
            Err(_) => Err(ParallelExecutionError::WorkerPanicked),
        }
    }

    /// Runs a closure over matching disjoint blocks from two mutable outputs.
    ///
    /// This avoids numerical scratch and extra memory passes when one grid-point
    /// operation must update two independently owned fields. Both outputs must
    /// have the same length and form complete blocks.
    pub fn try_for_each_output_pair_block<Value, KernelError, Operation>(
        &self,
        first_output_values: &mut [Value],
        second_output_values: &mut [Value],
        block_length: usize,
        operation: Operation,
    ) -> Result<(), ParallelExecutionError<KernelError>>
    where
        Value: Send,
        KernelError: Send,
        Operation: Fn(usize, &mut [Value], &mut [Value]) -> Result<(), KernelError> + Send + Sync,
    {
        if block_length == 0 {
            return Err(ParallelExecutionError::ZeroBlockLength);
        }
        if first_output_values.len() != second_output_values.len() {
            return Err(ParallelExecutionError::PairedOutputLengthMismatch {
                first_output_value_count: first_output_values.len(),
                second_output_value_count: second_output_values.len(),
            });
        }
        if first_output_values.len() % block_length != 0 {
            return Err(ParallelExecutionError::IncompleteOutputBlock {
                output_value_count: first_output_values.len(),
                block_length,
            });
        }

        let parallel_result = catch_unwind(AssertUnwindSafe(|| {
            self.thread_pool.install(|| {
                first_output_values
                    .par_chunks_mut(block_length)
                    .zip(second_output_values.par_chunks_mut(block_length))
                    .enumerate()
                    .try_for_each(|(block_index, (first_block, second_block))| {
                        operation(block_index, first_block, second_block)
                    })
            })
        }));

        match parallel_result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(ParallelExecutionError::Kernel(error)),
            Err(_) => Err(ParallelExecutionError::WorkerPanicked),
        }
    }

    fn try_with_non_zero_worker_count(worker_count: NonZeroUsize) -> ComputeResult<Self> {
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(worker_count.get())
            .thread_name(|worker_index| format!("wrf-cpu-{worker_index}"))
            .build()
            .map_err(|error| ComputeError::ThreadPoolInitializationFailed {
                message: error.to_string(),
            })?;

        Ok(Self {
            thread_pool: Arc::new(thread_pool),
            worker_count,
        })
    }
}

impl ComputeBackend for CpuBackend {
    type Field<Value>
        = CpuField<Value>
    where
        Value: FieldValue;

    fn backend_kind(&self) -> BackendKind {
        BackendKind::Cpu
    }

    fn create_field<Value>(
        &self,
        shape: GridShape,
        initial_value: Value,
    ) -> ComputeResult<Self::Field<Value>>
    where
        Value: FieldValue,
    {
        Ok(CpuField::from_value(shape, initial_value))
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Condvar, Mutex};
    use std::time::Duration;

    use super::*;

    #[test]
    fn try_new_uses_host_parallelism_without_feature_flag() {
        let backend = CpuBackend::try_new().unwrap();

        assert_eq!(
            backend.worker_count(),
            thread::available_parallelism()
                .unwrap_or(NonZeroUsize::MIN)
                .get()
        );
    }

    #[test]
    fn clone_reuses_the_existing_thread_pool() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let cloned_backend = backend.clone();

        assert!(Arc::ptr_eq(
            &backend.thread_pool,
            &cloned_backend.thread_pool
        ));
    }

    #[test]
    fn try_execute_parallel_uses_the_backend_pool_and_preserves_errors() {
        let backend = CpuBackend::try_with_worker_count(3).unwrap();

        let worker_count = backend
            .try_execute_parallel(|| Ok::<_, &'static str>(rayon::current_num_threads()))
            .unwrap();
        let error = backend.try_execute_parallel(|| Err::<(), _>("kernel failed"));

        assert_eq!(worker_count, 3);
        assert_eq!(error, Err(ParallelExecutionError::Kernel("kernel failed")));
    }

    #[test]
    fn try_for_each_output_chunk_uses_disjoint_parallel_workers() {
        let backend = CpuBackend::try_with_worker_count(4).unwrap();
        let active_worker_count = Mutex::new(0_usize);
        let worker_started = Condvar::new();
        let observed_concurrent_workers = AtomicBool::new(false);
        let mut values = vec![0_usize; 65_536];

        backend
            .try_for_each_output_chunk(
                &mut values,
                |linear_chunk, output_chunk| -> Result<(), Infallible> {
                    let mut active_worker_count = active_worker_count.lock().unwrap();
                    *active_worker_count += 1;
                    if *active_worker_count > 1 {
                        observed_concurrent_workers.store(true, Ordering::SeqCst);
                        worker_started.notify_all();
                    } else if !observed_concurrent_workers.load(Ordering::SeqCst) {
                        let (worker_count, _) = worker_started
                            .wait_timeout_while(active_worker_count, Duration::from_secs(1), |_| {
                                !observed_concurrent_workers.load(Ordering::SeqCst)
                            })
                            .unwrap();
                        active_worker_count = worker_count;
                    }
                    *active_worker_count -= 1;
                    drop(active_worker_count);
                    for (local_index, value) in output_chunk.iter_mut().enumerate() {
                        *value = linear_chunk.range().start + local_index;
                    }
                    Ok(())
                },
            )
            .unwrap();

        assert_eq!(values, (0..65_536).collect::<Vec<_>>());
        assert!(observed_concurrent_workers.load(Ordering::SeqCst));
    }

    #[test]
    fn try_for_each_output_chunk_preserves_kernel_error() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut values = vec![0_u8; 16];

        let result = backend.try_for_each_output_chunk(&mut values, |linear_chunk, _| {
            if linear_chunk.range().start > 0 {
                return Err("later chunk failed");
            }
            Ok(())
        });

        assert_eq!(
            result,
            Err(ParallelExecutionError::Kernel("later chunk failed"))
        );
    }

    #[test]
    fn try_for_each_output_block_preserves_complete_line_boundaries() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut values = vec![0_usize; 12];

        backend
            .try_for_each_output_block(
                &mut values,
                3,
                |block_index, output_block| -> Result<(), Infallible> {
                    output_block.fill(block_index);
                    Ok(())
                },
            )
            .unwrap();

        assert_eq!(values, [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]);
    }

    #[test]
    fn try_for_each_output_block_rejects_invalid_block_shapes() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut values = vec![0_u8; 5];

        assert_eq!(
            backend.try_for_each_output_block(&mut values, 0, |_, _| -> Result<(), Infallible> {
                Ok(())
            }),
            Err(ParallelExecutionError::ZeroBlockLength)
        );
        assert_eq!(
            backend.try_for_each_output_block(&mut values, 2, |_, _| -> Result<(), Infallible> {
                Ok(())
            }),
            Err(ParallelExecutionError::IncompleteOutputBlock {
                output_value_count: 5,
                block_length: 2,
            })
        );
    }

    #[test]
    fn try_for_each_output_block_reports_worker_panics() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut values = vec![0_u8; 8];

        let result = backend.try_for_each_output_block(
            &mut values,
            2,
            |block_index, _| -> Result<(), Infallible> {
                assert_ne!(block_index, 2, "intentional worker panic");
                Ok(())
            },
        );

        assert_eq!(result, Err(ParallelExecutionError::WorkerPanicked));
    }

    #[test]
    fn try_for_each_output_pair_block_updates_matching_blocks() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut first = vec![0_usize; 12];
        let mut second = vec![0_usize; 12];

        backend
            .try_for_each_output_pair_block(
                &mut first,
                &mut second,
                3,
                |block_index, first_block, second_block| -> Result<(), Infallible> {
                    first_block.fill(block_index);
                    second_block.fill(block_index + 10);
                    Ok(())
                },
            )
            .unwrap();

        assert_eq!(first, [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]);
        assert_eq!(second, [10, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13]);
    }

    #[test]
    fn try_for_each_output_pair_block_rejects_mismatched_lengths() {
        let backend = CpuBackend::try_with_worker_count(2).unwrap();
        let mut first = vec![0_u8; 4];
        let mut second = vec![0_u8; 5];

        assert_eq!(
            backend.try_for_each_output_pair_block(
                &mut first,
                &mut second,
                2,
                |_, _, _| -> Result<(), Infallible> { Ok(()) },
            ),
            Err(ParallelExecutionError::PairedOutputLengthMismatch {
                first_output_value_count: 4,
                second_output_value_count: 5,
            })
        );
    }
}
