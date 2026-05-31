//! Typed model of the minilogue **PROGRAM** area (448 unpacked bytes, header
//! `"PROG"`). Offsets, the 10-bit `(b2_9<<2)|low2` packing, and enum bit
//! positions are from Korg's MIDI Implementation cross-referenced with
//! `gazzar/loguetools`, and verified byte-exact against the program fixtures
//! (the LFO low-bit positions were confirmed on-device). See
//! `../../docs/sysex-notes.md` §5.
//!
//! The 20-byte-per-step sequencer event payload is not yet sub-decoded; it is
//! preserved verbatim (`Step::event`) so the program round-trips byte-exact.

use serde::{Deserialize, Serialize};

use crate::codec::CodecError;
use crate::global::byte_enum;

/// Length of the unpacked program area.
pub const PROGRAM_LEN: usize = 448;
const PROG: &[u8; 4] = b"PROG";
const SEQD: &[u8; 4] = b"SEQD";
const NAME: core::ops::Range<usize> = 4..16;

byte_enum! {
    /// Oscillator footage. 0=16', 1=8', 2=4', 3=2'.
    Octave { Sixteen = 0, Eight = 1, Four = 2, Two = 3 }
    valid = "0=16', 1=8', 2=4', 3=2'"
}
byte_enum! {
    /// Oscillator waveform.
    Wave { Square = 0, Triangle = 1, Sawtooth = 2 }
    valid = "0=sqr, 1=tri, 2=saw"
}
byte_enum! {
    /// Filter slope.
    FilterType { TwoPole = 0, FourPole = 1 }
    valid = "0=2-pole, 1=4-pole"
}
byte_enum! {
    /// Cutoff velocity / keyboard-track amount (0%, 50%, 100%).
    CutoffAmount { Off = 0, Half = 1, Full = 2 }
    valid = "0=0%, 1=50%, 2=100%"
}
byte_enum! {
    /// LFO modulation target.
    LfoTarget { Cutoff = 0, Shape = 1, Pitch = 2 }
    valid = "0=cutoff, 1=shape, 2=pitch"
}
byte_enum! {
    /// What the LFO's EG modulates.
    LfoEgMod { Off = 0, Rate = 1, Int = 2 }
    valid = "0=off, 1=rate, 2=int"
}
byte_enum! {
    /// LFO waveform.
    LfoWave { Square = 0, Triangle = 1, Sawtooth = 2 }
    valid = "0=sqr, 1=tri, 2=saw"
}
byte_enum! {
    /// Delay output routing.
    DelayRouting { Bypass = 0, PreFilter = 1, PostFilter = 2 }
    valid = "0=bypass, 1=pre_filter, 2=post_filter"
}
byte_enum! {
    /// Voice allocation mode.
    VoiceMode { Poly = 0, Duo = 1, Unison = 2, Mono = 3, Chord = 4, Delay = 5, Arp = 6, Sidechain = 7 }
    valid = "0..=7"
}
byte_enum! {
    /// Sequencer step resolution.
    StepResolution { Sixteenth = 0, Eighth = 1, Quarter = 2, Half = 3, Whole = 4 }
    valid = "0=1/16, 1=1/8, 2=1/4, 3=1/2, 4=1/1"
}

