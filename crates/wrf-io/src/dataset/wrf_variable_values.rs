use crate::WrfDataType;

/// Borrowed values for one typed WRF variable.
#[derive(Clone, Copy, Debug)]
pub enum WrfVariableValues<'a> {
    /// NetCDF `NC_CHAR` bytes.
    Character(&'a [u8]),
    /// Signed 32-bit values.
    Int32(&'a [i32]),
    /// Single-precision values.
    Float32(&'a [f32]),
    /// Double-precision values.
    Float64(&'a [f64]),
}

impl WrfVariableValues<'_> {
    /// Returns the represented NetCDF primitive.
    pub const fn data_type(self) -> WrfDataType {
        match self {
            Self::Character(_) => WrfDataType::Character,
            Self::Int32(_) => WrfDataType::Int32,
            Self::Float32(_) => WrfDataType::Float32,
            Self::Float64(_) => WrfDataType::Float64,
        }
    }

    /// Returns the number of represented elements.
    pub const fn len(self) -> usize {
        match self {
            Self::Character(values) => values.len(),
            Self::Int32(values) => values.len(),
            Self::Float32(values) => values.len(),
            Self::Float64(values) => values.len(),
        }
    }

    /// Reports whether no values are present.
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }
}
