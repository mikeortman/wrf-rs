use std::collections::HashMap;

use wrf_registry::{
    CoordinateAxis, DimensionLength, DimensionSpecification, ProcessorOrientation,
    RegistryDocument, RegistryValueType, StateStaggering, StateVariable,
};

use super::{RegistryDimensionTable, RestartStreamSelection, WrfMemoryOrder};
use crate::{
    WrfDataType, WrfDimensionName, WrfFileKind, WrfFileSchema, WrfGridDimensions, WrfIoError,
    WrfIoResult, WrfTimestamp, WrfVariableSchema,
};

const WRF_REAL_FIELD_TYPE: i32 = 104;
const WRF_DOUBLE_FIELD_TYPE: i32 = 105;
const WRF_INTEGER_FIELD_TYPE: i32 = 106;
const WRF_LOGICAL_FIELD_TYPE: i32 = 107;

/// Builds a restart NetCDF schema from borrowed, parsed WRF Registry metadata.
///
/// The builder mirrors WRF v4.7.1's Registry and classic NetCDF path at the
/// commit pinned in `UPSTREAM.toml`. It selects `r` states, resolves every
/// selected `dimspec`, applies WRF staggering and external axis ordering, and
/// emits one data name per generated time level. Registry metadata remains
/// borrowed and is never modified.
#[derive(Debug)]
pub struct WrfRegistryRestartSchemaBuilder<'registry> {
    registry: &'registry RegistryDocument,
    grid: WrfGridDimensions,
    start_date: WrfTimestamp,
    simulation_start_date: WrfTimestamp,
    west_east_spacing_meters: f32,
    south_north_spacing_meters: f32,
    namelist_values_by_name: HashMap<String, i64>,
}

