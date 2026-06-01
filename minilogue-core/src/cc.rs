//! Live-editing MIDI Control Change (CC) bindings.
//!
//! The original minilogue has no NRPN — every editable parameter that the
//! synth accepts live arrives as a single 7-bit CC on the global MIDI
//! channel. CCs travel on the **KBD/KNOB** port (port 1 of the USB pair);
//! bulk SysEx (program dumps, library writes) travels on the **SOUND** port
//! (port 2). This module owns the path → CC mapping verified against the
//! published "minilogue MIDI Implementation" document (KORG, 2016-07-08).
//!
//! Editors call [`program_cc_message`] with a parameter path from the
//! [`crate::params`] catalog and a value of the matching kind; they get back
//! either the three CC bytes ready to send on the performance port, or
//! `None` for paths the synth doesn't accept live (sequencer, name, etc.).

use serde::{Deserialize, Serialize};

/// A value of any catalog kind, untagged so editors can pass JS numbers,
/// strings, or booleans straight in (and receive them back) via
/// `serde_wasm_bindgen`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CcValue {
    /// Catches JS booleans first (untagged-enum order matters).
    Bool(bool),
    Number(f64),
    Token(String),
}

/// Per-path CC binding metadata. Distinct kinds because the synth's CC
/// values for choices/bools aren't a linear scaling of the catalog tokens.
#[derive(Debug, Clone, Copy)]
enum Mapping {
    /// Catalog range [min..=max] scales linearly to CC value 0..=127.
    Range { cc: u8, min: i32, max: i32 },
    /// Catalog choice token → exact CC byte. Tokens match the snake_case
    /// serde encoding of the corresponding enum in [`crate::program`].
    Choice {
        cc: u8,
        options: &'static [(&'static str, u8)],
    },
    /// Boolean: false → 0, true → 127.
    Bool { cc: u8 },
}

impl Mapping {
    fn encode(&self, value: &CcValue) -> Option<u8> {
        match (self, value) {
            (Mapping::Range { min, max, .. }, CcValue::Number(v)) => {
                let span = (*max - *min).max(1) as f64;
                let n = (((v - *min as f64) / span) * 127.0).round();
                Some(n.clamp(0.0, 127.0) as u8)
            }
            (Mapping::Choice { options, .. }, CcValue::Token(t)) => options
                .iter()
                .find_map(|(tok, byte)| (*tok == t.as_str()).then_some(*byte)),
            (Mapping::Bool { .. }, CcValue::Bool(b)) => Some(if *b { 127 } else { 0 }),
            _ => None,
        }
    }
    fn cc(&self) -> u8 {
        match self {
            Mapping::Range { cc, .. } | Mapping::Choice { cc, .. } | Mapping::Bool { cc } => *cc,
        }
    }
}

// --- shared choice byte tables (Korg's documented discrete values) ---

const OCTAVE_BYTES: &[(&str, u8)] = &[("sixteen", 0), ("eight", 42), ("four", 84), ("two", 127)];
const WAVE_BYTES: &[(&str, u8)] = &[("square", 0), ("triangle", 64), ("sawtooth", 127)];
const LFO_TARGET_BYTES: &[(&str, u8)] = &[("cutoff", 0), ("shape", 64), ("pitch", 127)];
const LFO_EG_BYTES: &[(&str, u8)] = &[("off", 0), ("rate", 64), ("int", 127)];
const AMOUNT_BYTES: &[(&str, u8)] = &[("off", 0), ("half", 64), ("full", 127)];
const POLE_BYTES: &[(&str, u8)] = &[("two_pole", 0), ("four_pole", 127)];
/// CC 88 byte values per the Korg doc — note the wire order is
/// bypass / post-filter / pre-filter, NOT the enum's declaration order.
const DELAY_ROUTING_BYTES: &[(&str, u8)] =
    &[("bypass", 0), ("post_filter", 64), ("pre_filter", 127)];

