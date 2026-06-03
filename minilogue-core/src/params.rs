//! Editor-facing parameter metadata: display labels, value ranges/units,
//! enum option labels, group, and tooltip help — keyed by the parameter's
//! path in the [`crate::Program`] / [`crate::GlobalArea`] serde structure
//! (e.g. `"vco1.pitch"`, `"filter.pole"`, `"voice_mode"`, `"midi_channel"`).
//!
//! This is the single source of truth an accessible editor uses to render
//! controls and announce them: the label is the accessible name, [`Kind`]
//! says whether it's a slider (with range/unit), a choice (with token→label
//! option pairs), a toggle, or text, and [`Meta::help`] is the tooltip /
//! screen-reader description. Help text is distilled from the minilogue
//! Owner's Manual (Program Architecture, pp. 12 & 15+).
//!
//! Option tokens match the snake_case serde encoding of the enums in
//! [`crate::program`] / [`crate::global`], so the editor can write them back
//! directly.

use serde::Serialize;

/// One selectable option: the on-the-wire token and its human label.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Choice {
    pub value: &'static str,
    pub label: &'static str,
}

const fn c(value: &'static str, label: &'static str) -> Choice {
    Choice { value, label }
}

/// The control kind for a parameter.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Kind {
    /// A numeric slider, inclusive `min..=max`, with an optional unit.
    Range {
        min: i32,
        max: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        unit: Option<&'static str>,
    },
    /// A choice from token→label options.
    Choice { options: &'static [Choice] },
    /// An on/off toggle.
    Bool,
    /// Free text up to `max_len` bytes.
    Text { max_len: usize },
}

/// Metadata for one editable parameter.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Meta {
    /// Path into the serde structure, e.g. `"vco1.pitch"`.
    pub path: &'static str,
    /// Accessible display name, e.g. `"VCO 1 Pitch"`.
    pub label: &'static str,
    /// Panel group, e.g. `"VCO 1"`, `"Filter"`, `"Sequencer"`.
    pub group: &'static str,
    pub kind: Kind,
    /// Tooltip / screen-reader help.
    pub help: &'static str,
    /// True if this is a bounded magnitude (a "level"): a slider where
    /// "50% of max" is a meaningful answer. The editor opts these sliders
    /// into the user's percentage display when their `levelDisplay`
    /// preference is set to percent. Centred/bipolar fields (where the
    /// displayed centre means "no effect") and discrete values are not levels.
    #[serde(default, skip_serializing_if = "is_false")]
    pub level: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

const fn range(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    min: i32,
    max: i32,
    unit: Option<&'static str>,
    help: &'static str,
) -> Meta {
    Meta {
        path,
        label,
        group,
        kind: Kind::Range { min, max, unit },
        help,
        level: false,
    }
}
/// Like [`range`] but flags the parameter as a level (bounded magnitude) so
/// the editor opts it into percentage display. See [`Meta::level`].
const fn level_range(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    min: i32,
    max: i32,
    unit: Option<&'static str>,
    help: &'static str,
) -> Meta {
    Meta {
        path,
        label,
        group,
        kind: Kind::Range { min, max, unit },
        help,
        level: true,
    }
}
const fn choice(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    options: &'static [Choice],
    help: &'static str,
) -> Meta {
    Meta {
        path,
        label,
        group,
        kind: Kind::Choice { options },
        help,
        level: false,
    }
}
const fn toggle(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    help: &'static str,
) -> Meta {
    Meta {
        path,
        label,
        group,
        kind: Kind::Bool,
        help,
        level: false,
    }
}

// --- shared option sets (tokens match the core enums' snake_case serde) ---

