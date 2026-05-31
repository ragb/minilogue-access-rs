//! Export captured device programs to real `.mnlgprog` / `.mnlglib` files.
//!
//!     cargo run -p minilogue-core --example export_samples -- <out_dir>

use std::path::{Path, PathBuf};

use minilogue_core::mnlg::{write_library, write_mnlgprog, MnlgProgram, ProgInfo};
use minilogue_core::pack::unpack;
use minilogue_core::{Frame, Function, Program};

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read(path).unwrap()
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

fn main() {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: export_samples <out_dir>"),
    );
    std::fs::create_dir_all(&out).unwrap();

    let info = ProgInfo {
        programmer: Some("minilogue-access-rs".into()),
        comment: Some("captured from device".into()),
    };

    for (fix, file) in [
        ("program_000.syx", "PolyLogue.mnlgprog"),
        ("program_049.syx", "QueBass.mnlgprog"),
    ] {
        let p = program(fix);
        let bytes = write_mnlgprog(&p, &info).unwrap();
        let path = out.join(file);
        std::fs::write(&path, &bytes).unwrap();
        println!("{} <- {} ({} bytes)", path.display(), p.name, bytes.len());
    }

    let lib: Vec<MnlgProgram> = ["program_000.syx", "program_049.syx", "current_program.syx"]
        .iter()
        .map(|n| MnlgProgram {
            program: program(n),
            info: info.clone(),
        })
        .collect();
    let bytes = write_library(&lib).unwrap();
    let path = out.join("minilogue_captured.mnlglib");
    std::fs::write(&path, &bytes).unwrap();
    println!("{} <- 3 programs ({} bytes)", path.display(), bytes.len());
}
