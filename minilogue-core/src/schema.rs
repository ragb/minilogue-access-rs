//! JSON Schema emitters (behind the `schema` feature). Used by the CLI's
//! `schema` subcommand and the CI drift check.

use schemars::schema::RootSchema;

use crate::global::GlobalArea;
use crate::program::Program;

pub fn global_area_schema() -> RootSchema {
    schemars::schema_for!(GlobalArea)
}

pub fn program_schema() -> RootSchema {
    schemars::schema_for!(Program)
}
