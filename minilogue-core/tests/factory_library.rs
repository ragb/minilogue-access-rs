//! Regression: every program in a real 200-slot Korg Librarian `.mnlglib`
//! decodes and round-trips byte-exact through the typed model.
//!
//! The fixture is Korg's factory patch data, so it is **not committed** (it is
//! kept locally and gitignored). These tests skip when it is absent — e.g. on
//! CI — and run in full on a machine that has it.

use std::io::{Cursor, Read};
use std::path::Path;

use minilogue_core::mnlg::read_library;
use minilogue_core::Program;

/// The library bytes, or `None` if the (uncommitted) fixture isn't present.
fn library_bytes() -> Option<Vec<u8>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("factory_library.mnlglib");
    if !path.exists() {
        eprintln!(
            "skipping: {} not present (Korg factory data, not redistributed)",
            path.display()
        );
        return None;
    }
    Some(std::fs::read(path).unwrap())
}

#[test]
fn imports_all_200_programs() {
    let Some(bytes) = library_bytes() else { return };
    let progs = read_library(&bytes).expect("read_library");
    assert_eq!(progs.len(), 200);
    assert_eq!(progs[0].program.name, "PolyLogue");
    assert_eq!(progs[1].program.name, "PWM Strings");
}

#[test]
fn every_real_program_round_trips_byte_exact() {
    let Some(bytes) = library_bytes() else { return };
    let mut archive = zip::ZipArchive::new(Cursor::new(&bytes[..])).unwrap();
    let mut names: Vec<String> = archive
        .file_names()
        .filter(|n| n.ends_with(".prog_bin"))
        .map(String::from)
        .collect();
    names.sort();
    assert_eq!(names.len(), 200);

    for name in &names {
        let mut bin = Vec::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_end(&mut bin)
            .unwrap();
        let prog = Program::from_bytes(&bin).unwrap_or_else(|e| panic!("{name}: decode: {e}"));
        assert_eq!(
            prog.to_bytes().unwrap().as_slice(),
            bin.as_slice(),
            "{name}: not byte-exact"
        );
    }
}
