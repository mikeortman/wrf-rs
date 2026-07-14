use wrf_compute::{CpuField, FieldStorage};

use crate::{
    SpecifiedBoundaryUpdateError, SpecifiedBoundaryUpdateRegion, SpecifiedBoundaryUpdateResult,
};

pub(super) fn validate(
    field: &CpuField<f32>,
    tendency: &CpuField<f32>,
    region: &SpecifiedBoundaryUpdateRegion,
) -> SpecifiedBoundaryUpdateResult<()> {
    validate_shape("field", field, region)?;
    validate_shape("tendency", tendency, region)
}

fn validate_shape(
    role: &'static str,
    field: &CpuField<f32>,
    region: &SpecifiedBoundaryUpdateRegion,
) -> SpecifiedBoundaryUpdateResult<()> {
    if field.shape() != region.shape() {
        return Err(SpecifiedBoundaryUpdateError::ShapeMismatch {
            field: role,
            expected: region.shape(),
            actual: field.shape(),
        });
    }
    Ok(())
}
