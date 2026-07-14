use wrf_compute::{CpuField, FieldStorage};

use super::super::{DryBoundaryRelaxationBoundaryData, DryBoundaryRelaxationRegion};
use crate::{
    SpecifiedBoundaryRelaxationCoefficients, SpecifiedBoundaryRelaxationField,
    SpecifiedBoundaryRelaxationInputs,
};

pub(super) fn full_field_inputs<'a>(
    field: &'a CpuField<f32>,
    boundary: DryBoundaryRelaxationBoundaryData<'a, CpuField<f32>>,
    coefficients: SpecifiedBoundaryRelaxationCoefficients<'a>,
) -> SpecifiedBoundaryRelaxationInputs<'a, CpuField<f32>> {
    let shape = field.shape();
    SpecifiedBoundaryRelaxationInputs::new(
        SpecifiedBoundaryRelaxationField::new(
            field,
            0..shape.west_east_points(),
            0..shape.south_north_points(),
            0..shape.bottom_top_points(),
        ),
        boundary.values,
        boundary.tendencies,
        coefficients,
    )
}

pub(super) fn workspace_inputs<'a>(
    workspace: &'a CpuField<f32>,
    boundary: DryBoundaryRelaxationBoundaryData<'a, CpuField<f32>>,
    coefficients: SpecifiedBoundaryRelaxationCoefficients<'a>,
    region: &'a DryBoundaryRelaxationRegion,
) -> SpecifiedBoundaryRelaxationInputs<'a, CpuField<f32>> {
    let (west_east, south_north, bottom_top) = region.workspace_ranges();
    SpecifiedBoundaryRelaxationInputs::new(
        SpecifiedBoundaryRelaxationField::new(
            workspace,
            west_east.clone(),
            south_north.clone(),
            bottom_top.clone(),
        ),
        boundary.values,
        boundary.tendencies,
        coefficients,
    )
}