/// Program-parameter CC table, in CC-number order (matches the published
/// MIDI Implementation document so spot-checks against the spec are easy).
const PROGRAM_CC: &[(&str, Mapping)] = &[
    // Envelopes — Amp EG
    (
        "amp_eg.attack",
        Mapping::Range {
            cc: 16,
            min: 0,
            max: 1023,
        },
    ),
    (
        "amp_eg.decay",
        Mapping::Range {
            cc: 17,
            min: 0,
            max: 1023,
        },
    ),
    (
        "amp_eg.sustain",
        Mapping::Range {
            cc: 18,
            min: 0,
            max: 1023,
        },
    ),
    (
        "amp_eg.release",
        Mapping::Range {
            cc: 19,
            min: 0,
            max: 1023,
        },
    ),
    // Mod EG
    (
        "mod_eg.attack",
        Mapping::Range {
            cc: 20,
            min: 0,
            max: 1023,
        },
    ),
    (
        "mod_eg.decay",
        Mapping::Range {
            cc: 21,
            min: 0,
            max: 1023,
        },
    ),
    (
        "mod_eg.sustain",
        Mapping::Range {
            cc: 22,
            min: 0,
            max: 1023,
        },
    ),
    (
        "mod_eg.release",
        Mapping::Range {
            cc: 23,
            min: 0,
            max: 1023,
        },
    ),
    // LFO
    (
        "lfo.rate",
        Mapping::Range {
            cc: 24,
            min: 0,
            max: 1023,
        },
    ),
    (
        "lfo.int",
        Mapping::Range {
            cc: 26,
            min: 0,
            max: 1023,
        },
    ),
    // Voice
    (
        "voice_mode_depth",
        Mapping::Range {
            cc: 27,
            min: 0,
            max: 1023,
        },
    ),
    // Delay
    (
        "delay.hi_pass_cutoff",
        Mapping::Range {
            cc: 29,
            min: 0,
            max: 1023,
        },
    ),
    (
        "delay.time",
        Mapping::Range {
            cc: 30,
            min: 0,
            max: 1023,
        },
    ),
    (
        "delay.feedback",
        Mapping::Range {
            cc: 31,
            min: 0,
            max: 1023,
        },
    ),
    // Mixer & VCOs
    (
        "mixer.noise",
        Mapping::Range {
            cc: 33,
            min: 0,
            max: 1023,
        },
    ),
    (
        "vco1.pitch",
        Mapping::Range {
            cc: 34,
            min: 0,
            max: 1023,
        },
    ),
    (
        "vco2.pitch",
        Mapping::Range {
            cc: 35,
            min: 0,
            max: 1023,
        },
    ),
    (
        "vco1.shape",
        Mapping::Range {
            cc: 36,
            min: 0,
            max: 1023,
        },
    ),
    (
        "vco2.shape",
        Mapping::Range {
            cc: 37,
            min: 0,
            max: 1023,
        },
    ),
    (
        "mixer.vco1",
        Mapping::Range {
            cc: 39,
            min: 0,
            max: 1023,
        },
    ),
    (
        "mixer.vco2",
        Mapping::Range {
            cc: 40,
            min: 0,
            max: 1023,
        },
    ),
    (
        "cross_mod_depth",
        Mapping::Range {
            cc: 41,
            min: 0,
            max: 1023,
        },
    ),
    (
        "vco2_pitch_eg_int",
        Mapping::Range {
            cc: 42,
            min: 0,
            max: 1023,
        },
    ),
    // Filter
    (
        "filter.cutoff",
        Mapping::Range {
            cc: 43,
            min: 0,
            max: 1023,
        },
    ),
    (
        "filter.resonance",
        Mapping::Range {
            cc: 44,
            min: 0,
            max: 1023,
        },
    ),
    (
        "filter.eg_int",
        Mapping::Range {
            cc: 45,
            min: 0,
            max: 1023,
        },
    ),
    // Octaves and waves
    (
        "vco1.octave",
        Mapping::Choice {
            cc: 48,
            options: OCTAVE_BYTES,
        },
    ),
    (
        "vco2.octave",
        Mapping::Choice {
            cc: 49,
            options: OCTAVE_BYTES,
        },
    ),
    (
        "vco1.wave",
        Mapping::Choice {
            cc: 50,
            options: WAVE_BYTES,
        },
    ),
    (
        "vco2.wave",
        Mapping::Choice {
            cc: 51,
            options: WAVE_BYTES,
        },
    ),
    // LFO mode
    (
        "lfo.target",
        Mapping::Choice {
            cc: 56,
            options: LFO_TARGET_BYTES,
        },
    ),
    (
        "lfo.eg_mod",
        Mapping::Choice {
            cc: 57,
            options: LFO_EG_BYTES,
        },
    ),
    (
        "lfo.wave",
        Mapping::Choice {
            cc: 58,
            options: WAVE_BYTES,
        },
    ),
    // Sync / Ring
    ("sync", Mapping::Bool { cc: 80 }),
    ("ring", Mapping::Bool { cc: 81 }),
    // Filter tracking / type
    (
        "filter.velocity",
        Mapping::Choice {
            cc: 82,
            options: AMOUNT_BYTES,
        },
    ),
    (
        "filter.keyboard_track",
        Mapping::Choice {
            cc: 83,
            options: AMOUNT_BYTES,
        },
    ),
    (
        "filter.pole",
        Mapping::Choice {
            cc: 84,
            options: POLE_BYTES,
        },
    ),
    // Delay routing
    (
        "delay.output_routing",
        Mapping::Choice {
            cc: 88,
            options: DELAY_ROUTING_BYTES,
        },
    ),
];

