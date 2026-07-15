use std::ops::Range;

use wrf_compute::GridShape;
use wrf_dynamics::{
    AcousticFluxAccumulationRegion, AcousticHorizontalMomentumRegion, AcousticMassThetaRegion,
    AcousticPressureRegion, AcousticStepFinalizationRegion, AcousticStepPreparationRegion,
    AcousticTrajectoryRegions, AcousticVerticalRegion, ColumnMassStaggeringRegion,
    DryTendencyAssemblyRegion, InverseDensityRegion, MoistureCoefficientRegion,
    MomentumCouplingRegion, OmegaDiagnosisRegion, PressurePointGeopotentialRegion,
    RungeKuttaPreparationRegions, VerticalAcousticCoefficientRegion,
};
use wrf_physics::{MicrophysicsBoundaryPolicy, MicrophysicsDriverDomain, MicrophysicsTile};

use crate::{ArwModelError, ArwModelResult};

/// One local padded ARW domain and the accepted-stage active ranges.
pub struct ArwModelGeometry {
    shape: GridShape,
    microphysics_shape: GridShape,
    mass_west_east: Range<usize>,
    mass_south_north: Range<usize>,
    half_levels: Range<usize>,
    tile_west_east: Range<usize>,
    tile_south_north: Range<usize>,
    tile_bottom_top: Range<usize>,
    pub(crate) runge_kutta: RungeKuttaPreparationRegions,
    pub(crate) final_column_mass: ColumnMassStaggeringRegion,
    pub(crate) dry_tendency: DryTendencyAssemblyRegion,
    acoustic_preparation: AcousticStepPreparationRegion,
    acoustic_pressure: AcousticPressureRegion,
    vertical_coefficients: VerticalAcousticCoefficientRegion,
    horizontal_momentum: AcousticHorizontalMomentumRegion,
    mass_theta: AcousticMassThetaRegion,
    vertical_momentum: AcousticVerticalRegion,
    flux_accumulation: AcousticFluxAccumulationRegion,
    pub(crate) finalization: AcousticStepFinalizationRegion,
    pub(crate) microphysics_domain: MicrophysicsDriverDomain,
    pub(crate) microphysics_tiles: Vec<MicrophysicsTile>,
}

