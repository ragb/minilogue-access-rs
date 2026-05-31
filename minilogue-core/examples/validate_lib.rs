//! Validate the codec against a real `.mnlglib` / `.mnlgprog`.
//!
//!     cargo run -p minilogue-core --example validate_lib -- <path>
//!
//! Imports the library and round-trips every `prog_bin` byte-exact through the
//! typed `Program` model.

use std::io::{Cursor, Read};

use minilogue_core::mnlg::read_library;
use minilogue_core::Program;

fn main() {
    let path = std::env::args().nth(1).expect("usage: validate_lib <path>");
    let bytes = std::fs::read(&path).unwrap();

    let progs = read_library(&bytes).expect("read_library");
    println!("read_library: {} programs imported", progs.len());
    let names: Vec<&String> = progs.iter().take(6).map(|p| &p.program.name).collect();
    println!("first names: {names:?}");

    // Byte-exact round-trip of every real prog_bin.
    let mut archive = zip::ZipArchive::new(Cursor::new(&bytes[..])).unwrap();
    let mut bin_names: Vec<String> = archive
        .file_names()
        .filter(|n| n.ends_with(".prog_bin"))
        .map(String::from)
        .collect();
    bin_names.sort();

    let (mut ok, mut fail) = (0u32, 0u32);
    for name in &bin_names {
        let mut bin = Vec::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_end(&mut bin)
            .unwrap();
        match Program::from_bytes(&bin).and_then(|p| p.to_bytes().map(|o| o.to_vec())) {
            Ok(out) if out == bin => ok += 1,
            Ok(out) => {
                fail += 1;
                if fail <= 3 {
                    let d: Vec<usize> = bin
                        .iter()
                        .zip(&out)
                        .enumerate()
                        .filter(|(_, (a, b))| a != b)
                        .map(|(i, _)| i)
                        .collect();
                    println!(
                        "  MISMATCH {name}: {} byte(s) at {:?}",
                        d.len(),
                        &d[..d.len().min(12)]
                    );
                }
            }
            Err(e) => {
                fail += 1;
                if fail <= 5 {
                    println!("  DECODE ERR {name}: {e}");
                }
            }
        }
    }
    println!(
        "byte-exact round-trip: {ok} ok, {fail} fail (of {})",
        bin_names.len()
    );
}