fn lookup(path: &str) -> Option<&'static Mapping> {
    PROGRAM_CC
        .iter()
        .find_map(|(p, m)| (*p == path).then_some(m))
}

/// Build the three-byte CC frame for one program-parameter change, or
/// `None` if `path` has no live CC mapping (sequencer, name, etc.) or
/// `value` doesn't match the path's kind.
///
/// `channel` is the 1-based MIDI channel (1..=16); the wire status byte is
/// `0xB0 | (channel - 1)`.
pub fn program_cc_message(path: &str, value: &CcValue, channel: u8) -> Option<Vec<u8>> {
    let m = lookup(path)?;
    let byte = m.encode(value)?;
    let status = 0xB0 | (channel.saturating_sub(1) & 0x0F);
    Some(vec![status, m.cc(), byte])
}

/// True when the synth accepts live CC for this path. Useful for the UI
/// to badge controls that need a "Send to synth" full dump instead.
pub fn supports_live(path: &str) -> bool {
    lookup(path).is_some()
}

// ---------------------------------------------------------------------------
// System CCs / utility messages
// ---------------------------------------------------------------------------

fn status_for(channel: u8, base: u8) -> u8 {
    base | (channel.saturating_sub(1) & 0x0F)
}

/// Bank Select MSB + LSB + Program Change to switch the synth's active
/// patch on `channel` (1-based) to `slot` (0..=199). Per Korg's spec, MSB
/// is always 0; LSB toggles between 0 (slots 0..=99) and 1 (slots
/// 100..=199); PC carries the slot number modulo 100.
///
/// Returns the three frames in send-order. Callers can splice them straight
/// into the outbound performance queue.
pub fn program_change(slot: u16, channel: u8) -> Vec<Vec<u8>> {
    let bank_lsb = if slot >= 100 { 1 } else { 0 };
    let pc_num = (slot % 100) as u8;
    let cc_status = status_for(channel, 0xB0);
    let pc_status = status_for(channel, 0xC0);
    vec![
        vec![cc_status, 0x00, 0x00],     // Bank Select MSB (always 0)
        vec![cc_status, 0x20, bank_lsb], // Bank Select LSB
        vec![pc_status, pc_num],         // Program Change
    ]
}

/// "Panic" — All Sound Off (immediate) + All Notes Off (release-aware).
/// Send both because some hosts honour only one. Channel is 1-based.
pub fn panic_messages(channel: u8) -> Vec<Vec<u8>> {
    let cc = status_for(channel, 0xB0);
    vec![
        vec![cc, 0x78, 0x00], // All Sound Off
        vec![cc, 0x7B, 0x00], // All Notes Off
    ]
}

/// One inbound CC decoded back into a catalog path + value. The channel is
/// 1-based (recovered from the status byte's low nibble + 1).
#[derive(Debug, Clone, Serialize)]
pub struct InboundCc {
    pub path: String,
    pub value: CcValue,
    pub channel: u8,
}