const OCTAVE: &[Choice] = &[
    c("sixteen", "16'"),
    c("eight", "8'"),
    c("four", "4'"),
    c("two", "2'"),
];
const WAVE: &[Choice] = &[
    c("square", "Square"),
    c("triangle", "Triangle"),
    c("sawtooth", "Sawtooth"),
];
const POLE: &[Choice] = &[
    c("two_pole", "2-pole (12 dB)"),
    c("four_pole", "4-pole (24 dB)"),
];
const AMOUNT: &[Choice] = &[c("off", "0%"), c("half", "50%"), c("full", "100%")];
const LFO_TARGET: &[Choice] = &[
    c("cutoff", "Cutoff"),
    c("shape", "Shape"),
    c("pitch", "Pitch"),
];
const LFO_EG: &[Choice] = &[c("off", "Off"), c("rate", "Rate"), c("int", "Int")];
const DELAY_ROUTING: &[Choice] = &[
    c("bypass", "Bypass"),
    c("pre_filter", "Pre-Filter"),
    c("post_filter", "Post-Filter"),
];
const VOICE_MODE: &[Choice] = &[
    c("poly", "Poly"),
    c("duo", "Duo"),
    c("unison", "Unison"),
    c("mono", "Mono"),
    c("chord", "Chord"),
    c("delay", "Delay"),
    c("arp", "Arp"),
    c("sidechain", "Sidechain"),
];
const STEP_RES: &[Choice] = &[
    c("sixteenth", "1/16"),
    c("eighth", "1/8"),
    c("quarter", "1/4"),
    c("half", "1/2"),
    c("whole", "1/1"),
];
const VELOCITY_CURVE: &[Choice] = &[
    c("type1", "Type 1"),
    c("type2", "Type 2"),
    c("type3", "Type 3"),
    c("type4", "Type 4"),
    c("type5", "Type 5"),
    c("type6", "Type 6"),
    c("type7", "Type 7"),
    c("type8", "Type 8"),
    c("const127", "Const 127"),
];
const KNOB_MODE: &[Choice] = &[c("jump", "Jump"), c("catch", "Catch"), c("scale", "Scale")];
const CLOCK_SOURCE: &[Choice] = &[
    c("auto_usb", "Auto (USB)"),
    c("auto_midi", "Auto (MIDI)"),
    c("internal", "Internal"),
];
const SYNC_UNIT: &[Choice] = &[c("sixteenth", "16th note"), c("eighth", "8th note")];
const SYNC_POLARITY: &[Choice] = &[c("rise", "Rising edge"), c("fall", "Falling edge")];
const MIDI_ROUTE: &[Choice] = &[c("usb_and_midi", "USB + MIDI"), c("usb", "USB only")];
const PARAM_DISPLAY: &[Choice] = &[c("normal", "Normal"), c("all", "All")];

const TEN_BIT: Option<&'static str> = None; // 0..1023 unitless

