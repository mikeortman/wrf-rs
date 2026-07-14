use wrf_compute::CpuField;

use crate::{
    AcousticHorizontalMomentumParameters, AcousticHorizontalMomentumRegion,
    AcousticHorizontalVerticalCoefficients, VerticalAcousticTopBoundary,
};

#[derive(Clone, Copy)]
pub(super) struct PressureInterpolation<'a> {
    pressure: &'a [f32],
    vertical: AcousticHorizontalVerticalCoefficients<'a>,
    boundary_weights: [f32; 3],
    lower_half_level: usize,
    upper_full_level: usize,
    top_boundary: VerticalAcousticTopBoundary,
    west_east_points: usize,
    bottom_top_points: usize,
}

impl<'a> PressureInterpolation<'a> {
    pub(super) fn new(
        pressure: &'a CpuField<f32>,
        vertical: AcousticHorizontalVerticalCoefficients<'a>,
        parameters: AcousticHorizontalMomentumParameters,
        region: &AcousticHorizontalMomentumRegion,
    ) -> Self {
        let half_levels = region.half_level_domain();
        let shape = region.shape();
        Self {
            pressure: pressure.values(),
            vertical,
            boundary_weights: parameters.boundary_pressure_weights,
            lower_half_level: half_levels.start,
            upper_full_level: half_levels.end,
            top_boundary: parameters.top_boundary,
            west_east_points: shape.west_east_points(),
            bottom_top_points: shape.bottom_top_points(),
        }
    }

    pub(super) fn west_east(&self, full_level: usize, west_east: usize, south_north: usize) -> f32 {
        self.interpolate(full_level, |half_level| {
            self.pressure[self.volume_index(west_east, half_level, south_north)]
                + self.pressure[self.volume_index(west_east - 1, half_level, south_north)]
        })
    }

    pub(super) fn south_north(
        &self,
        full_level: usize,
        west_east: usize,
        south_north: usize,
    ) -> f32 {
        self.interpolate(full_level, |half_level| {
            self.pressure[self.volume_index(west_east, half_level, south_north)]
                + self.pressure[self.volume_index(west_east, half_level, south_north - 1)]
        })
    }

    fn interpolate(&self, full_level: usize, pressure_pair: impl Fn(usize) -> f32) -> f32 {
        if full_level == self.lower_half_level {
            let [first, second, third] = self.boundary_weights;
            0.5 * (first * pressure_pair(self.lower_half_level)
                + second * pressure_pair(self.lower_half_level + 1)
                + third * pressure_pair(self.lower_half_level + 2))
        } else if full_level == self.upper_full_level {
            match self.top_boundary {
                VerticalAcousticTopBoundary::Nonrigid => 0.0,
                VerticalAcousticTopBoundary::RigidLid => {
                    let [first, second, third] = self.boundary_weights;
                    0.5 * (first * pressure_pair(self.upper_full_level - 1)
                        + second * pressure_pair(self.upper_full_level - 2)
                        + third * pressure_pair(self.upper_full_level - 3))
                }
            }
        } else {
            0.5 * (self.vertical.lower_interpolation_weight[full_level] * pressure_pair(full_level)
                + self.vertical.upper_interpolation_weight[full_level]
                    * pressure_pair(full_level - 1))
        }
    }

    const fn volume_index(&self, west_east: usize, bottom_top: usize, south_north: usize) -> usize {
        (south_north * self.bottom_top_points + bottom_top) * self.west_east_points + west_east
    }
}
