use crate::{FieldStorage, FieldValue, GridShape};

/// Contiguous host storage for a WRF field.
#[derive(Clone, Debug, PartialEq)]
pub struct CpuField<Value> {
    shape: GridShape,
    values: Vec<Value>,
}

impl<Value> CpuField<Value>
where
    Value: FieldValue,
{
    pub(crate) fn from_value(shape: GridShape, initial_value: Value) -> Self {
        Self {
            shape,
            values: vec![initial_value; shape.point_count()],
        }
    }

    /// Returns immutable contiguous host values in WRF linear order.
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// Returns mutable contiguous host values in WRF linear order.
    pub fn values_mut(&mut self) -> &mut [Value] {
        &mut self.values
    }
}

impl<Value> FieldStorage<Value> for CpuField<Value>
where
    Value: FieldValue,
{
    fn shape(&self) -> GridShape {
        self.shape
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_initializes_every_grid_point() {
        let shape = GridShape::try_new(2, 3, 4).unwrap();
        let field = CpuField::from_value(shape, 7.0_f32);

        assert_eq!(field.values(), vec![7.0_f32; 24]);
        assert_eq!(field.shape(), shape);
    }
}
