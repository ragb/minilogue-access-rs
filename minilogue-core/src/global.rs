//! Typed model of the minilogue **GLOBAL** area (96 unpacked bytes, header
//! `"GLOB"`). Offsets and value names are from Korg's MIDI Implementation and
//! verified byte-exact against `tests/fixtures/global.syx`.
//!
//! `from_bytes`/`to_bytes` operate on the 96-byte **unpacked** payload (i.e.
//! after [`crate::pack::unpack`] of a `0x51` frame's data).

use serde::{Deserialize, Serialize};

use crate::codec::CodecError;

/// Length of the unpacked global area.
pub const GLOBAL_LEN: usize = 96;
const MARKER: &[u8; 4] = b"GLOB";

macro_rules! byte_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $value:expr),+ $(,)? } valid = $valid:expr) => {
        $(#[$meta])*
        #[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
        #[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            const FIELD: &'static str = stringify!($name);
            fn from_byte(b: u8) -> Result<Self, CodecError> {
                match b {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(CodecError::InvalidValue { field: Self::FIELD, value: b, valid: $valid }),
                }
            }
            fn to_byte(self) -> u8 {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }
    };
}
pub(crate) use byte_enum;

byte_enum! {
    /// Velocity response curve (offset 6).
    VelocityCurve {
        Type1 = 0, Type2 = 1, Type3 = 2, Type4 = 3, Type5 = 4,
        Type6 = 5, Type7 = 6, Type8 = 7, Const127 = 8,
    }
    valid = "0..=8"
}

byte_enum! {
    /// Knob behaviour when the physical position differs from the stored value (offset 7).
    KnobMode { Jump = 0, Catch = 1, Scale = 2 }
    valid = "0=jump, 1=catch, 2=scale"
}

byte_enum! {
    /// Sequencer/arp clock source (offset 9).
    ClockSource { AutoUsb = 0, AutoMidi = 1, Internal = 2 }
    valid = "0=auto_usb, 1=auto_midi, 2=internal"
}

byte_enum! {
    /// Sync jack note unit (offsets 10 and 13).
    SyncUnit { Sixteenth = 0, Eighth = 1 }
    valid = "0=sixteenth, 1=eighth"
}

byte_enum! {
    /// Sync jack edge polarity (offsets 11 and 12).
    SyncPolarity { Rise = 0, Fall = 1 }
    valid = "0=rise, 1=fall"
}

byte_enum! {
    /// Where MIDI is routed (offset 16).
    MidiRoute { UsbAndMidi = 0, Usb = 1 }
    valid = "0=usb_and_midi, 1=usb"
}

byte_enum! {
    /// Parameter display mode (offset 26): stored 1=Normal, 2=All.
    ParameterDisplay { Normal = 1, All = 2 }
    valid = "1=normal, 2=all"
}

/// The minilogue global area.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalArea {
    /// Master tune, cents (−50..=50).
    pub master_tune: i8,
    /// Transpose, semitones (−12..=12).
    pub transpose: i8,
    pub velocity_curve: VelocityCurve,
    pub knob_mode: KnobMode,
    pub audio_in: bool,
    pub clock_source: ClockSource,
    pub sync_in_unit: SyncUnit,
    pub sync_out_polarity: SyncPolarity,
    pub sync_in_polarity: SyncPolarity,
    pub sync_out_unit: SyncUnit,
    pub midi_route: MidiRoute,
    /// MIDI channel, 1..=16 (stored on the wire as 0..=15).
    pub midi_channel: u8,
    pub local_sw: bool,
    pub rx_short_enabled: bool,
    pub tx_short_enabled: bool,
    /// Display brightness, 1..=10 (stored on the wire as 0..=9).
    pub brightness: u8,
    pub auto_power_off: bool,
    pub parameter_display: ParameterDisplay,
    pub oscilloscope: bool,
    /// Bytes 64..=79 of the global area — 16 program-number-like values
    /// (0..=199 if interpreted that way).
    ///
    /// Korg's MIDI Implementation Chart documents only the first **8** as
    /// favourite-slot bookmarks (bytes 64..=71, mapped to the 8 FAV
    /// buttons). Bytes 72..=79 hold an additional 8 program-number-shaped
    /// bytes in observed fixtures — could be reserved slots, a second-tier
    /// favourites bank, or unrelated state. Their device-side meaning is
    /// **not verified** against any documented spec or button behaviour.
    ///
    /// All 16 are stored here so the GlobalArea round-trips byte-exact;
    /// the editor exposes only the documented 8.
    pub favorites: Vec<u8>,
}