/// Program (patch) parameters, in panel order.
pub const PROGRAM_PARAMS: &[Meta] = &[
    Meta {
        path: "name",
        label: "Program Name",
        group: "Program",
        kind: Kind::Text { max_len: 12 },
        help: "The patch name, up to 12 characters.",
        level: false,
    },
    // VCO 1
    choice(
        "vco1.octave",
        "VCO 1 Octave",
        "VCO 1",
        OCTAVE,
        "Octave (footage) of oscillator 1: 16' (lowest) to 2' (highest).",
    ),
    choice(
        "vco1.wave",
        "VCO 1 Wave",
        "VCO 1",
        WAVE,
        "Waveform of oscillator 1.",
    ),
    range(
        "vco1.pitch",
        "VCO 1 Pitch",
        "VCO 1",
        0,
        1023,
        TEN_BIT,
        "Fine pitch of oscillator 1; centre (512) is no detune.",
    ),
    level_range(
        "vco1.shape",
        "VCO 1 Shape",
        "VCO 1",
        0,
        1023,
        TEN_BIT,
        "Wave shape of oscillator 1 (pulse width / shape morph).",
    ),
    // VCO 2
    choice(
        "vco2.octave",
        "VCO 2 Octave",
        "VCO 2",
        OCTAVE,
        "Octave (footage) of oscillator 2: 16' to 2'.",
    ),
    choice(
        "vco2.wave",
        "VCO 2 Wave",
        "VCO 2",
        WAVE,
        "Waveform of oscillator 2.",
    ),
    range(
        "vco2.pitch",
        "VCO 2 Pitch",
        "VCO 2",
        0,
        1023,
        TEN_BIT,
        "Fine pitch of oscillator 2; centre is no detune.",
    ),
    level_range(
        "vco2.shape",
        "VCO 2 Shape",
        "VCO 2",
        0,
        1023,
        TEN_BIT,
        "Wave shape of oscillator 2.",
    ),
    // VCO 2 modulation
    level_range(
        "cross_mod_depth",
        "Cross Mod Depth",
        "VCO 2 Modulation",
        0,
        1023,
        TEN_BIT,
        "Amount oscillator 2 frequency-modulates oscillator 1.",
    ),
    level_range(
        "vco2_pitch_eg_int",
        "VCO 2 Pitch EG Int",
        "VCO 2 Modulation",
        0,
        1023,
        TEN_BIT,
        "How much the mod EG sweeps oscillator 2 pitch.",
    ),
    toggle(
        "sync",
        "Oscillator Sync",
        "VCO 2 Modulation",
        "Hard-sync oscillator 2 to oscillator 1 for biting timbres.",
    ),
    toggle(
        "ring",
        "Ring Mod",
        "VCO 2 Modulation",
        "Ring-modulate the two oscillators for metallic tones.",
    ),
    // Mixer
    level_range(
        "mixer.vco1",
        "VCO 1 Level",
        "Mixer",
        0,
        1023,
        TEN_BIT,
        "Level of oscillator 1 into the filter.",
    ),
    level_range(
        "mixer.vco2",
        "VCO 2 Level",
        "Mixer",
        0,
        1023,
        TEN_BIT,
        "Level of oscillator 2 into the filter.",
    ),
    level_range(
        "mixer.noise",
        "Noise Level",
        "Mixer",
        0,
        1023,
        TEN_BIT,
        "Level of the noise generator into the filter.",
    ),
    // Filter
    level_range(
        "filter.cutoff",
        "Cutoff",
        "Filter",
        0,
        1023,
        TEN_BIT,
        "Low-pass filter cutoff frequency. Lower is darker.",
    ),
    level_range(
        "filter.resonance",
        "Resonance",
        "Filter",
        0,
        1023,
        TEN_BIT,
        "Emphasis at the cutoff. High values ring or self-oscillate.",
    ),
    range(
        "filter.eg_int",
        "Filter EG Int",
        "Filter",
        0,
        1023,
        TEN_BIT,
        "How much the mod EG sweeps the cutoff (bipolar around centre).",
    ),
    choice(
        "filter.pole",
        "Filter Type",
        "Filter",
        POLE,
        "Filter slope: 2-pole (12 dB/oct) is gentler, 4-pole (24 dB/oct) steeper.",
    ),
    choice(
        "filter.keyboard_track",
        "Key Track",
        "Filter",
        AMOUNT,
        "How much the played note raises the cutoff (keyboard tracking).",
    ),
    choice(
        "filter.velocity",
        "Velocity",
        "Filter",
        AMOUNT,
        "How much playing velocity raises the cutoff.",
    ),
    // Amp EG
    level_range(
        "amp_eg.attack",
        "Amp EG Attack",
        "Amp EG",
        0,
        1023,
        TEN_BIT,
        "Time for the volume to rise after a key is pressed.",
    ),
    level_range(
        "amp_eg.decay",
        "Amp EG Decay",
        "Amp EG",
        0,
        1023,
        TEN_BIT,
        "Time to fall from peak to the sustain level.",
    ),
    level_range(
        "amp_eg.sustain",
        "Amp EG Sustain",
        "Amp EG",
        0,
        1023,
        TEN_BIT,
        "Held volume level while a key stays down.",
    ),
    level_range(
        "amp_eg.release",
        "Amp EG Release",
        "Amp EG",
        0,
        1023,
        TEN_BIT,
        "Time for the volume to fade after the key is released.",
    ),
    // Mod EG
    level_range(
        "mod_eg.attack",
        "EG Attack",
        "EG",
        0,
        1023,
        TEN_BIT,
        "Attack time of the modulation envelope.",
    ),
    level_range(
        "mod_eg.decay",
        "EG Decay",
        "EG",
        0,
        1023,
        TEN_BIT,
        "Decay time of the modulation envelope.",
    ),
    level_range(
        "mod_eg.sustain",
        "EG Sustain",
        "EG",
        0,
        1023,
        TEN_BIT,
        "Sustain level of the modulation envelope.",
    ),
    level_range(
        "mod_eg.release",
        "EG Release",
        "EG",
        0,
        1023,
        TEN_BIT,
        "Release time of the modulation envelope.",
    ),
    // LFO
    choice("lfo.wave", "LFO Wave", "LFO", WAVE, "LFO waveform."),
    choice(
        "lfo.eg_mod",
        "LFO EG Mod",
        "LFO",
        LFO_EG,
        "Lets the mod EG modulate the LFO rate or intensity.",
    ),
    level_range(
        "lfo.rate",
        "LFO Rate",
        "LFO",
        0,
        1023,
        TEN_BIT,
        "LFO speed.",
    ),
    level_range(
        "lfo.int",
        "LFO Int",
        "LFO",
        0,
        1023,
        TEN_BIT,
        "LFO depth applied to the target.",
    ),
    choice(
        "lfo.target",
        "LFO Target",
        "LFO",
        LFO_TARGET,
        "What the LFO modulates: cutoff, wave shape, or pitch.",
    ),
    // Delay
    level_range(
        "delay.hi_pass_cutoff",
        "Delay Hi Pass Cutoff",
        "Delay",
        0,
        1023,
        TEN_BIT,
        "High-pass filter on the delay feedback; raises to thin the echoes.",
    ),
    level_range(
        "delay.time",
        "Delay Time",
        "Delay",
        0,
        1023,
        TEN_BIT,
        "Delay time.",
    ),
    level_range(
        "delay.feedback",
        "Delay Feedback",
        "Delay",
        0,
        1023,
        TEN_BIT,
        "How much delayed signal feeds back; higher means more repeats.",
    ),
    choice(
        "delay.output_routing",
        "Output Routing",
        "Delay",
        DELAY_ROUTING,
        "Where the delay sits: bypassed, before the filter, or after it.",
    ),
    // Voice
    choice(
        "voice_mode",
        "Voice Mode",
        "Voice",
        VOICE_MODE,
        "How the 4 voices are allocated (Poly, Duo, Unison, Mono, Chord, Delay, Arp, Sidechain).",
    ),
    range(
        "voice_mode_depth",
        "Voice Mode Depth",
        "Voice",
        0,
        1023,
        TEN_BIT,
        "Mode-specific amount (e.g. detune in Unison, chord type in Chord, arp pattern in Arp).",
    ),
    level_range(
        "amp_velocity",
        "Amp Velocity",
        "Amp EG",
        0,
        127,
        None,
        "How much playing velocity affects loudness.",
    ),
    level_range(
        "portamento_time",
        "Portamento Time",
        "Voice",
        0,
        127,
        None,
        "Glide time between notes. 0 is off.",
    ),
    level_range(
        "program_level",
        "Program Level",
        "Program",
        77,
        127,
        Some("77=-25dB..127=+25dB"),
        "Per-patch output level (raw 77..127 = -25..+25 dB).",
    ),
    // Sequencer
    range(
        "sequencer.bpm_tenths",
        "Tempo",
        "Sequencer",
        100,
        3000,
        Some("0.1 BPM"),
        "Sequencer tempo in tenths of a BPM (1200 = 120.0 BPM).",
    ),
    range(
        "sequencer.step_length",
        "Step Length",
        "Sequencer",
        1,
        16,
        None,
        "Number of active steps in the sequence.",
    ),
    range(
        "sequencer.swing",
        "Swing",
        "Sequencer",
        -75,
        75,
        Some("%"),
        "Shuffle feel; shifts off-beat steps later (+) or earlier (-).",
    ),
    range(
        "sequencer.default_gate_time",
        "Default Gate Time",
        "Sequencer",
        0,
        72,
        Some("0..72 = 0..100%"),
        "Default note length per step (0..72 maps to 0..100%).",
    ),
    choice(
        "sequencer.step_resolution",
        "Step Resolution",
        "Sequencer",
        STEP_RES,
        "Musical length of each step.",
    ),
];

