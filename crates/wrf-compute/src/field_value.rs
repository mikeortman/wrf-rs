/// A lightweight scalar that may be stored in a numerical field.
///
/// Restricting fields to copyable scalars prevents accidental per-grid-point
/// heap ownership and gives a future device backend a conservative value set.
pub trait FieldValue: Copy + Send + Sync + 'static {}

macro_rules! implement_field_value {
    ($($value_type:ty),+ $(,)?) => {
        $(impl FieldValue for $value_type {})+
    };
}

implement_field_value!(f32, f64, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
