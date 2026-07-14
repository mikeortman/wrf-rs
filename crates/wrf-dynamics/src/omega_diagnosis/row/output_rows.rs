use crate::OmegaDiagnosisGridMetrics;

use super::{
    OmegaDiagnosisLevelCoefficients, OmegaDiagnosisMapFactorRows, OmegaDiagnosisMassRows,
    OmegaDiagnosisVelocityRows,
};

pub(crate) struct OmegaDiagnosisOutputRows<'a> {
    divergence: &'a mut [f32],
    column_mass_tendency: &'a mut [f32],
}

impl<'a> OmegaDiagnosisOutputRows<'a> {
    pub(crate) fn new(divergence: &'a mut [f32], column_mass_tendency: &'a mut [f32]) -> Self {
        assert_eq!(divergence.len(), column_mass_tendency.len());
        Self {
            divergence,
            column_mass_tendency,
        }
    }

    pub(crate) fn calculate_and_accumulate(
        &mut self,
        velocities: &OmegaDiagnosisVelocityRows<'_>,
        masses: &OmegaDiagnosisMassRows<'_>,
        map_factors: &OmegaDiagnosisMapFactorRows<'_>,
        coefficients: OmegaDiagnosisLevelCoefficients,
        grid_metrics: OmegaDiagnosisGridMetrics,
    ) {
        let point_count = self.divergence.len();
        assert_eq!(velocities.point_count(), point_count);
        assert_eq!(masses.point_count(), point_count);
        assert_eq!(map_factors.point_count(), point_count);

        for point_index in 0..point_count {
            let west_east_mass = 0.5
                * (masses.perturbation_current[point_index]
                    + masses.base_current[point_index]
                    + masses.perturbation_west[point_index]
                    + masses.base_west[point_index]);
            let east_mass = 0.5
                * (masses.perturbation_east[point_index]
                    + masses.base_east[point_index]
                    + masses.perturbation_current[point_index]
                    + masses.base_current[point_index]);
            let south_north_mass = 0.5
                * (masses.perturbation_current[point_index]
                    + masses.base_current[point_index]
                    + masses.perturbation_south[point_index]
                    + masses.base_south[point_index]);
            let north_mass = 0.5
                * (masses.perturbation_north[point_index]
                    + masses.base_north[point_index]
                    + masses.perturbation_current[point_index]
                    + masses.base_current[point_index]);
            let west_east_flux = (coefficients.mass_multiplier * west_east_mass
                + coefficients.mass_offset)
                * velocities.west_east[point_index]
                / map_factors.west_east_momentum_south_north[point_index];
            let east_flux = (coefficients.mass_multiplier * east_mass + coefficients.mass_offset)
                * velocities.east[point_index]
                / map_factors.east_momentum_south_north[point_index];
            let south_north_flux = (coefficients.mass_multiplier * south_north_mass
                + coefficients.mass_offset)
                * velocities.south_north[point_index]
                * map_factors.inverse_south_north_momentum_west_east[point_index];
            let north_flux = (coefficients.mass_multiplier * north_mass + coefficients.mass_offset)
                * velocities.north[point_index]
                * map_factors.inverse_north_momentum_west_east[point_index];
            let divergence = map_factors.mass_point_west_east[point_index]
                * coefficients.eta_layer_thickness
                * (grid_metrics.inverse_west_east_spacing * (east_flux - west_east_flux)
                    + grid_metrics.inverse_south_north_spacing * (north_flux - south_north_flux));

            self.divergence[point_index] = divergence;
            self.column_mass_tendency[point_index] += divergence;
        }
    }
}
