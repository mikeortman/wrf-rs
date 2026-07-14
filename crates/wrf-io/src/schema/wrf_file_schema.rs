use std::collections::HashSet;

use crate::{
    WrfAttribute, WrfAttributeValue, WrfDataType, WrfDimension, WrfDimensionName, WrfFileKind,
    WrfGridDimensions, WrfIoError, WrfIoResult, WrfTimestamp, WrfVariableName, WrfVariableSchema,
};

/// Complete typed schema for the minimum ARW initialization/restart slice.
#[derive(Clone, Debug, PartialEq)]
pub struct WrfFileSchema {
    file_kind: WrfFileKind,
    dimensions: Vec<WrfDimension>,
    attributes: Vec<WrfAttribute>,
    variables: Vec<WrfVariableSchema>,
}

impl WrfFileSchema {
    /// Builds the dependency-closed ARW core schema used by the first I/O slice.
    #[allow(clippy::too_many_arguments)]
    pub fn try_minimal_arw(
        file_kind: WrfFileKind,
        grid: WrfGridDimensions,
        start_date: WrfTimestamp,
        simulation_start_date: WrfTimestamp,
        west_east_spacing_meters: f32,
        south_north_spacing_meters: f32,
    ) -> WrfIoResult<Self> {
        Self::validate_spacing("west-east", west_east_spacing_meters)?;
        Self::validate_spacing("south-north", south_north_spacing_meters)?;

        let dimensions = vec![
            WrfDimension::unlimited(WrfDimensionName::Time, 1),
            WrfDimension::fixed(WrfDimensionName::DateStringLength, 19),
            WrfDimension::fixed(WrfDimensionName::WestEast, grid.west_east()),
            WrfDimension::fixed(WrfDimensionName::SouthNorth, grid.south_north()),
            WrfDimension::fixed(WrfDimensionName::BottomTop, grid.bottom_top()),
            WrfDimension::fixed(
                WrfDimensionName::WestEastStaggered,
                grid.west_east_staggered(),
            ),
            WrfDimension::fixed(
                WrfDimensionName::SouthNorthStaggered,
                grid.south_north_staggered(),
            ),
            WrfDimension::fixed(
                WrfDimensionName::BottomTopStaggered,
                grid.bottom_top_staggered(),
            ),
        ];

        let mut attributes = vec![
            WrfAttribute::new(
                "TITLE",
                WrfAttributeValue::Text(" OUTPUT FROM WRF V4.7.1 MODEL".to_owned()),
            ),
            WrfAttribute::new(
                "START_DATE",
                WrfAttributeValue::Text(start_date.to_string()),
            ),
            WrfAttribute::new(
                "SIMULATION_START_DATE",
                WrfAttributeValue::Text(simulation_start_date.to_string()),
            ),
            WrfAttribute::new(
                "WEST-EAST_GRID_DIMENSION",
                WrfAttributeValue::Int32(vec![Self::to_i32(
                    "west_east",
                    grid.west_east_staggered(),
                )?]),
            ),
            WrfAttribute::new(
                "SOUTH-NORTH_GRID_DIMENSION",
                WrfAttributeValue::Int32(vec![Self::to_i32(
                    "south_north",
                    grid.south_north_staggered(),
                )?]),
            ),
            WrfAttribute::new(
                "BOTTOM-TOP_GRID_DIMENSION",
                WrfAttributeValue::Int32(vec![Self::to_i32(
                    "bottom_top",
                    grid.bottom_top_staggered(),
                )?]),
            ),
            WrfAttribute::new(
                "DX",
                WrfAttributeValue::Float32(vec![west_east_spacing_meters]),
            ),
            WrfAttribute::new(
                "DY",
                WrfAttributeValue::Float32(vec![south_north_spacing_meters]),
            ),
            WrfAttribute::new("GRIDTYPE", WrfAttributeValue::Text("C".to_owned())),
        ];
        if file_kind.requires_restart_flag() {
            attributes.push(WrfAttribute::new(
                "FLAG_RESTART",
                WrfAttributeValue::Int32(vec![1]),
            ));
        }

        let variables = Self::minimal_arw_variables()?;
        Self::try_from_parts(file_kind, dimensions, attributes, variables)
    }

