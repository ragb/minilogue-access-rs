//! Decode real device fixtures into typed values and print the YAML surface.
//!
//!     cargo run -p minilogue-core --example library_tour

use std::path::Path;

use minilogue_core::pack::unpack;
use minilogue_core::yaml::{global_to_yaml_string, program_to_yaml_string};
use minilogue_core::{Frame, Function, GlobalArea, Program};

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read(path).unwrap()
}

fn main() {
    // A stored program slot (0x4C dump: 2-byte program number, then payload).
    let frame = Frame::decode(&fixture("program_000.syx")).unwrap();
    let start = if frame.function == Function::ProgramDump.code() {
        2
    } else {
        0
    };
    let program = Program::from_bytes(&unpack(&frame.data[start..])).unwrap();
    println!("=== program_000 ({}): ===", program.name);
    println!("{}", program_to_yaml_string(&program).unwrap());

    // The global area.
    let frame = Frame::decode(&fixture("global.syx")).unwrap();
    let global = GlobalArea::from_bytes(&unpack(&frame.data)).unwrap();
    println!("=== global ===");
    println!("{}", global_to_yaml_string(&global).unwrap());
}
