#![forbid(unsafe_code)]

//! Korg minilogue SysEx + Sound Librarian (`.mnlgprog`/`.mnlglib`) codec.
//!
//! Pure: no MIDI, no file I/O. The framing ([`sysex`]), 7→8 packing ([`pack`]),
//! and function codes ([`function`]) are verified against real device captures.
//! [`global`] is the typed 96-byte global area; the typed 448-byte program model
//! is built next. See `../../docs/sysex-notes.md`.

pub mod codec;
pub mod function;
pub mod global;
pub mod mnlg;
pub mod pack;
pub mod params;
pub mod program;
#[cfg(feature = "schema")]
pub mod schema;
pub mod sysex;
pub mod yaml;

pub use codec::CodecError;
pub use function::Function;
pub use global::GlobalArea;
pub use mnlg::{read_mnlgprog, write_mnlgprog, MnlgProgram, ProgInfo};
pub use program::Program;
pub use sysex::{Frame, SysExError, KORG_ID, MODEL_ID};
