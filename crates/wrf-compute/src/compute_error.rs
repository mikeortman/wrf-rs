use std::fmt;

/// A failure at the compute storage or execution boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComputeError {
    /// At least one grid dimension was zero.
    EmptyGridDimension,
    /// Multiplying the grid dimensions exceeded addressable memory.
    GridPointCountOverflow,
    /// An explicit worker count was zero.
    ZeroWorkerCount,
    /// The persistent CPU thread pool could not be created.
    ThreadPoolInitializationFailed {
        /// The scheduler's diagnostic from thread-pool construction.
        message: String,
    },
}

/// The typed result returned by compute setup and infallible-operation boundaries.
pub type ComputeResult<T> = Result<T, ComputeError>;

/// A typed parallel-execution failure that preserves a kernel's own error.
#[derive(Debug, Eq, PartialEq)]
pub enum ParallelExecutionError<KernelError> {
    /// An exact output block was requested with zero values.
    ZeroBlockLength,
    /// The output cannot be divided into complete blocks of the requested size.
    IncompleteOutputBlock {
        /// Number of values in the supplied output slice.
        output_value_count: usize,
        /// Required number of values in each indivisible block.
        block_length: usize,
    },
    /// Two paired output slices have different lengths.
    PairedOutputLengthMismatch {
        /// Number of values in the first output slice.
        first_output_value_count: usize,
        /// Number of values in the second output slice.
        second_output_value_count: usize,
    },
    /// A numerical kernel returned a recoverable typed error.
    Kernel(KernelError),
    /// A worker panicked rather than returning a recoverable error.
    WorkerPanicked,
}

impl fmt::Display for ComputeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGridDimension => formatter.write_str("grid dimensions must be non-zero"),
            Self::GridPointCountOverflow => {
                formatter.write_str("grid dimensions exceed addressable memory")
            }
            Self::ZeroWorkerCount => formatter.write_str("CPU worker count must be non-zero"),
            Self::ThreadPoolInitializationFailed { message } => {
                write!(
                    formatter,
                    "CPU thread pool initialization failed: {message}"
                )
            }
        }
    }
}

impl std::error::Error for ComputeError {}

impl<KernelError> fmt::Display for ParallelExecutionError<KernelError>
where
    KernelError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroBlockLength => formatter.write_str("output block length must be non-zero"),
            Self::IncompleteOutputBlock {
                output_value_count,
                block_length,
            } => write!(
                formatter,
                "{output_value_count} output values do not form complete blocks of length {block_length}"
            ),
            Self::PairedOutputLengthMismatch {
                first_output_value_count,
                second_output_value_count,
            } => write!(
                formatter,
                "paired outputs contain {first_output_value_count} and {second_output_value_count} values"
            ),
            Self::Kernel(error) => write!(formatter, "numerical kernel failed: {error}"),
            Self::WorkerPanicked => formatter.write_str("a CPU execution worker panicked"),
        }
    }
}

impl<KernelError> std::error::Error for ParallelExecutionError<KernelError> where
    KernelError: std::error::Error + 'static
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_error_display_explains_invalid_worker_count() {
        assert_eq!(
            ComputeError::ZeroWorkerCount.to_string(),
            "CPU worker count must be non-zero"
        );
    }
}