    pub(crate) fn try_from_parts(
        file_kind: WrfFileKind,
        dimensions: Vec<WrfDimension>,
        attributes: Vec<WrfAttribute>,
        variables: Vec<WrfVariableSchema>,
    ) -> WrfIoResult<Self> {
        let mut names = HashSet::with_capacity(variables.len());
        for variable in &variables {
            if !names.insert(variable.name().clone()) {
                return Err(WrfIoError::DuplicateVariable {
                    variable: variable.name().clone(),
                });
            }
        }

        Ok(Self {
            file_kind,
            dimensions,
            attributes,
            variables,
        })
    }

    /// Returns the initialization or restart role.
    pub const fn file_kind(&self) -> WrfFileKind {
        self.file_kind
    }

    /// Returns dimensions in file-definition order.
    pub fn dimensions(&self) -> &[WrfDimension] {
        &self.dimensions
    }

    /// Returns global attributes in file-definition order.
    pub fn attributes(&self) -> &[WrfAttribute] {
        &self.attributes
    }

    /// Returns variables in file-definition order.
    pub fn variables(&self) -> &[WrfVariableSchema] {
        &self.variables
    }

    /// Finds one variable schema by typed name.
    pub fn variable(&self, name: &WrfVariableName) -> Option<&WrfVariableSchema> {
        self.variables
            .iter()
            .find(|variable| variable.name() == name)
    }

    /// Calculates a variable's checked element count from its dimensions.
    pub fn variable_element_count(&self, variable: &WrfVariableSchema) -> WrfIoResult<usize> {
        variable
            .dimensions()
            .iter()
            .try_fold(1_usize, |count, name| {
                let length = self
                    .dimensions
                    .iter()
                    .find(|dimension| dimension.name() == *name)
                    .map(WrfDimension::length)
                    .ok_or_else(|| WrfIoError::UnsupportedDimension {
                        name: name.as_str().to_owned(),
                    })?;
                count
                    .checked_mul(length)
                    .ok_or_else(|| WrfIoError::ElementCountOverflow {
                        variable: variable.name().clone(),
                    })
            })
    }

    fn validate_spacing(axis: &'static str, value: f32) -> WrfIoResult<()> {
        if value.is_finite() && value > 0.0 {
            return Ok(());
        }
        Err(WrfIoError::InvalidGridSpacing { axis, value })
    }

    fn to_i32(name: &'static str, length: usize) -> WrfIoResult<i32> {
        i32::try_from(length).map_err(|_| WrfIoError::DimensionLengthOverflow { name, length })
    }

