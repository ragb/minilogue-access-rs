//! Shared codec error type for the typed program/global data model.

use thiserror::Error;

/// Errors from decoding/encoding typed structures over the unpacked byte areas.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CodecError {
    #[error("expected {expected} bytes, got {actual}")]
    WrongLength { expected: usize, actual: usize },

    #[error("bad {marker:?} marker at offset 0")]
    BadMarker { marker: &'static str },

    #[error("invalid {field} value {value:#04x} (valid: {valid})")]
    InvalidValue {
        field: &'static str,
        value: u8,
        valid: &'static str,
    },

    #[error("{field} out of range: {value} (valid: {valid})")]
    OutOfRange {
        field: &'static str,
        value: i32,
        valid: &'static str,
    },

    #[error("YAML: {0}")]
    Yaml(String),

    #[error("container: {0}")]
    Container(String),
}
