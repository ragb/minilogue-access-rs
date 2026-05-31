//! Korg minilogue SysEx frame: `F0 42 3g 00 01 2C <func> [data...] F7`.
//!
//! No checksum, no address tree. The byte after the model ID is the function
//! code; everything between it and `F7` is kept raw in [`Frame::data`] (still
//! 7→8 packed, and still including the 2-byte program number for `0x4C` program
//! dumps). Higher layers (`program`, `global`) unpack and interpret it.
//!
//! Framing device-verified 2026-05-31 (`docs/sysex-notes.md` §2).

use thiserror::Error;

pub const SYSEX_START: u8 = 0xF0;
pub const SYSEX_END: u8 = 0xF7;
/// Korg MIDI manufacturer ID.
pub const KORG_ID: u8 = 0x42;
/// Korg "extended" format header that precedes the model ID.
pub const FORMAT_HEADER: [u8; 2] = [0x00, 0x01];
/// Model ID for the original minilogue (one byte — defined once so sibling
/// models stay a future feature-flag).
pub const MODEL_ID: u8 = 0x2C;

/// Broadcast channel byte (ignored by this device for dump requests — it wants
/// the exact channel — but defined for completeness).
pub const CHANNEL_BROADCAST: u8 = 0x7F;

/// Smallest possible frame: `F0 42 3g 00 01 2C <func> F7`.
const MIN_FRAME_LEN: usize = 8;

/// A decoded minilogue SysEx frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Raw channel byte: `0x30 | (channel-1)` for channels 1–16, or `0x7F`.
    pub channel: u8,
    /// Function code (see [`crate::Function`]).
    pub function: u8,
    /// Raw bytes between the function byte and `F7` (still packed).
    pub data: Vec<u8>,
}

impl Frame {
    /// Build the channel byte for a 1-based MIDI channel (1–16).
    pub const fn channel_byte(channel: u8) -> u8 {
        0x30 | ((channel - 1) & 0x0F)
    }

    /// Create a frame from a function byte and raw (already-packed) data.
    pub fn new(channel: u8, function: u8, data: Vec<u8>) -> Self {
        Self {
            channel,
            function,
            data,
        }
    }

    /// Serialize to wire bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.data.len() + MIN_FRAME_LEN);
        out.push(SYSEX_START);
        out.push(KORG_ID);
        out.push(self.channel);
        out.extend_from_slice(&FORMAT_HEADER);
        out.push(MODEL_ID);
        out.push(self.function);
        out.extend_from_slice(&self.data);
        out.push(SYSEX_END);
        out
    }

    /// Parse wire bytes into a frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, SysExError> {
        if bytes.len() < MIN_FRAME_LEN {
            return Err(SysExError::TooShort(bytes.len()));
        }
        if bytes[0] != SYSEX_START {
            return Err(SysExError::MissingStart);
        }
        if *bytes.last().unwrap() != SYSEX_END {
            return Err(SysExError::MissingEnd);
        }
        if bytes[1] != KORG_ID {
            return Err(SysExError::NotKorg(bytes[1]));
        }
        if bytes[3] != FORMAT_HEADER[0] || bytes[4] != FORMAT_HEADER[1] {
            return Err(SysExError::BadFormatHeader([bytes[3], bytes[4]]));
        }
        if bytes[5] != MODEL_ID {
            return Err(SysExError::WrongModel(bytes[5]));
        }
        Ok(Self {
            channel: bytes[2],
            function: bytes[6],
            data: bytes[7..bytes.len() - 1].to_vec(),
        })
    }
}

/// Errors from [`Frame::decode`].
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SysExError {
    #[error("frame too short ({0} bytes, need at least {MIN_FRAME_LEN})")]
    TooShort(usize),
    #[error("missing F0 start sentinel")]
    MissingStart,
    #[error("missing F7 end sentinel")]
    MissingEnd,
    #[error("not a Korg frame (manufacturer id {0:#04x})")]
    NotKorg(u8),
    #[error("bad format header {0:02x?} (expected 00 01)")]
    BadFormatHeader([u8; 2]),
    #[error("wrong model id {0:#04x} (expected minilogue {MODEL_ID:#04x})")]
    WrongModel(u8),
}
