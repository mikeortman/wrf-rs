/// NetCDF primitive types used by the minimum WRF state schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WrfDataType {
    /// NetCDF `NC_CHAR` bytes.
    Character,
    /// Signed 32-bit integer.
    Int32,
    /// IEEE single precision.
    Float32,
    /// IEEE double precision.
    Float64,
}

impl WrfDataType {
    pub(crate) const fn byte_count(self) -> usize {
        match self {
            Self::Character => 1,
            Self::Int32 | Self::Float32 => 4,
            Self::Float64 => 8,
        }
    }
}
