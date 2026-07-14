//! Typed parsing and selected artifact generation for the WRF Registry DSL.
//!
//! The compatibility reference is the Registry parser and generators bundled
//! with WRF v4.7.1 under `tools/`. This crate supports dimension
//! specifications, state variables, runtime-configuration entries, and typed
//! package-selected scalar layouts, plus the `pre_parse` preprocessing layer:
//! `include` expansion and
//! `ifdef`/`ifndef`/`endif`/`define` conditional selection. It deliberately
//! keeps Registry code generation separate from future runtime domain
//! ownership.
//!
//! # Supported source
//!
//! [`RegistryParser`] accepts dependency-closed `dimspec`, `state`, `rconfig`,
//! and `package` entries, including WRF-compatible quotes, comments, case
//! folding, and backslash continuations. Package scalar groups are resolved in
//! source order with distinct definition, packed-WRF, and dense-Rust indices.
//! Entry locations refer to the first physical line of the file that actually
//! holds the entry, across nested includes. Unsupported Registry categories
//! return a typed [`RegistryParseError`] instead of being silently discarded.
//!
//! [`RegistryParser::parse_file`] preprocesses `include` and conditional
//! directives with [`RegistryPreprocessor`] before parsing; symbols come from
//! a [`RegistryDefinitions`] table equivalent to upstream `-D` flags.
//! Preprocessing failures return a typed [`RegistryPreprocessError`] carrying
//! the offending physical location plus the include chain that reached it.
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
mod preprocessor;
mod registry_source_error;
mod scalar_layout;
mod source_location;

pub use generated_state::{
    GeneratedRegistryArtifacts, RegistryArtifactGenerator, RegistryGenerationError,
    RegistryGenerationResult,
};
pub use model::{
    ConfigurationEntryCount, CoordinateAxis, DimensionLength, DimensionSpecification,
    PackageCondition, PackageVariableGroup, ProcessorOrientation, RegistryDocument, RegistryEntry,
    RegistryPackage, RegistryValueType, RuntimeConfiguration, RuntimeConfigurationChoice,
    StateDimensions, StateStaggering, StateVariable,
};
pub use parser::{RegistryParseError, RegistryParseErrorKind, RegistryParser, RegistryResult};
pub use preprocessor::{
    ConditionalDirective, FileSystemSourceProvider, PreprocessedRegistrySource,
    RegistryDefinitions, RegistryPreprocessError, RegistryPreprocessErrorKind,
    RegistryPreprocessResult, RegistryPreprocessor, RegistrySourceProvider,
};
pub use registry_source_error::RegistrySourceError;
pub use scalar_layout::{
    DefinitionParameterIndex, RegistryResolutionError, RegistryResolutionErrorKind,
    RegistryResolutionResult, ResolvedScalarArrayLayout, ResolvedScalarArrayMember,
    RustDenseScalarIndex, WrfPackedScalarIndex,
};
pub use source_location::SourceLocation;