impl<'registry> WrfRegistryRestartSchemaBuilder<'registry> {
    /// Creates a builder for one Registry document and ARW domain.
    ///
    /// Namelist-bounded dimensions must be supplied with
    /// [`Self::with_namelist_value`] before [`Self::try_build`]. Validation is
    /// deferred until `try_build` so the complete schema is checked in one
    /// deterministic preflight pass.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        registry: &'registry RegistryDocument,
        grid: WrfGridDimensions,
        start_date: WrfTimestamp,
        simulation_start_date: WrfTimestamp,
        west_east_spacing_meters: f32,
        south_north_spacing_meters: f32,
    ) -> Self {
        Self {
            registry,
            grid,
            start_date,
            simulation_start_date,
            west_east_spacing_meters,
            south_north_spacing_meters,
            namelist_values_by_name: HashMap::new(),
        }
    }

    /// Supplies one integer namelist value used by a Registry dimension bound.
    ///
    /// A later call with the same name replaces the earlier value, matching a
    /// resolved runtime configuration table rather than a source declaration.
    #[must_use]
    pub fn with_namelist_value(mut self, name: impl Into<String>, value: i64) -> Self {
        self.namelist_values_by_name.insert(name.into(), value);
        self
    }

    /// Validates and builds the complete restart schema without mutating input.
    ///
    /// Variable order follows Registry state order, and generated time levels
    /// are ordered from one through `ntl`. File dimensions follow WRF's
    /// first-use table: `Time`, `DateStrLen`, then external X/Y/Z first use.
    pub fn try_build(&self) -> WrfIoResult<WrfFileSchema> {
        let dimensions_by_name: HashMap<&str, &DimensionSpecification> = self
            .registry
            .dimensions()
            .map(|dimension| (dimension.name(), dimension))
            .collect();
        let mut dimension_table = RegistryDimensionTable::new();
        let mut variables = vec![Self::times_variable()?];

        for state in self.registry.state_variables() {
            if !RestartStreamSelection::is_selected(state)? {
                continue;
            }
            self.append_state_variables(
                state,
                &dimensions_by_name,
                &mut dimension_table,
                &mut variables,
            )?;
        }

        let dimensions = dimension_table.into_dimensions(1);
        let attributes = WrfFileSchema::arw_global_attributes(
            WrfFileKind::Restart,
            self.grid,
            &self.start_date,
            &self.simulation_start_date,
            self.west_east_spacing_meters,
            self.south_north_spacing_meters,
        )?;
        WrfFileSchema::try_from_parts(WrfFileKind::Restart, dimensions, attributes, variables)
    }

    fn append_state_variables(
        &self,
        state: &StateVariable,
        dimensions_by_name: &HashMap<&str, &DimensionSpecification>,
        dimension_table: &mut RegistryDimensionTable,
        variables: &mut Vec<WrfVariableSchema>,
    ) -> WrfIoResult<()> {
        Self::validate_supported_state_shape(state)?;
        let (data_type, field_type) = Self::map_value_type(state)?;
        let (file_dimensions, memory_order) =
            self.resolve_file_dimensions(state, dimensions_by_name, dimension_table)?;
        let stagger = Self::stagger_text(state.staggering());
        let registry_data_name = state.data_name().unwrap_or("");
        let data_name = if registry_data_name.is_empty() || registry_data_name.starts_with(' ') {
            state.name()
        } else {
            registry_data_name
        }
        .to_ascii_uppercase();
        let time_levels = state.time_levels().get();

        for time_level in 1..=time_levels {
            let variable_name = if time_levels > 1 {
                format!("{data_name}_{time_level}")
            } else {
                data_name.clone()
            };
            variables.push(WrfVariableSchema::wrf_field(
                variable_name.trim_end_matches(' '),
                data_type,
                field_type,
                file_dimensions.clone(),
                memory_order.attribute_text(),
                state.description().unwrap_or("-").trim_end_matches(' '),
                state.units().unwrap_or("-").trim_end_matches(' '),
                &stagger,
            )?);
        }
        Ok(())
    }

    fn validate_supported_state_shape(state: &StateVariable) -> WrfIoResult<()> {
        if state.dimensions().is_boundary_array() {
            return Err(WrfIoError::UnsupportedBoundaryArray {
                state: state.name().to_owned(),
            });
        }
        if state.dimensions().is_scalar_array_member() {
            return Err(WrfIoError::UnsupportedScalarArrayMember {
                state: state.name().to_owned(),
            });
        }
        if !state.dimensions().subgrid_positions().is_empty() {
            return Err(WrfIoError::UnsupportedSubgridDimensions {
                state: state.name().to_owned(),
            });
        }
        match state.dimensions().processor_orientation() {
            ProcessorOrientation::Z => {}
            ProcessorOrientation::X => {
                return Err(WrfIoError::UnsupportedProcessorOrientation {
                    state: state.name().to_owned(),
                    orientation: "X",
                });
            }
            ProcessorOrientation::Y => {
                return Err(WrfIoError::UnsupportedProcessorOrientation {
                    state: state.name().to_owned(),
                    orientation: "Y",
                });
            }
        }
        if matches!(state.value_type(), RegistryValueType::Logical)
            && state.dimensions().names().len() >= 3
        {
            return Err(WrfIoError::UnsupportedLogicalFieldDimensions {
                state: state.name().to_owned(),
                dimensions: state.dimensions().names().len(),
            });
        }
        Ok(())
    }

    fn resolve_file_dimensions(
        &self,
        state: &StateVariable,
        dimensions_by_name: &HashMap<&str, &DimensionSpecification>,
        dimension_table: &mut RegistryDimensionTable,
    ) -> WrfIoResult<(Vec<WrfDimensionName>, WrfMemoryOrder)> {
        let registry_dimensions = state
            .dimensions()
            .names()
            .iter()
            .map(|name| {
                dimensions_by_name
                    .get(name.as_str())
                    .copied()
                    .ok_or_else(|| WrfIoError::UnknownRegistryDimension {
                        state: state.name().to_owned(),
                        dimension: name.clone(),
                    })
            })
            .collect::<WrfIoResult<Vec<_>>>()?;
        let axes = registry_dimensions
            .iter()
            .map(|dimension| dimension.axis())
            .collect::<Vec<_>>();
        let memory_order = WrfMemoryOrder::try_new(state.name(), &axes)?;
        let mut external_dimensions = Vec::with_capacity(registry_dimensions.len());

        for registry_position in memory_order.external_permutation() {
            let dimension = registry_dimensions[registry_position];
            let length = self.resolve_dimension_length(state, dimension)?;
            let file_name = Self::resolve_dimension_name(state, dimension)?;
            let registered_name = match file_name {
                Some(name) => {
                    dimension_table.require_named(name.clone(), length)?;
                    name
                }
                None => dimension_table.require_anonymous(length)?,
            };
            external_dimensions.push(registered_name);
        }

        // The Fortran NetCDF interface reverses VDimIDs. WRF supplies external
        // X/Y/Z followed by Time, producing file order Time/Z/Y/X.
        let mut file_dimensions = Vec::with_capacity(external_dimensions.len() + 1);
        file_dimensions.push(WrfDimensionName::Time);
        file_dimensions.extend(external_dimensions.into_iter().rev());
        Ok((file_dimensions, memory_order))
    }

    fn resolve_dimension_length(
        &self,
        state: &StateVariable,
        dimension: &DimensionSpecification,
    ) -> WrfIoResult<usize> {
        match dimension.length() {
            DimensionLength::StandardDomain => self.standard_domain_length(state, dimension),
            DimensionLength::Namelist { start, end } => {
                let start = self.resolve_namelist_bound(dimension.name(), start)?;
                let end = self.resolve_namelist_bound(dimension.name(), end)?;
                Self::inclusive_length(dimension.name(), start, end)
            }
            DimensionLength::Constant { start, end } => {
                Self::inclusive_length(dimension.name(), i64::from(*start), i64::from(*end))
            }
        }
    }

    fn standard_domain_length(
        &self,
        state: &StateVariable,
        dimension: &DimensionSpecification,
    ) -> WrfIoResult<usize> {
        let is_staggered = Self::is_axis_staggered(state.staggering(), dimension.axis());
        match (dimension.axis(), is_staggered) {
            (CoordinateAxis::X, false) => Ok(self.grid.west_east()),
            (CoordinateAxis::X, true) => Ok(self.grid.west_east_staggered()),
            (CoordinateAxis::Y, false) => Ok(self.grid.south_north()),
            (CoordinateAxis::Y, true) => Ok(self.grid.south_north_staggered()),
            (CoordinateAxis::Z, false) => Ok(self.grid.bottom_top()),
            (CoordinateAxis::Z, true) => Ok(self.grid.bottom_top_staggered()),
            (CoordinateAxis::Constant, _) => Err(WrfIoError::UnsupportedStandardDomainAxis {
                dimension: dimension.name().to_owned(),
            }),
        }
    }

    fn resolve_namelist_bound(&self, dimension: &str, bound: &str) -> WrfIoResult<i64> {
        if let Ok(value) = bound.parse::<i64>() {
            return Ok(value);
        }
        self.namelist_values_by_name
            .get(bound)
            .copied()
            .ok_or_else(|| WrfIoError::MissingNamelistDimensionLength {
                dimension: dimension.to_owned(),
                namelist: bound.to_owned(),
            })
    }

    fn inclusive_length(dimension: &str, start: i64, end: i64) -> WrfIoResult<usize> {
        let Some(length) = end.checked_sub(start).and_then(|span| span.checked_add(1)) else {
            return Err(WrfIoError::RegistryDimensionLengthOverflow {
                dimension: dimension.to_owned(),
                start,
                end,
            });
        };
        if length <= 0 {
            return Err(WrfIoError::EmptyRegistryDimension {
                dimension: dimension.to_owned(),
                start,
                end,
            });
        }
        usize::try_from(length).map_err(|_| WrfIoError::RegistryDimensionLengthOverflow {
            dimension: dimension.to_owned(),
            start,
            end,
        })
    }

    fn resolve_dimension_name(
        state: &StateVariable,
        dimension: &DimensionSpecification,
    ) -> WrfIoResult<Option<WrfDimensionName>> {
        // `gen_allocs.c` only fills `dimname1..3` for X, Y, and Z. Constant
        // coordinate axes therefore reach `wrf_io.F90` unnamed even when the
        // Registry's dimspec carries a descriptive dataset label.
        if dimension.axis() == CoordinateAxis::Constant {
            return Ok(None);
        }
        let Some(data_name) = dimension.data_name() else {
            return Ok(None);
        };
        let name = if Self::is_axis_staggered(state.staggering(), dimension.axis()) {
            format!("{data_name}_stag")
        } else {
            data_name.to_owned()
        };
        let name = name.trim_end_matches(' ');
        if name.is_empty() {
            return Ok(None);
        }
        WrfDimensionName::try_from_name(name).map(Some)
    }

    fn is_axis_staggered(staggering: StateStaggering, axis: CoordinateAxis) -> bool {
        match axis {
            CoordinateAxis::X => staggering.is_x_staggered(),
            CoordinateAxis::Y => staggering.is_y_staggered(),
            CoordinateAxis::Z => staggering.is_z_staggered(),
            CoordinateAxis::Constant => false,
        }
    }

    fn map_value_type(state: &StateVariable) -> WrfIoResult<(WrfDataType, i32)> {
        match state.value_type() {
            RegistryValueType::Real => Ok((WrfDataType::Float32, WRF_REAL_FIELD_TYPE)),
            RegistryValueType::DoublePrecision => Ok((WrfDataType::Float64, WRF_DOUBLE_FIELD_TYPE)),
            RegistryValueType::Integer => Ok((WrfDataType::Int32, WRF_INTEGER_FIELD_TYPE)),
            RegistryValueType::Logical => Ok((WrfDataType::Int32, WRF_LOGICAL_FIELD_TYPE)),
            RegistryValueType::Character256 => Err(WrfIoError::UnsupportedRegistryValueType {
                state: state.name().to_owned(),
                value_type: state.value_type().to_string(),
            }),
        }
    }

    fn stagger_text(staggering: StateStaggering) -> String {
        let mut value = String::with_capacity(3);
        if staggering.is_x_staggered() {
            value.push('X');
        }
        if staggering.is_y_staggered() {
            value.push('Y');
        }
        if staggering.is_z_staggered() {
            value.push('Z');
        }
        value
    }

    fn times_variable() -> WrfIoResult<WrfVariableSchema> {
        WrfVariableSchema::try_new(
            "Times",
            WrfDataType::Character,
            vec![WrfDimensionName::Time, WrfDimensionName::DateStringLength],
            Vec::new(),
        )
    }
}

