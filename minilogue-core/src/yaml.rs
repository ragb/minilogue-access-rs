//! YAML string codec for the typed areas. Pure (no file I/O — lives in core so
//! the wasm crate can use it). File I/O + schema headers live in the CLI.

use crate::codec::CodecError;
use crate::global::GlobalArea;
use crate::program::Program;

/// Editor schema hint written at the top of a global YAML file.
pub const GLOBAL_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/minilogue-global.schema.json";
/// Editor schema hint written at the top of a program YAML file.
pub const PROGRAM_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/minilogue-program.schema.json";

pub fn global_to_yaml_string(g: &GlobalArea) -> Result<String, CodecError> {
    serde_yaml::to_string(g).map_err(|e| CodecError::Yaml(e.to_string()))
}

pub fn global_from_yaml_str(s: &str) -> Result<GlobalArea, CodecError> {
    serde_yaml::from_str(s).map_err(|e| CodecError::Yaml(e.to_string()))
}

pub fn program_to_yaml_string(p: &Program) -> Result<String, CodecError> {
    serde_yaml::to_string(p).map_err(|e| CodecError::Yaml(e.to_string()))
}

pub fn program_from_yaml_str(s: &str) -> Result<Program, CodecError> {
    serde_yaml::from_str(s).map_err(|e| CodecError::Yaml(e.to_string()))
}