fn bool_byte(b: u8, field: &'static str) -> Result<bool, CodecError> {
    match b {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(CodecError::InvalidValue {
            field,
            value: b,
            valid: "0=off, 1=on",
        }),
    }
}

fn signed(b: u8, field: &'static str, lo: i32, hi: i32) -> Result<i8, CodecError> {
    let v = b as i8 as i32;
    if v < lo || v > hi {
        return Err(CodecError::OutOfRange {
            field,
            value: v,
            valid: "see field range",
        });
    }
    Ok(v as i8)
}

impl GlobalArea {
    /// Decode the 96-byte unpacked global payload.
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() != GLOBAL_LEN {
            return Err(CodecError::WrongLength {
                expected: GLOBAL_LEN,
                actual: b.len(),
            });
        }
        if &b[0..4] != MARKER {
            return Err(CodecError::BadMarker { marker: "GLOB" });
        }
        Ok(Self {
            master_tune: signed(b[4], "master_tune", -50, 50)?,
            transpose: signed(b[5], "transpose", -12, 12)?,
            velocity_curve: VelocityCurve::from_byte(b[6])?,
            knob_mode: KnobMode::from_byte(b[7])?,
            audio_in: bool_byte(b[8], "audio_in")?,
            clock_source: ClockSource::from_byte(b[9])?,
            sync_in_unit: SyncUnit::from_byte(b[10])?,
            sync_out_polarity: SyncPolarity::from_byte(b[11])?,
            sync_in_polarity: SyncPolarity::from_byte(b[12])?,
            sync_out_unit: SyncUnit::from_byte(b[13])?,
            midi_route: MidiRoute::from_byte(b[16])?,
            midi_channel: checked_add1(b[17], "midi_channel", 0, 15)?,
            local_sw: bool_byte(b[18], "local_sw")?,
            rx_short_enabled: bool_byte(b[19], "rx_short_enabled")?,
            tx_short_enabled: bool_byte(b[20], "tx_short_enabled")?,
            brightness: checked_add1(b[24], "brightness", 0, 9)?,
            auto_power_off: bool_byte(b[25], "auto_power_off")?,
            parameter_display: ParameterDisplay::from_byte(b[26])?,
            oscilloscope: bool_byte(b[27], "oscilloscope")?,
            favorites: b[64..80].to_vec(),
        })
    }

    /// Encode back to the 96-byte unpacked payload (byte-exact with `from_bytes`).
    pub fn to_bytes(&self) -> Result<[u8; GLOBAL_LEN], CodecError> {
        let mut b = [0u8; GLOBAL_LEN];
        b[0..4].copy_from_slice(MARKER);
        b[4] = self.master_tune as u8;
        b[5] = self.transpose as u8;
        b[6] = self.velocity_curve.to_byte();
        b[7] = self.knob_mode.to_byte();
        b[8] = self.audio_in as u8;
        b[9] = self.clock_source.to_byte();
        b[10] = self.sync_in_unit.to_byte();
        b[11] = self.sync_out_polarity.to_byte();
        b[12] = self.sync_in_polarity.to_byte();
        b[13] = self.sync_out_unit.to_byte();
        b[16] = self.midi_route.to_byte();
        b[17] = checked_sub1(self.midi_channel, "midi_channel", 1, 16)?;
        b[18] = self.local_sw as u8;
        b[19] = self.rx_short_enabled as u8;
        b[20] = self.tx_short_enabled as u8;
        b[24] = checked_sub1(self.brightness, "brightness", 1, 10)?;
        b[25] = self.auto_power_off as u8;
        b[26] = self.parameter_display.to_byte();
        b[27] = self.oscilloscope as u8;
        if self.favorites.len() != 16 {
            return Err(CodecError::WrongLength {
                expected: 16,
                actual: self.favorites.len(),
            });
        }
        b[64..80].copy_from_slice(&self.favorites);
        Ok(b)
    }
}

/// Wire byte `lo..=hi` → 1-based value (byte + 1), validated.
fn checked_add1(b: u8, field: &'static str, lo: u8, hi: u8) -> Result<u8, CodecError> {
    if b < lo || b > hi {
        return Err(CodecError::OutOfRange {
            field,
            value: b as i32,
            valid: "wire range",
        });
    }
    Ok(b + 1)
}

/// 1-based value `lo..=hi` → wire byte (value − 1), validated.
fn checked_sub1(v: u8, field: &'static str, lo: u8, hi: u8) -> Result<u8, CodecError> {
    if v < lo || v > hi {
        return Err(CodecError::OutOfRange {
            field,
            value: v as i32,
            valid: "1-based range",
        });
    }
    Ok(v - 1)
}