#[cfg(test)]
mod tests {
    use wrf_registry::RegistryParser;

    use crate::{WrfAttributeValue, WrfVariableName};

    use super::*;

    const COMPLETE_REGISTRY: &str = r#"dimspec i 1 standard_domain x west_east
dimspec k 2 standard_domain z bottom_top
dimspec j 3 standard_domain y south_north
dimspec s - namelist=1:num_soil_layers z soil_layers
dimspec n - constant=2 z modes
dimspec c - constant=(0:5) c categories
dimspec a - constant=(2:8) c -
state real temperature ikj dyn_em 1 - ir "T " "potential temperature " "K "
state real u ikj dyn_em 1 X ir U "x-wind component" "m s-1"
state real v ikj dyn_em 1 Y ir V "y-wind component" "m s-1"
state real w ikj dyn_em 1 Z ir W "z-wind component" "m s-1"
state integer land_mask ji misc 1 - ir LANDMASK "land mask" "1"
state doubleprecision energy ikj dyn_em 1 - ir ENERGY "total energy" "J"
state logical active ij misc 1 - ir ACTIVE "active cell" "1"
state real soil s misc 1 - ir SOIL "soil state" "kg kg-1"
state real soil_staggered s misc 1 Z ir SOILSTAG "staggered soil state" "kg kg-1"
state real mode n misc 1 - ir MODE "mode state" "1"
state real mode_staggered n misc 1 Z ir MODESTAG "staggered mode state" "1"
state integer category c misc 1 - ir CATEGORY "category code" "1"
state real anonymous a misc 1 - ir ANON "anonymous coordinate" "1"
state real xtime - misc 1 - ir " ignored" "minutes since start" "minutes"
state real tendency ikj dyn_em 2 - ir TEND "time-level tendency" "K s-1"
state character*256 note - misc 1 - h NOTE "history only" "-"
"#;

