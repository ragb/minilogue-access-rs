//! WASM/TypeScript bindings for `minilogue-core`.
//!
//! Thin re-export surface; populated with program/global types (via
//! `tsify-next`) once the core data model exists.

use wasm_bindgen::prelude::*;

/// Korg model ID for the original minilogue (`0x2C`).
#[wasm_bindgen]
pub fn model_id() -> u8 {
    minilogue_core::MODEL_ID
}
