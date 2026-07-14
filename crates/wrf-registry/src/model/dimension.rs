use crate::SourceLocation;

/// How a Registry dimension obtains its inclusive coordinate bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DimensionLength {
    /// Bounds are supplied by the model domain.
    StandardDomain,
    /// Bounds are supplied by one or two integer namelist entries.
    Namelist {
        /// Namelist variable or literal providing the inclusive start.
        start: String,
        /// Namelist variable providing the inclusive end.
        end: String,
    },
    /// Bounds are compile-time integer constants.
    Constant {
        /// Inclusive constant start.
        start: i32,
        /// Inclusive constant end.
        end: i32,
    },
}

/// Coordinate axis assigned to a Registry dimension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoordinateAxis {
    /// West-east coordinate.
    X,
    /// South-north coordinate.
    Y,
    /// Bottom-top coordinate.
    Z,
    /// A non-spatial coordinate, represented by `c` in the Registry DSL.
    Constant,
}

/// A parsed `dimspec` entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DimensionSpecification {
    pub(crate) location: SourceLocation,
    pub(crate) name: String,
    pub(crate) order: Option<u8>,
    pub(crate) length: DimensionLength,
    pub(crate) axis: CoordinateAxis,
    pub(crate) data_name: Option<String>,
}

impl DimensionSpecification {
    /// Returns the beginning of the logical `dimspec` entry.
    #[must_use]
    pub const fn location(&self) -> &SourceLocation {
        &self.location
    }

    /// Returns the Registry dimension symbol.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional one-based model dimension order.
    #[must_use]
    pub const fn order(&self) -> Option<u8> {
        self.order
    }

    /// Returns how the dimension's inclusive bounds are determined.
    #[must_use]
    pub const fn length(&self) -> &DimensionLength {
        &self.length
    }

    /// Returns the coordinate-axis classification.
    #[must_use]
    pub const fn axis(&self) -> CoordinateAxis {
        self.axis
    }

    /// Returns the optional external data name.
    #[must_use]
    pub fn data_name(&self) -> Option<&str> {
        self.data_name.as_deref()
    }
}