impl ArwModelGeometry {
    /// Builds the WRF-oracle-compatible padded domain around active mass points.
    ///
    /// One lower horizontal halo point, one upper stagger point, and one
    /// lower/upper vertical storage point are retained. The minimum vertical
    /// extent is three mass levels because the accepted implicit acoustic solve
    /// requires that many.
    ///
    /// # Errors
    ///
    /// Returns a typed invalid-geometry error if any accepted component rejects
    /// the derived ranges.
    pub fn try_new(
        active_west_east_points: usize,
        active_south_north_points: usize,
        half_level_count: usize,
    ) -> ArwModelResult<Self> {
        let shape = GridShape::try_new(
            active_west_east_points
                .checked_add(2)
                .ok_or(wrf_compute::ComputeError::GridPointCountOverflow)?,
            active_south_north_points
                .checked_add(2)
                .ok_or(wrf_compute::ComputeError::GridPointCountOverflow)?,
            half_level_count
                .checked_add(2)
                .ok_or(wrf_compute::ComputeError::GridPointCountOverflow)?,
        )?;
        let mass_west_east = 1..active_west_east_points + 1;
        let mass_south_north = 1..active_south_north_points + 1;
        let half_levels = 1..half_level_count + 1;
        let tile_west_east = 1..active_west_east_points + 2;
        let tile_south_north = 1..active_south_north_points + 2;
        let tile_bottom_top = 1..half_level_count + 2;

        let column_mass = ColumnMassStaggeringRegion::try_new(
            shape.horizontal_shape(),
            mass_west_east.clone(),
            mass_south_north.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
        )
        .map_err(|_| invalid("column-mass staggering"))?;
        let momentum = MomentumCouplingRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|_| invalid("momentum coupling"))?;
        let omega = OmegaDiagnosisRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|_| invalid("omega diagnosis"))?;
        let moisture = MoistureCoefficientRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|_| invalid("moisture coefficients"))?;
        let inverse_density = InverseDensityRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
        )
        .map_err(|_| invalid("inverse density"))?;
        let pressure_point_geopotential = PressurePointGeopotentialRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
        )
        .map_err(|_| invalid("pressure-point geopotential"))?;
        let final_column_mass = column_mass.clone();
        let runge_kutta = RungeKuttaPreparationRegions::new(
            column_mass,
            momentum,
            omega,
            moisture,
            inverse_density,
            pressure_point_geopotential,
        );
        let dry_tendency = DryTendencyAssemblyRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|_| invalid("dry tendency assembly"))?;
        let acoustic_preparation = AcousticStepPreparationRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|_| invalid("acoustic preparation"))?;
        let acoustic_pressure = AcousticPressureRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
        )
        .map_err(|_| invalid("acoustic pressure"))?;
        let vertical_coefficients = VerticalAcousticCoefficientRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            mass_west_east.clone(),
            mass_south_north.clone(),
        )
        .map_err(|_| invalid("vertical acoustic coefficients"))?;
        let horizontal_momentum = AcousticHorizontalMomentumRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|_| invalid("acoustic horizontal momentum"))?;
        let mass_theta = AcousticMassThetaRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            mass_west_east.clone(),
            mass_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|_| invalid("acoustic mass and theta"))?;
        let vertical_momentum = AcousticVerticalRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            mass_west_east.clone(),
            mass_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|_| invalid("acoustic vertical momentum"))?;
        let flux_accumulation = AcousticFluxAccumulationRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
            tile_bottom_top.clone(),
        )
        .map_err(|_| invalid("acoustic flux accumulation"))?;
        let finalization = AcousticStepFinalizationRegion::try_new(
            shape,
            mass_west_east.clone(),
            mass_south_north.clone(),
            half_levels.clone(),
            tile_west_east.clone(),
            tile_south_north.clone(),
        )
        .map_err(|_| invalid("acoustic step finalization"))?;
        let microphysics_domain = MicrophysicsDriverDomain::try_new(
            GridShape::try_new(
                shape.west_east_points(),
                shape.south_north_points(),
                half_level_count,
            )?,
            mass_west_east.clone(),
            mass_south_north.clone(),
            0..half_level_count,
            MicrophysicsBoundaryPolicy::new(false, false, 0),
        )
        .map_err(|_| invalid("Kessler microphysics"))?;
        let microphysics_shape = microphysics_domain.field_shape();
        let microphysics_tiles = vec![MicrophysicsTile::new(
            mass_west_east.clone(),
            mass_south_north.clone(),
        )];
        Ok(Self {
            shape,
            microphysics_shape,
            mass_west_east,
            mass_south_north,
            half_levels,
            tile_west_east,
            tile_south_north,
            tile_bottom_top,
            runge_kutta,
            final_column_mass,
            dry_tendency,
            acoustic_preparation,
            acoustic_pressure,
            vertical_coefficients,
            horizontal_momentum,
            mass_theta,
            vertical_momentum,
            flux_accumulation,
            finalization,
            microphysics_domain,
            microphysics_tiles,
        })
    }

    /// Returns the common padded dynamics and mass-field shape.
    #[must_use]
    pub const fn shape(&self) -> GridShape {
        self.shape
    }

    /// Returns the compact surface-based mass shape used by microphysics.
    #[must_use]
    pub const fn microphysics_shape(&self) -> GridShape {
        self.microphysics_shape
    }

    pub(crate) fn acoustic_regions(&self) -> AcousticTrajectoryRegions<'_> {
        AcousticTrajectoryRegions::new(
            &self.acoustic_preparation,
            &self.acoustic_pressure,
            &self.vertical_coefficients,
            &self.horizontal_momentum,
            &self.mass_theta,
            &self.vertical_momentum,
            &self.flux_accumulation,
        )
    }

    /// Returns the active unstaggered west-east mass range.
    #[must_use]
    pub fn mass_west_east(&self) -> Range<usize> {
        self.mass_west_east.clone()
    }

    /// Returns the active unstaggered south-north mass range.
    #[must_use]
    pub fn mass_south_north(&self) -> Range<usize> {
        self.mass_south_north.clone()
    }

    /// Returns the active half-level range.
    #[must_use]
    pub fn half_levels(&self) -> Range<usize> {
        self.half_levels.clone()
    }

    /// Returns the west-east tile range including its upper stagger point.
    #[must_use]
    pub fn tile_west_east(&self) -> Range<usize> {
        self.tile_west_east.clone()
    }

    /// Returns the south-north tile range including its upper stagger point.
    #[must_use]
    pub fn tile_south_north(&self) -> Range<usize> {
        self.tile_south_north.clone()
    }

    /// Returns the vertical tile range including its upper stagger level.
    #[must_use]
    pub fn tile_bottom_top(&self) -> Range<usize> {
        self.tile_bottom_top.clone()
    }
}

fn invalid(component: &'static str) -> ArwModelError {
    ArwModelError::InvalidGeometry { component }
}
