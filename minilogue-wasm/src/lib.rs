//! WASM / TypeScript bindings for `minilogue-core`.
//!
//! Exposes the full editor surface: decode/encode SysEx program & global dumps,
//! build dump-request frames, `.mnlgprog`/`.mnlglib` import/export, YAML
//! conversion, and the parameter metadata/help catalog. The typed [`Program`],
//! [`GlobalArea`], [`ProgInfo`] and [`MnlgProgram`] cross the boundary as
//! TypeScript objects via `tsify`. JS names are camelCase to match the editor's
//! other device codecs.

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use minilogue_core::pack::{pack, unpack};
use minilogue_core::{Frame, Function, GlobalArea, MnlgProgram, ProgInfo, Program};

fn js_err<E: std::fmt::Display>(e: E) -> JsError {
    JsError::new(&e.to_string())
}

/// Wire channel byte for a 1-based MIDI channel (channel 1 → 0x30).
fn channel_byte(channel: u8) -> u8 {
    0x30 | (channel.saturating_sub(1) & 0x0F)
}

/// 2-byte program number (pp = low 7 bits, PP = bit 7).
fn program_number(slot: u16) -> Vec<u8> {
    vec![(slot & 0x7F) as u8, ((slot >> 7) & 0x01) as u8]
}

fn frame(channel: u8, function: u8, data: Vec<u8>) -> Vec<u8> {
    Frame::new(channel_byte(channel), function, data).encode()
}

// --- SysEx: programs ---

/// Decode a CURRENT (`0x40`) or PROGRAM (`0x4C`) dump frame into a [`Program`].
#[wasm_bindgen(js_name = decodeProgramDump)]
pub fn decode_program_dump(bytes: &[u8]) -> Result<Program, JsError> {
    let f = Frame::decode(bytes).map_err(js_err)?;
    let payload = if f.function == Function::ProgramDump.code() {
        // 0x4C echoes a 2-byte program number before the packed payload.
        unpack(f.data.get(2..).unwrap_or_default())
    } else {
        unpack(&f.data)
    };
    Program::from_bytes(&payload).map_err(js_err)
}

/// Encode a program as a CURRENT PROGRAM dump (`0x40`) — loads the edit buffer.
#[wasm_bindgen(js_name = encodeCurrentProgram)]
pub fn encode_current_program(program: Program, channel: u8) -> Result<Vec<u8>, JsError> {
    let data = pack(&program.to_bytes().map_err(js_err)?);
    Ok(frame(channel, Function::CurrentProgramDump.code(), data))
}

/// Encode a program as a PROGRAM dump (`0x4C`) that writes to `slot` (0..=199).
#[wasm_bindgen(js_name = encodeProgramWrite)]
pub fn encode_program_write(program: Program, slot: u16, channel: u8) -> Result<Vec<u8>, JsError> {
    let mut data = program_number(slot);
    data.extend(pack(&program.to_bytes().map_err(js_err)?));
    Ok(frame(channel, Function::ProgramDump.code(), data))
}

/// CURRENT PROGRAM DUMP REQUEST frame (reply is a `0x40` dump).
#[wasm_bindgen(js_name = currentProgramRequest)]
pub fn current_program_request(channel: u8) -> Vec<u8> {
    frame(channel, Function::CurrentProgramDumpRequest.code(), vec![])
}

/// PROGRAM DUMP REQUEST frame for `slot` (reply is a `0x4C` dump).
#[wasm_bindgen(js_name = programRequest)]
pub fn program_request(slot: u16, channel: u8) -> Vec<u8> {
    frame(
        channel,
        Function::ProgramDumpRequest.code(),
        program_number(slot),
    )
}

// --- SysEx: global ---

/// Decode a GLOBAL dump frame (`0x51`) into a [`GlobalArea`].
#[wasm_bindgen(js_name = decodeGlobalDump)]
pub fn decode_global_dump(bytes: &[u8]) -> Result<GlobalArea, JsError> {
    let f = Frame::decode(bytes).map_err(js_err)?;
    GlobalArea::from_bytes(&unpack(&f.data)).map_err(js_err)
}

