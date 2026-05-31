//! minilogue SysEx function codes — the byte that identifies a message.
//!
//! Device-verified codes (`docs/sysex-notes.md` §3). There is no Roland-style
//! address tree: the function code *is* the message identity.

/// A known minilogue SysEx function code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Function {
    /// `0x10` host→synth — request the edit buffer.
    CurrentProgramDumpRequest,
    /// `0x1C` host→synth — request a stored slot (+ 2-byte program number).
    ProgramDumpRequest,
    /// `0x0E` host→synth — request the global area.
    GlobalDumpRequest,
    /// `0x40` synth→host — the edit buffer (512 packed bytes → 448 unpacked).
    CurrentProgramDump,
    /// `0x4C` synth→host — a stored slot (2-byte program number + 512 packed).
    ProgramDump,
    /// `0x51` synth→host — the global area (110 packed bytes → 96 unpacked).
    GlobalDump,
    /// `0x23` synth→host — DATA LOAD COMPLETED (ACK).
    DataLoadCompleted,
    /// `0x24` synth→host — DATA LOAD ERROR (NAK).
    DataLoadError,
    /// `0x26` synth→host — DATA FORMAT ERROR.
    DataFormatError,
}

impl Function {
    /// The on-wire function byte.
    pub const fn code(self) -> u8 {
        match self {
            Function::CurrentProgramDumpRequest => 0x10,
            Function::ProgramDumpRequest => 0x1C,
            Function::GlobalDumpRequest => 0x0E,
            Function::CurrentProgramDump => 0x40,
            Function::ProgramDump => 0x4C,
            Function::GlobalDump => 0x51,
            Function::DataLoadCompleted => 0x23,
            Function::DataLoadError => 0x24,
            Function::DataFormatError => 0x26,
        }
    }

    /// Classify a function byte, or `None` if unrecognised.
    pub const fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            0x10 => Function::CurrentProgramDumpRequest,
            0x1C => Function::ProgramDumpRequest,
            0x0E => Function::GlobalDumpRequest,
            0x40 => Function::CurrentProgramDump,
            0x4C => Function::ProgramDump,
            0x51 => Function::GlobalDump,
            0x23 => Function::DataLoadCompleted,
            0x24 => Function::DataLoadError,
            0x26 => Function::DataFormatError,
            _ => return None,
        })
    }
}