macro_rules! prog_struct {
    ($(#[$m:meta])* $name:ident { $($(#[$fm:meta])* $f:ident : $t:ty),+ $(,)? }) => {
        $(#[$m])*
        #[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
        #[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct $name { $($(#[$fm])* pub $f : $t),+ }
    };
}

prog_struct! {
    /// A VCO: 10-bit pitch and shape, plus octave and waveform.
    Oscillator { pitch: u16, shape: u16, octave: Octave, wave: Wave }
}
prog_struct! {
    /// Source mixer levels (10-bit each).
    Mixer { vco1: u16, vco2: u16, noise: u16 }
}
prog_struct! {
    /// Filter section.
    Filter { cutoff: u16, resonance: u16, eg_int: u16, velocity: CutoffAmount, keyboard_track: CutoffAmount, pole: FilterType }
}
prog_struct! {
    /// An envelope generator (10-bit A/D/S/R).
    Eg { attack: u16, decay: u16, sustain: u16, release: u16 }
}
prog_struct! {
    /// LFO section.
    Lfo { rate: u16, int: u16, target: LfoTarget, eg_mod: LfoEgMod, wave: LfoWave }
}
prog_struct! {
    /// Delay effect.
    Delay { hi_pass_cutoff: u16, time: u16, feedback: u16, output_routing: DelayRouting }
}
prog_struct! {
    /// One of four motion-recording lanes.
    MotionSlot { param0: u8, param1: u8, step_bitmap: u16 }
}
prog_struct! {
    /// One sequencer step. `event` is the raw 20-byte payload (not yet decoded).
    Step { on: bool, motion_on: bool, event: Vec<u8> }
}
prog_struct! {
    /// 16-step note + motion sequencer.
    Sequencer {
        /// Tempo × 10 (1000 = 100.0 BPM), range 100..=3000.
        bpm_tenths: u16,
        step_length: u8,
        swing: i8,
        default_gate_time: u8,
        step_resolution: StepResolution,
        motion_slots: Vec<MotionSlot>,
        steps: Vec<Step>,
    }
}
prog_struct! {
    /// A complete minilogue program.
    Program {
        name: String,
        vco1: Oscillator,
        vco2: Oscillator,
        cross_mod_depth: u16,
        vco2_pitch_eg_int: u16,
        sync: bool,
        ring: bool,
        mixer: Mixer,
        filter: Filter,
        amp_eg: Eg,
        mod_eg: Eg,
        amp_velocity: u8,
        lfo: Lfo,
        delay: Delay,
        voice_mode: VoiceMode,
        voice_mode_depth: u16,
        portamento_time: u8,
        /// Pitch-bend range, ±1..=12 semitones (raw encoding preserved).
        bend_range: u8,
        /// Program output level, 77..=127 = −25..+25 dB.
        program_level: u8,
        /// Slider assignment target, 0..=28 (see Korg doc; named enum TODO).
        slider_assign: u8,
        /// Keyboard octave (offset 73), raw. Low 3 bits = 0..=4 (−2..+2);
        /// upper bits carry flags, so the byte is preserved verbatim.
        keyboard_octave: u8,
        /// LFO/portamento flag byte (offset 69), preserved raw pending decode.
        lfo_portamento_flags: u8,
        sequencer: Sequencer,
        /// Opaque device bytes (reserved regions + a few undocumented high bits
        /// of packed bytes) preserved verbatim for byte-exact round-trip. Not
        /// meant to be hand-edited.
        reserved: Vec<u8>,
    }
}

/// Reserved whole-byte offsets, preserved verbatim.
const RESERVED_OFFSETS: &[usize] = &[
    16, 17, 18, 19, 32, 44, 45, 46, 47, 48, 63, 65, 67, 68, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83,
    84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 102, 107,
];
/// Packed bytes whose masked bits are undocumented; preserved via the mask.
const PACKED_REMAINDERS: &[(usize, u8)] = &[(56, 0x80), (60, 0x3C), (62, 0x03), (64, 0xC8)];

#[inline]
fn lo2(b: u8, shift: u8) -> u8 {
    (b >> shift) & 0x03
}
#[inline]
fn ten(upper: u8, low2: u8) -> u16 {
    ((upper as u16) << 2) | (low2 as u16)
}
#[inline]
fn u16le(lo: u8, hi: u8) -> u16 {
    (lo as u16) | ((hi as u16) << 8)
}
#[inline]
fn split10(v: u16) -> (u8, u8) {
    ((v >> 2) as u8, (v & 0x03) as u8)
}

impl Program {
    /// Decode the 448-byte unpacked program payload.
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() != PROGRAM_LEN {
            return Err(CodecError::WrongLength {
                expected: PROGRAM_LEN,
                actual: b.len(),
            });
        }
        if &b[0..4] != PROG {
            return Err(CodecError::BadMarker { marker: "PROG" });
        }
        if &b[96..100] != SEQD {
            return Err(CodecError::BadMarker { marker: "SEQD" });
        }

        let name = String::from_utf8_lossy(&b[NAME])
            .trim_end_matches('\0')
            .to_string();

        let steps_on = u16le(b[108], b[109]);
        let steps_motion = u16le(b[110], b[111]);
        let mut motion_slots = Vec::with_capacity(4);
        for i in 0..4 {
            motion_slots.push(MotionSlot {
                param0: b[112 + i * 2],
                param1: b[113 + i * 2],
                step_bitmap: u16le(b[120 + i * 2], b[121 + i * 2]),
            });
        }
        let mut steps = Vec::with_capacity(16);
        for i in 0..16 {
            let off = 128 + i * 20;
            steps.push(Step {
                on: steps_on & (1 << i) != 0,
                motion_on: steps_motion & (1 << i) != 0,
                event: b[off..off + 20].to_vec(),
            });
        }

        Ok(Self {
            name,
            vco1: Oscillator {
                pitch: ten(b[20], lo2(b[52], 0)),
                shape: ten(b[21], lo2(b[52], 2)),
                octave: Octave::from_byte(lo2(b[52], 4))?,
                wave: Wave::from_byte(lo2(b[52], 6))?,
            },
            vco2: Oscillator {
                pitch: ten(b[22], lo2(b[53], 0)),
                shape: ten(b[23], lo2(b[53], 2)),
                octave: Octave::from_byte(lo2(b[53], 4))?,
                wave: Wave::from_byte(lo2(b[53], 6))?,
            },
            cross_mod_depth: ten(b[24], lo2(b[54], 0)),
            vco2_pitch_eg_int: ten(b[25], lo2(b[54], 2)),
            sync: b[55] & 0x01 != 0,
            ring: b[55] & 0x02 != 0,
            mixer: Mixer {
                vco1: ten(b[26], lo2(b[54], 4)),
                vco2: ten(b[27], lo2(b[54], 6)),
                noise: ten(b[28], lo2(b[55], 2)),
            },
            filter: Filter {
                cutoff: ten(b[29], lo2(b[55], 4)),
                resonance: ten(b[30], lo2(b[55], 6)),
                eg_int: ten(b[31], lo2(b[56], 0)),
                velocity: CutoffAmount::from_byte(lo2(b[56], 2))?,
                keyboard_track: CutoffAmount::from_byte(lo2(b[56], 4))?,
                pole: FilterType::from_byte((b[56] >> 6) & 0x01)?,
            },
            amp_eg: Eg {
                attack: ten(b[34], lo2(b[57], 0)),
                decay: ten(b[35], lo2(b[57], 2)),
                sustain: ten(b[36], lo2(b[57], 4)),
                release: ten(b[37], lo2(b[57], 6)),
            },
            mod_eg: Eg {
                attack: ten(b[38], lo2(b[58], 0)),
                decay: ten(b[39], lo2(b[58], 2)),
                sustain: ten(b[40], lo2(b[58], 4)),
                release: ten(b[41], lo2(b[58], 6)),
            },
            amp_velocity: b[33],
            lfo: Lfo {
                rate: ten(b[42], lo2(b[59], 0)),
                int: ten(b[43], lo2(b[59], 2)),
                target: LfoTarget::from_byte(lo2(b[59], 4))?,
                eg_mod: LfoEgMod::from_byte(lo2(b[59], 6))?,
                wave: LfoWave::from_byte(lo2(b[60], 0))?,
            },
            delay: Delay {
                hi_pass_cutoff: ten(b[49], lo2(b[62], 2)),
                time: ten(b[50], lo2(b[62], 4)),
                feedback: ten(b[51], lo2(b[62], 6)),
                output_routing: DelayRouting::from_byte(lo2(b[60], 6))?,
            },
            voice_mode: VoiceMode::from_byte(b[64] & 0x07)?,
            voice_mode_depth: ten(b[70], lo2(b[64], 4)),
            portamento_time: b[61],
            bend_range: b[66],
            program_level: b[71],
            slider_assign: b[72],
            keyboard_octave: b[73],
            lfo_portamento_flags: b[69],
            sequencer: Sequencer {
                bpm_tenths: u16le(b[100], b[101]),
                step_length: b[103],
                swing: b[104] as i8,
                default_gate_time: b[105],
                step_resolution: StepResolution::from_byte(b[106])?,
                motion_slots,
                steps,
            },
            reserved: RESERVED_OFFSETS
                .iter()
                .map(|&o| b[o])
                .chain(PACKED_REMAINDERS.iter().map(|&(o, m)| b[o] & m))
                .collect(),
        })
    }

    /// Encode back to the 448-byte unpacked payload (byte-exact with `from_bytes`).
    pub fn to_bytes(&self) -> Result<[u8; PROGRAM_LEN], CodecError> {
        let mut b = [0u8; PROGRAM_LEN];
        b[0..4].copy_from_slice(PROG);
        b[96..100].copy_from_slice(SEQD);

        let name = self.name.as_bytes();
        if name.len() > 12 {
            return Err(CodecError::OutOfRange {
                field: "name",
                value: name.len() as i32,
                valid: "<= 12 bytes",
            });
        }
        b[4..4 + name.len()].copy_from_slice(name);

        // 10-bit uppers + the shared low-bit / categorical bytes.
        let (u, l) = split10(self.vco1.pitch);
        b[20] = u;
        b[52] |= l;
        let (u, s) = split10(self.vco1.shape);
        b[21] = u;
        b[52] |= s << 2;
        b[52] |= self.vco1.octave.to_byte() << 4;
        b[52] |= self.vco1.wave.to_byte() << 6;

        let (u, l) = split10(self.vco2.pitch);
        b[22] = u;
        b[53] |= l;
        let (u, s) = split10(self.vco2.shape);
        b[23] = u;
        b[53] |= s << 2;
        b[53] |= self.vco2.octave.to_byte() << 4;
        b[53] |= self.vco2.wave.to_byte() << 6;

        let (u, l) = split10(self.cross_mod_depth);
        b[24] = u;
        b[54] |= l;
        let (u, l) = split10(self.vco2_pitch_eg_int);
        b[25] = u;
        b[54] |= l << 2;
        let (u, l) = split10(self.mixer.vco1);
        b[26] = u;
        b[54] |= l << 4;
        let (u, l) = split10(self.mixer.vco2);
        b[27] = u;
        b[54] |= l << 6;

        b[55] |= self.sync as u8;
        b[55] |= (self.ring as u8) << 1;
        let (u, l) = split10(self.mixer.noise);
        b[28] = u;
        b[55] |= l << 2;
        let (u, l) = split10(self.filter.cutoff);
        b[29] = u;
        b[55] |= l << 4;
        let (u, l) = split10(self.filter.resonance);
        b[30] = u;
        b[55] |= l << 6;

        let (u, l) = split10(self.filter.eg_int);
        b[31] = u;
        b[56] |= l;
        b[56] |= self.filter.velocity.to_byte() << 2;
        b[56] |= self.filter.keyboard_track.to_byte() << 4;
        b[56] |= self.filter.pole.to_byte() << 6;

        b[33] = self.amp_velocity;
        write_eg(&mut b, &self.amp_eg, [34, 35, 36, 37], 57);
        write_eg(&mut b, &self.mod_eg, [38, 39, 40, 41], 58);

        let (u, l) = split10(self.lfo.rate);
        b[42] = u;
        b[59] |= l;
        let (u, l) = split10(self.lfo.int);
        b[43] = u;
        b[59] |= l << 2;
        b[59] |= self.lfo.target.to_byte() << 4;
        b[59] |= self.lfo.eg_mod.to_byte() << 6;
        b[60] |= self.lfo.wave.to_byte();
        b[60] |= self.delay.output_routing.to_byte() << 6;

        let (u, l) = split10(self.delay.hi_pass_cutoff);
        b[49] = u;
        b[62] |= l << 2;
        let (u, l) = split10(self.delay.time);
        b[50] = u;
        b[62] |= l << 4;
        let (u, l) = split10(self.delay.feedback);
        b[51] = u;
        b[62] |= l << 6;

        b[64] |= self.voice_mode.to_byte();
        let (u, l) = split10(self.voice_mode_depth);
        b[70] = u;
        b[64] |= l << 4;

        b[61] = self.portamento_time;
        b[66] = self.bend_range;
        b[71] = self.program_level;
        b[72] = self.slider_assign;
        b[69] = self.lfo_portamento_flags;
        b[73] = self.keyboard_octave;

        // Sequencer.
        let seq = &self.sequencer;
        b[100] = (seq.bpm_tenths & 0xFF) as u8;
        b[101] = (seq.bpm_tenths >> 8) as u8;
        b[103] = seq.step_length;
        b[104] = seq.swing as u8;
        b[105] = seq.default_gate_time;
        b[106] = seq.step_resolution.to_byte();

        if seq.steps.len() != 16 {
            return Err(CodecError::WrongLength {
                expected: 16,
                actual: seq.steps.len(),
            });
        }
        if seq.motion_slots.len() != 4 {
            return Err(CodecError::WrongLength {
                expected: 4,
                actual: seq.motion_slots.len(),
            });
        }
        let mut steps_on = 0u16;
        let mut steps_motion = 0u16;
        for (i, step) in seq.steps.iter().enumerate() {
            if step.on {
                steps_on |= 1 << i;
            }
            if step.motion_on {
                steps_motion |= 1 << i;
            }
            if step.event.len() != 20 {
                return Err(CodecError::WrongLength {
                    expected: 20,
                    actual: step.event.len(),
                });
            }
            let off = 128 + i * 20;
            b[off..off + 20].copy_from_slice(&step.event);
        }
        b[108] = (steps_on & 0xFF) as u8;
        b[109] = (steps_on >> 8) as u8;
        b[110] = (steps_motion & 0xFF) as u8;
        b[111] = (steps_motion >> 8) as u8;
        for (i, slot) in seq.motion_slots.iter().enumerate() {
            b[112 + i * 2] = slot.param0;
            b[113 + i * 2] = slot.param1;
            b[120 + i * 2] = (slot.step_bitmap & 0xFF) as u8;
            b[121 + i * 2] = (slot.step_bitmap >> 8) as u8;
        }

        // Replay preserved opaque bytes: reserved whole bytes, then the masked
        // undocumented bits OR'd onto their packed bytes.
        let expected = RESERVED_OFFSETS.len() + PACKED_REMAINDERS.len();
        if self.reserved.len() != expected {
            return Err(CodecError::WrongLength {
                expected,
                actual: self.reserved.len(),
            });
        }
        for (&off, &val) in RESERVED_OFFSETS.iter().zip(&self.reserved) {
            b[off] = val;
        }
        for (i, &(off, _)) in PACKED_REMAINDERS.iter().enumerate() {
            b[off] |= self.reserved[RESERVED_OFFSETS.len() + i];
        }

        Ok(b)
    }
}

fn write_eg(b: &mut [u8], eg: &Eg, uppers: [usize; 4], low_byte: usize) {
    for (i, (&off, v)) in uppers
        .iter()
        .zip([eg.attack, eg.decay, eg.sustain, eg.release])
        .enumerate()
    {
        let (u, l) = split10(v);
        b[off] = u;
        b[low_byte] |= l << (i * 2);
    }
}
