//! `.mnlgprog` / `.mnlglib` container round-trips using real captured programs.

use std::path::Path;

use minilogue_core::mnlg::{read_library, write_library};
use minilogue_core::pack::unpack;
use minilogue_core::{
    read_mnlgprog, write_mnlgprog, Frame, Function, MnlgProgram, ProgInfo, Program,
};

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

fn program(name: &str) -> Program {
    let frame = Frame::decode(&fixture(name)).unwrap();
    let start = if frame.function == Function::ProgramDump.code() {
        2
    } else {
        0
    };
    Program::from_bytes(&unpack(&frame.data[start..])).unwrap()
}

#[test]
fn mnlgprog_round_trips() {
    // PolyLogue is a real factory patch captured from slot 0.
    let prog = program("program_000.syx");
    let info = ProgInfo {
        programmer: Some("Korg".into()),
        comment: Some("factory".into()),
    };
    let bytes = write_mnlgprog(&prog, &info).unwrap();

    // The output is a valid ZIP that re-imports to the same program + metadata.
    let back = read_mnlgprog(&bytes).unwrap();
    assert_eq!(back.program, prog);
    assert_eq!(back.info, info);
    // The 448-byte payload itself is byte-exact.
    assert_eq!(back.program.to_bytes().unwrap(), prog.to_bytes().unwrap());
}

#[test]
fn mnlglib_round_trips_multiple() {
    let progs: Vec<MnlgProgram> = ["program_000.syx", "program_049.syx", "current_program.syx"]
        .iter()
        .map(|n| MnlgProgram {
            program: program(n),
            info: ProgInfo::default(),
        })
        .collect();

    let bytes = write_library(&progs).unwrap();
    let back = read_library(&bytes).unwrap();
    assert_eq!(back.len(), 3);
    for (a, b) in progs.iter().zip(&back) {
        assert_eq!(a.program, b.program);
    }
    // Names survive.
    assert_eq!(back[0].program.name, "PolyLogue");
    assert_eq!(back[1].program.name, "QueBass");
}
