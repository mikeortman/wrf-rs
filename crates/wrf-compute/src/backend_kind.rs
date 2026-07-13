/// The physical processor family executing a numerical kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendKind {
    /// Native host CPU execution.
    Cpu,
    /// Device GPU execution, reserved for a future backend implementation.
    Gpu,
}
