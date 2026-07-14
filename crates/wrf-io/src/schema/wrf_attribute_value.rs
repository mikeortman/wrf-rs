/// A typed NetCDF attribute value supported by WRF state files.
#[derive(Clone, Debug)]
pub enum WrfAttributeValue {
    /// NetCDF character text.
    Text(String),
    /// One or more signed 32-bit integers.
    Int32(Vec<i32>),
    /// One or more exact single-precision values.
    Float32(Vec<f32>),
    /// One or more exact double-precision values.
    Float64(Vec<f64>),
}

impl PartialEq for WrfAttributeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(left), Self::Text(right)) => left == right,
            (Self::Int32(left), Self::Int32(right)) => left == right,
            (Self::Float32(left), Self::Float32(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right)
                        .all(|(left, right)| left.to_bits() == right.to_bits())
            }
            (Self::Float64(left), Self::Float64(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right)
                        .all(|(left, right)| left.to_bits() == right.to_bits())
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_preserves_float_bits_including_nan_payloads_and_signed_zero() {
        assert_ne!(
            WrfAttributeValue::Float32(vec![0.0]),
            WrfAttributeValue::Float32(vec![-0.0])
        );
        assert_eq!(
            WrfAttributeValue::Float32(vec![f32::from_bits(0x7fc0_0042)]),
            WrfAttributeValue::Float32(vec![f32::from_bits(0x7fc0_0042)])
        );
    }
}
