//! Typed program: byte-exact + YAML round-trip against every program fixture.

use std::path::Path;

use minilogue_core::pack::unpack;
use minilogue_core::program::{VoiceMode, Wave};
use minilogue_core::yaml::{program_from_yaml_str, program_to_yaml_string};
use minilogue_core::{Frame, Function, Program};

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Unpacked 448-byte payload of a program fixture (handles the 0x4C 2-byte
/// program-number prefix vs the 0x40 edit-buffer form).
fn program_payload(name: &str) -> Vec<u8> {
    let frame = Frame::decode(&fixture(name)).unwrap();
    let start = if frame.function == Function::ProgramDump.code() {
        2
    } else {
        0
    };
    unpack(&frame.data[start..])
}

const PROGRAM_FIXTURES: &[&str] = &[
    "current_program.syx",
    "current_with_sequence.syx",
    "program_000.syx",
    "program_049.syx",
];

#[test]
fn programs_round_trip_byte_exact() {
    for name in PROGRAM_FIXTURES {
        let payload = program_payload(name);
        let prog = Program::from_bytes(&payload).unwrap_or_else(|e| panic!("{name}: {e}"));
        let out = prog.to_bytes().unwrap();
        let diffs: Vec<String> = payload
            .iter()
            .zip(out.iter())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (a, b))| format!("  off {i} (0x{i:02X}): {a:02X} -> {b:02X}"))
            .collect();
        assert!(
            diffs.is_empty(),
            "{name}: {} byte(s) differ:\n{}",
            diffs.len(),
            diffs.join("\n")
        );
    }
}

#[test]
fn program_yaml_round_trips() {
    for name in PROGRAM_FIXTURES {
        let prog = Program::from_bytes(&program_payload(name)).unwrap();
        let yaml = program_to_yaml_string(&prog).unwrap();
        let back = program_from_yaml_str(&yaml).unwrap();
        assert_eq!(back, prog, "{name}: YAML round-trip");
    }
}

#[test]
fn program_decodes_names_and_sequence() {
    let prog = Program::from_bytes(&program_payload("program_000.syx")).unwrap();
    assert_eq!(prog.name, "PolyLogue");
    assert_eq!(prog.sequencer.steps.len(), 16);
    assert_eq!(prog.sequencer.motion_slots.len(), 4);

    // The on-device sequence fixture: 16-step, steps 1/3/4/5 active.
    let seq = Program::from_bytes(&program_payload("current_with_sequence.syx")).unwrap();
    assert_eq!(seq.sequencer.bpm_tenths, 1200); // 120.0 BPM
    assert_eq!(seq.sequencer.step_length, 16);
    let active: Vec<usize> = seq
        .sequencer
        .steps
        .iter()
        .enumerate()
        .filter(|(_, s)| s.on)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(active, vec![0, 2, 3, 4]);

    // Enum decode sanity.
    let _ = (Wave::Sawtooth, VoiceMode::Poly);
}