/// Global (system) parameters.
pub const GLOBAL_PARAMS: &[Meta] = &[
    range(
        "master_tune",
        "Master Tune",
        "Tune",
        -50,
        50,
        Some("cents"),
        "Overall tuning of the whole instrument.",
    ),
    range(
        "transpose",
        "Transpose",
        "Tune",
        -12,
        12,
        Some("semitones"),
        "Transposes the keyboard in semitones.",
    ),
    choice(
        "velocity_curve",
        "Velocity Curve",
        "Keyboard",
        VELOCITY_CURVE,
        "How key velocity maps to level (Type 1-8, or fixed Const 127).",
    ),
    choice(
        "knob_mode",
        "Knob Mode",
        "General",
        KNOB_MODE,
        "How knobs respond when the physical position differs from the stored value.",
    ),
    toggle(
        "audio_in",
        "Audio In",
        "General",
        "Enable the rear audio input through the voice path.",
    ),
    choice(
        "clock_source",
        "Clock Source",
        "Sync",
        CLOCK_SOURCE,
        "Sequencer/arp clock source.",
    ),
    choice(
        "sync_in_unit",
        "Sync In Unit",
        "Sync",
        SYNC_UNIT,
        "Pulse value received at the Sync In jack.",
    ),
    choice(
        "sync_out_unit",
        "Sync Out Unit",
        "Sync",
        SYNC_UNIT,
        "Pulse value sent from the Sync Out jack.",
    ),
    choice(
        "sync_in_polarity",
        "Sync In Polarity",
        "Sync",
        SYNC_POLARITY,
        "Which edge of the Sync In pulse triggers.",
    ),
    choice(
        "sync_out_polarity",
        "Sync Out Polarity",
        "Sync",
        SYNC_POLARITY,
        "Edge polarity of the Sync Out pulse.",
    ),
    choice(
        "midi_route",
        "MIDI Route",
        "MIDI",
        MIDI_ROUTE,
        "Whether MIDI flows over USB+MIDI or USB only.",
    ),
    range(
        "midi_channel",
        "MIDI Channel",
        "MIDI",
        1,
        16,
        None,
        "Global MIDI channel.",
    ),
    toggle(
        "local_sw",
        "Local Control",
        "MIDI",
        "When off, the keyboard is disconnected from the sound engine (controls MIDI only).",
    ),
    toggle(
        "rx_short_enabled",
        "Rx Short Messages",
        "MIDI",
        "Receive short (non-SysEx) MIDI messages.",
    ),
    toggle(
        "tx_short_enabled",
        "Tx Short Messages",
        "MIDI",
        "Transmit short (non-SysEx) MIDI messages.",
    ),
    level_range(
        "brightness",
        "Brightness",
        "General",
        1,
        10,
        None,
        "LED display brightness.",
    ),
    toggle(
        "auto_power_off",
        "Auto Power Off",
        "General",
        "Automatically power down after inactivity.",
    ),
    choice(
        "parameter_display",
        "Parameter Display",
        "General",
        PARAM_DISPLAY,
        "Whether the display shows the normal set or all parameters.",
    ),
    toggle(
        "oscilloscope",
        "Oscilloscope",
        "General",
        "Show the real-time oscilloscope on the display.",
    ),
];

