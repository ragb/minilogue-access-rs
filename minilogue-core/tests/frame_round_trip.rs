//! Byte-exact round-trip of real device captures, plus the unpack pipeline.

use std::path::Path;

use minilogue_core::pack::unpack;
use minilogue_core::{Frame, Function};

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

const FIXTURES: &[&str] = &[
    "current_program.syx",
    "global.syx",
    "program_000.syx",
    "program_049.syx",
];

#[test]
fn fixtures_round_trip_byte_exact() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let frame = Frame::decode(&bytes).unwrap_or_else(|e| panic!("{name}: decode: {e}"));
        assert_eq!(
            frame.encode(),
            bytes,
            "{name}: re-encode must be byte-exact"
        );
        assert_eq!(frame.channel, 0x30, "{name}: channel 1");
    }
}

#[test]
fn current_program_unpacks_to_prog() {
    let frame = Frame::decode(&fixture("current_program.syx")).unwrap();
    assert_eq!(frame.function, Function::CurrentProgramDump.code());
    let data = unpack(&frame.data);
    assert_eq!(data.len(), 448);
    assert_eq!(&data[0..4], b"PROG");
}

#[test]
fn global_unpacks_to_glob() {
    let frame = Frame::decode(&fixture("global.syx")).unwrap();
    assert_eq!(frame.function, Function::GlobalDump.code());
    let data = unpack(&frame.data);
    assert_eq!(data.len(), 96);
    assert_eq!(&data[0..4], b"GLOB");
}

#[test]
fn program_dump_carries_program_number_then_prog() {
    let frame = Frame::decode(&fixture("program_049.syx")).unwrap();
    assert_eq!(frame.function, Function::ProgramDump.code());
    // 0x4C echoes a 2-byte program number (pp PP) before the packed payload.
    let program = (frame.data[0] as u16) | ((frame.data[1] as u16) << 7);
    assert_eq!(program, 49);
    let data = unpack(&frame.data[2..]);
    assert_eq!(data.len(), 448);
    assert_eq!(&data[0..4], b"PROG");
}