    fn minimal_arw_variables() -> WrfIoResult<Vec<WrfVariableSchema>> {
        use WrfDimensionName::{
            BottomTop, BottomTopStaggered, DateStringLength, SouthNorth, SouthNorthStaggered, Time,
            WestEast, WestEastStaggered,
        };

        let mass_3d = vec![Time, BottomTop, SouthNorth, WestEast];
        let vertical_staggered = vec![Time, BottomTopStaggered, SouthNorth, WestEast];
        let surface = vec![Time, SouthNorth, WestEast];

        Ok(vec![
            WrfVariableSchema::try_new(
                "Times",
                WrfDataType::Character,
                vec![Time, DateStringLength],
                Vec::new(),
            )?,
            WrfVariableSchema::arw_float(
                "U",
                vec![Time, BottomTop, SouthNorth, WestEastStaggered],
                "XYZ",
                "x-wind component",
                "m s-1",
                "X",
            )?,
            WrfVariableSchema::arw_float(
                "V",
                vec![Time, BottomTop, SouthNorthStaggered, WestEast],
                "XYZ",
                "y-wind component",
                "m s-1",
                "Y",
            )?,
            WrfVariableSchema::arw_float(
                "W",
                vertical_staggered.clone(),
                "XYZ",
                "z-wind component",
                "m s-1",
                "Z",
            )?,
            WrfVariableSchema::arw_float(
                "PH",
                vertical_staggered.clone(),
                "XYZ",
                "perturbation geopotential",
                "m2 s-2",
                "Z",
            )?,
            WrfVariableSchema::arw_float(
                "PHB",
                vertical_staggered,
                "XYZ",
                "base-state geopotential",
                "m2 s-2",
                "Z",
            )?,
            WrfVariableSchema::arw_float(
                "THM",
                mass_3d.clone(),
                "XYZ",
                "either 1) pert moist pot temp=(1+Rv/Rd Qv)*(theta)-T0, or 2) pert dry pot temp=theta-T0; based on use_theta_m setting",
                "K",
                "",
            )?,
            WrfVariableSchema::arw_float(
                "MU",
                surface.clone(),
                "XY ",
                "perturbation dry air mass in column",
                "Pa",
                "",
            )?,
            WrfVariableSchema::arw_float(
                "MUB",
                surface,
                "XY ",
                "base state dry air mass in column",
                "Pa",
                "",
            )?,
            WrfVariableSchema::arw_float(
                "P",
                mass_3d.clone(),
                "XYZ",
                "perturbation pressure",
                "Pa",
                "",
            )?,
            WrfVariableSchema::arw_float(
                "PB",
                mass_3d.clone(),
                "XYZ",
                "BASE STATE PRESSURE ",
                "Pa",
                "",
            )?,
            WrfVariableSchema::arw_float(
                "QVAPOR",
                mass_3d,
                "XYZ",
                "Water vapor mixing ratio",
                "kg kg-1",
                "",
            )?,
            WrfVariableSchema::arw_float(
                "XTIME",
                vec![Time],
                "0  ",
                "minutes since YYYY-MM-DD hh:mm:ss",
                "minutes",
                "",
            )?,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_restart_schema_has_wrf_dimensions_metadata_and_restart_flag() {
        let schema = WrfFileSchema::try_minimal_arw(
            WrfFileKind::Restart,
            WrfGridDimensions::try_new(4, 3, 2).unwrap(),
            WrfTimestamp::try_new("2000-09-18_16:42:01").unwrap(),
            WrfTimestamp::try_new("2000-09-18_16:00:00").unwrap(),
            12_000.0,
            12_000.0,
        )
        .unwrap();

        assert_eq!(schema.dimensions().len(), 8);
        assert_eq!(schema.variables().len(), 13);
        assert!(schema.attributes().iter().any(|attribute| {
            attribute.name() == "FLAG_RESTART"
                && attribute.value() == &WrfAttributeValue::Int32(vec![1])
        }));
        assert_eq!(
            schema
                .variable(&WrfVariableName::try_new("U").unwrap())
                .unwrap()
                .dimensions(),
            &[
                WrfDimensionName::Time,
                WrfDimensionName::BottomTop,
                WrfDimensionName::SouthNorth,
                WrfDimensionName::WestEastStaggered,
            ]
        );
    }

    #[test]
    fn initialization_schema_omits_the_restart_flag() {
        let schema = WrfFileSchema::try_minimal_arw(
            WrfFileKind::Initialization,
            WrfGridDimensions::try_new(4, 3, 2).unwrap(),
            WrfTimestamp::try_new("2000-09-18_16:42:01").unwrap(),
            WrfTimestamp::try_new("2000-09-18_16:42:01").unwrap(),
            12_000.0,
            12_000.0,
        )
        .unwrap();

        assert!(
            schema
                .attributes()
                .iter()
                .all(|attribute| attribute.name() != "FLAG_RESTART")
        );
    }
}