    #[test]
    fn try_build_resolves_complete_registry_restart_schema_and_metadata() {
        let registry = RegistryParser::parse("Registry.fixture", COMPLETE_REGISTRY).unwrap();
        let schema = fixture_builder(&registry)
            .with_namelist_value("num_soil_layers", 4)
            .try_build()
            .unwrap();

        let dimensions = schema
            .dimensions()
            .iter()
            .map(|dimension| (dimension.name().as_str(), dimension.length()))
            .collect::<Vec<_>>();
        assert_eq!(
            dimensions,
            vec![
                ("Time", 1),
                ("DateStrLen", 19),
                ("west_east", 4),
                ("south_north", 3),
                ("bottom_top", 2),
                ("west_east_stag", 5),
                ("south_north_stag", 4),
                ("bottom_top_stag", 3),
                ("soil_layers", 4),
                ("soil_layers_stag", 4),
                ("modes", 2),
                ("modes_stag", 2),
                ("DIM0012", 6),
                ("DIM0013", 7),
            ]
        );

        let variable_names = schema
            .variables()
            .iter()
            .map(|variable| variable.name().as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            variable_names,
            vec![
                "Times", "T", "U", "V", "W", "LANDMASK", "ENERGY", "ACTIVE", "SOIL", "SOILSTAG",
                "MODE", "MODESTAG", "CATEGORY", "ANON", "XTIME", "TEND_1", "TEND_2",
            ]
        );
        assert_eq!(schema.variables().len(), 17);

        let temperature = variable(&schema, "T");
        assert_eq!(
            temperature.dimensions(),
            &[
                WrfDimensionName::Time,
                WrfDimensionName::BottomTop,
                WrfDimensionName::SouthNorth,
                WrfDimensionName::WestEast,
            ]
        );
        assert_eq!(attribute_text(temperature, "MemoryOrder"), "XYZ");
        assert_eq!(
            attribute_text(temperature, "description"),
            "potential temperature"
        );
        assert_eq!(attribute_text(temperature, "units"), "K");
        assert_eq!(attribute_text(temperature, "stagger"), "");
        assert_eq!(attribute_i32(temperature, "FieldType"), 104);

        let land_mask = variable(&schema, "LANDMASK");
        assert_eq!(land_mask.data_type(), WrfDataType::Int32);
        assert_eq!(attribute_i32(land_mask, "FieldType"), 106);
        assert_eq!(attribute_text(land_mask, "MemoryOrder"), "XY ");
        assert_eq!(
            land_mask.dimensions(),
            &[
                WrfDimensionName::Time,
                WrfDimensionName::SouthNorth,
                WrfDimensionName::WestEast,
            ]
        );

        assert_eq!(
            variable(&schema, "ENERGY").data_type(),
            WrfDataType::Float64
        );
        assert_eq!(attribute_i32(variable(&schema, "ENERGY"), "FieldType"), 105);
        assert_eq!(variable(&schema, "ACTIVE").data_type(), WrfDataType::Int32);
        assert_eq!(attribute_i32(variable(&schema, "ACTIVE"), "FieldType"), 107);
        assert_eq!(
            attribute_text(variable(&schema, "SOILSTAG"), "stagger"),
            "Z"
        );
        assert_eq!(
            variable(&schema, "CATEGORY")
                .dimensions()
                .last()
                .unwrap()
                .as_str(),
            "DIM0012"
        );
        assert_eq!(
            variable(&schema, "ANON")
                .dimensions()
                .last()
                .unwrap()
                .as_str(),
            "DIM0013"
        );
        assert_eq!(
            variable(&schema, "XTIME").dimensions(),
            &[WrfDimensionName::Time]
        );
    }