/// Decode a 3-byte CC frame from the synth back into a catalog path +
/// value, or `None` if it isn't a CC for a known program parameter (or
/// has an unrecognized value for a choice).
///
/// Use this to mirror hardware-knob movements into the editor: subscribe
/// to the synth's KBD/KNOB port output and feed each short MIDI frame
/// here; for `Some(InboundCc)`, apply the value to the local program
/// draft without echoing back.
pub fn decode_program_cc(bytes: &[u8]) -> Option<InboundCc> {
    if bytes.len() != 3 {
        return None;
    }
    let status = bytes[0];
    if (status & 0xF0) != 0xB0 {
        return None;
    }
    let cc = bytes[1];
    let raw = bytes[2];
    if raw > 127 {
        return None;
    }
    let channel = (status & 0x0F) + 1;

    let (path, mapping) = PROGRAM_CC.iter().find(|(_, m)| m.cc() == cc)?;
    let value = match mapping {
        Mapping::Range { min, max, .. } => {
            let span = (*max - *min) as f64;
            let v = ((raw as f64) / 127.0) * span + (*min as f64);
            CcValue::Number(v.round())
        }
        Mapping::Choice { options, .. } => {
            // Per Korg's RECEIVED-data spec (notes *5-1..*5-11), choice CCs
            // are binned into equal slices of 0..=127:
            //   N=4 (octave): 0..31 → 16', 32..63 → 8', 64..95 → 4', 96..127 → 2'
            //   N=3 (wave/etc.): 0..42 → opt0, 43..85 → opt1, 86..127 → opt2
            //   N=2 (sync/ring/pole): 0..63 → off, 64..127 → on
            // Formula: idx = (raw * N) / 128, capped at N-1. Matches the
            // spec exactly for N=2/3/4. Options are stored in spec order.
            let n = options.len();
            if n == 0 {
                return None;
            }
            let idx = ((raw as usize) * n / 128).min(n - 1);
            let (tok, _) = options.get(idx)?;
            CcValue::Token((*tok).to_string())
        }
        Mapping::Bool { .. } => CcValue::Bool(raw >= 64),
    };
    Some(InboundCc {
        path: (*path).to_string(),
        value,
        channel,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_scales_linearly_to_cc43() {
        let m = program_cc_message("filter.cutoff", &CcValue::Number(512.0), 1).unwrap();
        assert_eq!(m, vec![0xB0, 43, 64]);
        let m = program_cc_message("filter.cutoff", &CcValue::Number(0.0), 1).unwrap();
        assert_eq!(m, vec![0xB0, 43, 0]);
        let m = program_cc_message("filter.cutoff", &CcValue::Number(1023.0), 1).unwrap();
        assert_eq!(m, vec![0xB0, 43, 127]);
    }

    #[test]
    fn channel_offsets_status_byte() {
        let m = program_cc_message("filter.cutoff", &CcValue::Number(0.0), 10).unwrap();
        assert_eq!(m[0], 0xB9);
    }

    #[test]
    fn vco_octave_picks_documented_bytes() {
        for (token, expected) in [("sixteen", 0), ("eight", 42), ("four", 84), ("two", 127)] {
            let m = program_cc_message("vco1.octave", &CcValue::Token(token.into()), 1).unwrap();
            assert_eq!(m, vec![0xB0, 48, expected], "token {token}");
        }
    }

    #[test]
    fn sync_bool_maps_to_full_or_zero() {
        let on = program_cc_message("sync", &CcValue::Bool(true), 1).unwrap();
        assert_eq!(on, vec![0xB0, 80, 127]);
        let off = program_cc_message("sync", &CcValue::Bool(false), 1).unwrap();
        assert_eq!(off, vec![0xB0, 80, 0]);
    }

    #[test]
    fn unknown_path_returns_none() {
        assert!(program_cc_message("name", &CcValue::Token("foo".into()), 1).is_none());
        assert!(program_cc_message("sequencer.bpm_tenths", &CcValue::Number(1200.0), 1).is_none());
    }

    #[test]
    fn wrong_kind_returns_none() {
        // Numeric value on a choice path.
        assert!(program_cc_message("vco1.wave", &CcValue::Number(64.0), 1).is_none());
        // Token on a range path.
        assert!(program_cc_message("filter.cutoff", &CcValue::Token("foo".into()), 1).is_none());
    }

    #[test]
    fn cutoff_round_trips() {
        // Encode 768/1023 → CC byte; decode back → close to 768 within scaling.
        let out = program_cc_message("filter.cutoff", &CcValue::Number(768.0), 1).unwrap();
        let back = decode_program_cc(&out).unwrap();
        assert_eq!(back.path, "filter.cutoff");
        match back.value {
            CcValue::Number(n) => assert!((n - 768.0).abs() <= 8.0, "got {n}"),
            other => panic!("expected Number, got {other:?}"),
        }
        assert_eq!(back.channel, 1);
    }

    #[test]
    fn octave_round_trips_exactly() {
        for token in ["sixteen", "eight", "four", "two"] {
            let out = program_cc_message("vco1.octave", &CcValue::Token(token.into()), 1).unwrap();
            let back = decode_program_cc(&out).unwrap();
            assert_eq!(back.path, "vco1.octave");
            assert!(matches!(back.value, CcValue::Token(ref t) if t == token));
        }
    }

    #[test]
    fn bool_round_trips() {
        let on = program_cc_message("sync", &CcValue::Bool(true), 1).unwrap();
        let back = decode_program_cc(&on).unwrap();
        assert!(matches!(back.value, CcValue::Bool(true)));

        let off = program_cc_message("sync", &CcValue::Bool(false), 1).unwrap();
        let back = decode_program_cc(&off).unwrap();
        assert!(matches!(back.value, CcValue::Bool(false)));
    }

    #[test]
    fn decode_channel_is_one_based() {
        let mut frame = program_cc_message("filter.cutoff", &CcValue::Number(512.0), 10).unwrap();
        assert_eq!(frame[0], 0xB9);
        let back = decode_program_cc(&frame).unwrap();
        assert_eq!(back.channel, 10);
        frame[0] = 0xB0;
        let back = decode_program_cc(&frame).unwrap();
        assert_eq!(back.channel, 1);
    }

    #[test]
    fn decode_rejects_non_cc_status() {
        // Note On 0x90 — not a CC.
        assert!(decode_program_cc(&[0x90, 60, 100]).is_none());
        // Wrong length.
        assert!(decode_program_cc(&[0xB0, 43]).is_none());
        // Unknown CC number.
        assert!(decode_program_cc(&[0xB0, 7, 100]).is_none());
    }

    #[test]
    fn decode_choice_bins_intermediate_values() {
        // Per Korg's RECEIVED spec: CC48 (VCO Octave) bins 0..31 → 16',
        // 32..63 → 8', 64..95 → 4', 96..127 → 2'. A third-party DAW sending
        // CC48 value 50 should land on "eight" (the closest anchor 42), not
        // be silently dropped.
        let cases = [
            (0u8, "sixteen"),
            (31u8, "sixteen"),
            (42u8, "eight"),
            (50u8, "eight"),
            (84u8, "four"),
            (127u8, "two"),
        ];
        for (raw, expected) in cases {
            let back = decode_program_cc(&[0xB0, 48, raw]).unwrap();
            assert!(
                matches!(back.value, CcValue::Token(ref t) if t == expected),
                "raw {raw} → expected {expected}, got {:?}",
                back.value
            );
        }
    }

    #[test]
    fn program_change_slot_under_100() {
        let frames = program_change(5, 1);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0], vec![0xB0, 0x00, 0x00]); // Bank MSB
        assert_eq!(frames[1], vec![0xB0, 0x20, 0x00]); // Bank LSB = 0
        assert_eq!(frames[2], vec![0xC0, 5]); // PC
    }

    #[test]
    fn program_change_slot_over_99_flips_bank_lsb() {
        let frames = program_change(150, 1);
        assert_eq!(frames[1], vec![0xB0, 0x20, 0x01]); // Bank LSB = 1
        assert_eq!(frames[2], vec![0xC0, 50]); // PC = 150 - 100
    }

    #[test]
    fn program_change_respects_channel() {
        let frames = program_change(0, 10);
        assert_eq!(frames[0][0], 0xB9); // CC on ch 10
        assert_eq!(frames[2][0], 0xC9); // PC on ch 10
    }

    #[test]
    fn panic_messages_emits_all_sound_off_and_notes_off() {
        let frames = panic_messages(1);
        assert_eq!(frames, vec![vec![0xB0, 0x78, 0x00], vec![0xB0, 0x7B, 0x00]]);
    }

    #[test]
    fn every_program_cc_mapping_has_a_catalog_entry() {
        use crate::params::PROGRAM_PARAMS;
        for (path, _) in PROGRAM_CC {
            assert!(
                PROGRAM_PARAMS.iter().any(|m| m.path == *path),
                "{path} mapped to a CC but missing from PROGRAM_PARAMS"
            );
        }
    }
}
