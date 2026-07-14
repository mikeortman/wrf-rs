//! Typed parsing and selected artifact generation for the WRF Registry DSL.
//!
//! The compatibility reference is the Registry parser and generators bundled
//! with WRF v4.7.1 under `tools/`. This first slice supports dimension
//! specifications, state variables, and runtime-configuration entries. It
//! deliberately keeps Registry code generation separate from future runtime
//! domain ownership.
//!
//! # Supported source
//!
//! [`RegistryParser`] accepts dependency-closed `dimspec`, `state`, and
//! `rconfig` entries, including WRF-compatible quotes, comments, case folding,
//! and backslash continuations. Entry locations refer to the first physical
//! line. Unsupported Registry categories return a typed [`RegistryParseError`]
//! instead of being silently discarded.
//!
//! # Example
//!
//! ```
//! use wrf_registry::{RegistryArtifactGenerator, RegistryParser};
//!
//! let source = "\
//! dimspec i 1 standard_domain x west_east
//! dimspec k 2 standard_domain z bottom_top
//! dimspec j 3 standard_domain y south_north
//! state real temperature ikj dyn_em 1 - irh \"T\" \"temperature\" \"K\"
//! rconfig logical restart namelist,time_control 1 .false. - \"RESTART\" \"restart flag\" \"flag\"
//! ";
//!
//! let document = RegistryParser::parse("Registry.example", source)?;
//! assert_eq!(document.state_variables().count(), 1);
//! assert_eq!(document.runtime_configurations().count(), 1);
//!
//! let generated = RegistryArtifactGenerator::generate(&document)?;
//! assert!(generated.state_struct().contains(":: temperature"));
//! assert!(generated.model_data_order().contains("DATA_ORDER_XZY"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![forbid(unsafe_code)]

mod generated_state;
mod model;
mod parser;
mod source_location;

pub use generated_state::{
    GeneratedRegistryArtifacts, RegistryArtifactGenerator, RegistryGenerationError,
    RegistryGenerationResult,
};
pub use model::{
    ConfigurationEntryCount, CoordinateAxis, DimensionLength, DimensionSpecification,
    ProcessorOrientation, RegistryDocument, RegistryEntry, RegistryValueType, RuntimeConfiguration,
    StateDimensions, StateStaggering, StateVariable,
};
pub use parser::{RegistryParseError, RegistryParseErrorKind, RegistryParser, RegistryResult};
pub use source_location::SourceLocation;