/// Tooltip help for a parameter path, searching program then global. `None`
/// if the path isn't a catalogued parameter.
pub fn help_for(path: &str) -> Option<&'static str> {
    param(path).map(|m| m.help)
}

/// The full metadata for a parameter path (program then global).
pub fn param(path: &str) -> Option<&'static Meta> {
    PROGRAM_PARAMS
        .iter()
        .chain(GLOBAL_PARAMS.iter())
        .find(|m| m.path == path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bounded magnitudes the editor should be able to render as a percentage.
    const KNOWN_LEVELS: &[&str] = &[
        "vco1.shape",
        "filter.cutoff",
        "filter.resonance",
        "amp_eg.attack",
        "amp_eg.decay",
        "amp_eg.sustain",
        "amp_eg.release",
        "lfo.rate",
        "lfo.int",
        "delay.time",
        "delay.feedback",
        "program_level",
    ];

    #[test]
    fn known_levels_are_flagged() {
        for path in KNOWN_LEVELS {
            let m = param(path).unwrap_or_else(|| panic!("missing catalog entry: {path}"));
            assert!(m.level, "{path} should be a level");
            assert!(
                matches!(m.kind, Kind::Range { .. }),
                "{path} level must be a Range"
            );
        }
    }

    #[test]
    fn vco_pitch_is_not_a_level() {
        // Centred/bipolar values (512 = no detune) are not magnitudes.
        for path in ["vco1.pitch", "vco2.pitch"] {
            assert!(!param(path).unwrap().level, "{path} must not be a level");
        }
    }

    #[test]
    fn non_range_kinds_are_never_levels() {
        // Choice / Bool / Text controls are discrete and can't be a percentage.
        for m in PROGRAM_PARAMS.iter().chain(GLOBAL_PARAMS.iter()) {
            if !matches!(m.kind, Kind::Range { .. }) {
                assert!(!m.level, "non-Range {} must not be a level", m.path);
            }
        }
    }

    #[test]
    fn level_is_omitted_when_false() {
        // `level: false` is the default and must not bloat the serialized catalog;
        // `level: true` must be emitted so the editor can read it.
        let plain = serde_yaml::to_value(param("vco1.pitch").unwrap()).unwrap();
        assert!(
            plain.get("level").is_none(),
            "non-level entries must omit the `level` key"
        );

        let leveled = serde_yaml::to_value(param("filter.cutoff").unwrap()).unwrap();
        assert_eq!(
            leveled.get("level"),
            Some(&serde_yaml::Value::Bool(true)),
            "level entries must serialize `level: true`"
        );
    }
}