/// Encode a [`GlobalArea`] as a GLOBAL dump (`0x51`).
#[wasm_bindgen(js_name = encodeGlobal)]
pub fn encode_global(global: GlobalArea, channel: u8) -> Result<Vec<u8>, JsError> {
    let data = pack(&global.to_bytes().map_err(js_err)?);
    Ok(frame(channel, Function::GlobalDump.code(), data))
}

/// GLOBAL DATA DUMP REQUEST frame (reply is a `0x51` dump).
#[wasm_bindgen(js_name = globalRequest)]
pub fn global_request(channel: u8) -> Vec<u8> {
    frame(channel, Function::GlobalDumpRequest.code(), vec![])
}

// --- .mnlgprog / .mnlglib containers ---

/// A library: programs plus metadata. Crosses the boundary as
/// `{ programs: MnlgProgram[] }`.
#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Library {
    pub programs: Vec<MnlgProgram>,
}

/// Import a single-program `.mnlgprog`.
#[wasm_bindgen(js_name = importMnlgprog)]
pub fn import_mnlgprog(bytes: &[u8]) -> Result<MnlgProgram, JsError> {
    minilogue_core::read_mnlgprog(bytes).map_err(js_err)
}

/// Export a single program to `.mnlgprog` bytes.
#[wasm_bindgen(js_name = exportMnlgprog)]
pub fn export_mnlgprog(program: Program, info: ProgInfo) -> Result<Vec<u8>, JsError> {
    minilogue_core::write_mnlgprog(&program, &info).map_err(js_err)
}

/// Import a `.mnlgprog`/`.mnlglib` into a [`Library`].
#[wasm_bindgen(js_name = importLibrary)]
pub fn import_library(bytes: &[u8]) -> Result<Library, JsError> {
    let programs = minilogue_core::mnlg::read_library(bytes).map_err(js_err)?;
    Ok(Library { programs })
}

/// Export a [`Library`] to `.mnlglib` bytes.
#[wasm_bindgen(js_name = exportLibrary)]
pub fn export_library(library: Library) -> Result<Vec<u8>, JsError> {
    minilogue_core::mnlg::write_library(&library.programs).map_err(js_err)
}

// --- YAML ---

#[wasm_bindgen(js_name = programToYaml)]
pub fn program_to_yaml(program: Program) -> Result<String, JsError> {
    minilogue_core::yaml::program_to_yaml_string(&program).map_err(js_err)
}

#[wasm_bindgen(js_name = programFromYaml)]
pub fn program_from_yaml(yaml: &str) -> Result<Program, JsError> {
    minilogue_core::yaml::program_from_yaml_str(yaml).map_err(js_err)
}

#[wasm_bindgen(js_name = globalToYaml)]
pub fn global_to_yaml(global: GlobalArea) -> Result<String, JsError> {
    minilogue_core::yaml::global_to_yaml_string(&global).map_err(js_err)
}

#[wasm_bindgen(js_name = globalFromYaml)]
pub fn global_from_yaml(yaml: &str) -> Result<GlobalArea, JsError> {
    minilogue_core::yaml::global_from_yaml_str(yaml).map_err(js_err)
}

// --- parameter metadata / help (for the editor UI) ---

/// Tooltip / screen-reader help for a parameter path (e.g. `"filter.cutoff"`).
#[wasm_bindgen(js_name = helpFor)]
pub fn help_for(path: &str) -> Option<String> {
    minilogue_core::params::help_for(path).map(String::from)
}

/// Program parameter catalog: `[{ path, label, group, kind, help }]` — labels,
/// ranges/units, and enum option labels for building accessible controls.
#[wasm_bindgen(js_name = programParamCatalog)]
pub fn program_param_catalog() -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(minilogue_core::params::PROGRAM_PARAMS).map_err(js_err)
}

/// Global parameter catalog (same shape as [`program_param_catalog`]).
#[wasm_bindgen(js_name = globalParamCatalog)]
pub fn global_param_catalog() -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(minilogue_core::params::GLOBAL_PARAMS).map_err(js_err)
}

// --- constants ---

/// Korg model ID for the original minilogue (`0x2C`).
#[wasm_bindgen(js_name = modelId)]
pub fn model_id() -> u8 {
    minilogue_core::MODEL_ID
}