    #[test]
    fn try_build_rejects_missing_and_degenerate_namelist_bounds() {
        let source = "dimspec s - namelist=soil_start:num_soil_layers z soil_layers\n\
state real soil s misc 1 - r SOIL \"soil\" \"1\"\n";
        let registry = RegistryParser::parse("Registry.fixture", source).unwrap();
        assert!(matches!(
            fixture_builder(&registry).try_build(),
            Err(WrfIoError::MissingNamelistDimensionLength { namelist, .. })
                if namelist == "soil_start"
        ));
        assert!(matches!(
            fixture_builder(&registry)
                .with_namelist_value("soil_start", 5)
                .with_namelist_value("num_soil_layers", 4)
                .try_build(),
            Err(WrfIoError::EmptyRegistryDimension {
                start: 5,
                end: 4,
                ..
            })
        ));
    }

    #[test]
    fn try_build_rejects_out_of_scope_selected_registry_shapes_and_types() {
        let cases = [
            (
                "dimspec i 1 standard_domain x west_east\n\
dimspec j 3 standard_domain y south_north\n\
state real subgrid *ij dyn_em 1 - r SUBGRID \"subgrid\" \"1\"\n",
                "subgrid",
            ),
            (
                "dimspec i 1 standard_domain x west_east\n\
dimspec j 3 standard_domain y south_north\n\
state real boundary ijb dyn_em 1 - r BOUNDARY \"boundary\" \"1\"\n",
                "boundary",
            ),
            (
                "dimspec i 1 standard_domain x west_east\n\
dimspec j 3 standard_domain y south_north\n\
state real scalar_member ijf dyn_em 1 - r SCALAR \"scalar\" \"1\"\n",
                "scalar",
            ),
            (
                "state character*256 character_state - misc 1 - r TEXT \"text\" \"1\"\n",
                "character",
            ),
            (
                "dimspec i 1 standard_domain x west_east\n\
dimspec k 2 standard_domain z bottom_top\n\
dimspec j 3 standard_domain y south_north\n\
state logical logical_volume ikj misc 1 - r LOGICAL_VOLUME \"logical\" \"1\"\n",
                "logical_volume",
            ),
            (
                "dimspec i 1 standard_domain x west_east\n\
dimspec k 2 standard_domain z bottom_top\n\
dimspec j 3 standard_domain y south_north\n\
state real xposed ikjx misc 1 - r XPOSED \"xposed\" \"1\"\n",
                "orientation",
            ),
            (
                "dimspec i 1 standard_domain x west_east\n\
state real x_only i dyn_em 1 - r XONLY \"x only\" \"1\"\n",
                "memory",
            ),
            (
                "dimspec c 1 standard_domain c constant_axis\n\
state real invalid c misc 1 - r INVALID \"invalid\" \"1\"\n",
                "standard",
            ),
        ];

        for (source, expected) in cases {
            let registry = RegistryParser::parse("Registry.fixture", source).unwrap();
            let error = fixture_builder(&registry).try_build().unwrap_err();
            match expected {
                "subgrid" => assert!(matches!(
                    error,
                    WrfIoError::UnsupportedSubgridDimensions { .. }
                )),
                "boundary" => {
                    assert!(matches!(error, WrfIoError::UnsupportedBoundaryArray { .. }))
                }
                "scalar" => assert!(matches!(
                    error,
                    WrfIoError::UnsupportedScalarArrayMember { .. }
                )),
                "character" => assert!(matches!(
                    error,
                    WrfIoError::UnsupportedRegistryValueType { .. }
                )),
                "logical_volume" => assert!(matches!(
                    error,
                    WrfIoError::UnsupportedLogicalFieldDimensions { dimensions: 3, .. }
                )),
                "orientation" => assert!(matches!(
                    error,
                    WrfIoError::UnsupportedProcessorOrientation {
                        orientation: "X",
                        ..
                    }
                )),
                "memory" => {
                    assert!(matches!(error, WrfIoError::UnsupportedMemoryOrder { .. }))
                }
                "standard" => assert!(matches!(
                    error,
                    WrfIoError::UnsupportedStandardDomainAxis { .. }
                )),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn try_build_ignores_unselected_unsupported_state_and_accepts_empty_selection() {
        let registry = RegistryParser::parse(
            "Registry.fixture",
            "dimspec i 1 standard_domain x west_east\n\
dimspec k 2 standard_domain z bottom_top\n\
dimspec j 3 standard_domain y south_north\n\
state character*256 note - misc 1 - h NOTE \"history only\" \"-\"\n\
state real xposed ikjy misc 1 - h XPOSED \"history xposed\" \"1\"\n",
        )
        .unwrap();
        let schema = fixture_builder(&registry).try_build().unwrap();

        assert_eq!(schema.variables().len(), 1);
        assert_eq!(schema.variables()[0].name().as_str(), "Times");
        assert_eq!(schema.dimensions().len(), 2);
    }

    #[test]
    fn try_build_rejects_reserved_and_conflicting_dimension_names() {
        let reserved = RegistryParser::parse(
            "Registry.fixture",
            "dimspec c - constant=2 z \"Time\"\n\
state real value c misc 1 - r VALUE \"value\" \"1\"\n",
        )
        .unwrap();
        let reserved_result = fixture_builder(&reserved).try_build();
        assert!(
            matches!(
                &reserved_result,
                Err(WrfIoError::ReservedRegistryDimensionName { .. })
            ),
            "unexpected result: {reserved_result:?}"
        );

        let conflicting = RegistryParser::parse(
            "Registry.fixture",
            "dimspec n - constant=2 z shared\n\
dimspec q - constant=3 z shared\n\
state real first n misc 1 - r FIRST \"first\" \"1\"\n\
state real second q misc 1 - r SECOND \"second\" \"1\"\n",
        )
        .unwrap();
        assert!(matches!(
            fixture_builder(&conflicting).try_build(),
            Err(WrfIoError::DimensionLengthConflict {
                existing: 2,
                requested: 3,
                ..
            })
        ));
    }

    #[test]
    fn try_build_is_deterministic_across_threads() {
        let registry = RegistryParser::parse("Registry.fixture", COMPLETE_REGISTRY).unwrap();
        let expected = fixture_builder(&registry)
            .with_namelist_value("num_soil_layers", 4)
            .try_build()
            .unwrap();

        std::thread::scope(|scope| {
            let handles = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        fixture_builder(&registry)
                            .with_namelist_value("num_soil_layers", 4)
                            .try_build()
                            .unwrap()
                    })
                })
                .collect::<Vec<_>>();
            for handle in handles {
                assert_eq!(handle.join().unwrap(), expected);
            }
        });
    }

    fn fixture_builder(registry: &RegistryDocument) -> WrfRegistryRestartSchemaBuilder<'_> {
        WrfRegistryRestartSchemaBuilder::new(
            registry,
            WrfGridDimensions::try_new(4, 3, 2).unwrap(),
            WrfTimestamp::try_new("2000-09-18_16:42:01").unwrap(),
            WrfTimestamp::try_new("2000-09-18_16:00:00").unwrap(),
            12_000.0,
            12_000.0,
        )
    }

    fn variable<'schema>(schema: &'schema WrfFileSchema, name: &str) -> &'schema WrfVariableSchema {
        schema
            .variable(&WrfVariableName::try_new(name).unwrap())
            .unwrap()
    }

    fn attribute_text<'variable>(
        variable: &'variable WrfVariableSchema,
        name: &str,
    ) -> &'variable str {
        variable
            .attributes()
            .iter()
            .find(|attribute| attribute.name() == name)
            .and_then(|attribute| match attribute.value() {
                WrfAttributeValue::Text(value) => Some(value.as_str()),
                _ => None,
            })
            .unwrap()
    }

    fn attribute_i32(variable: &WrfVariableSchema, name: &str) -> i32 {
        variable
            .attributes()
            .iter()
            .find(|attribute| attribute.name() == name)
            .and_then(|attribute| match attribute.value() {
                WrfAttributeValue::Int32(values) => values.first().copied(),
                _ => None,
            })
            .unwrap()
    }
}
