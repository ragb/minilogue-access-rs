//! The parameter catalog must stay in lock-step with the real model: every
//! `path` must resolve in the serde structure, and every choice option token
//! must be a real serde enum token.

use std::collections::HashSet;
use std::path::Path;

use minilogue_core::pack::unpack;
use minilogue_core::params::{help_for, Kind, GLOBAL_PARAMS, PROGRAM_PARAMS};
use minilogue_core::{Frame, Function, GlobalArea, Program};

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read(path).unwrap()
}

fn program() -> Program {
    let f = Frame::decode(&fixture("program_000.syx")).unwrap();
    let start = if f.function == Function::ProgramDump.code() {
        2
    } else {
        0
    };
    Program::from_bytes(&unpack(&f.data[start..])).unwrap()
}

fn global() -> GlobalArea {
    let f = Frame::decode(&fixture("global.syx")).unwrap();
    GlobalArea::from_bytes(&unpack(&f.data)).unwrap()
}

fn nav<'a>(value: &'a serde_yaml::Value, path: &str) -> Option<&'a serde_yaml::Value> {
    let mut cur = value;
    for seg in path.split('.') {
        cur = cur.get(seg)?;
    }
    Some(cur)
}

fn check(value: &serde_yaml::Value, metas: &[minilogue_core::params::Meta], what: &str) {
    for m in metas {
        let node =
            nav(value, m.path).unwrap_or_else(|| panic!("{what}: path {:?} not in model", m.path));
        if let Kind::Choice { options } = m.kind {
            let tok = node
                .as_str()
                .unwrap_or_else(|| panic!("{what}: {:?} is not a string token", m.path));
            assert!(
                options.iter().any(|o| o.value == tok),
                "{what}: {:?} value {tok:?} is not among its options",
                m.path
            );
        }
    }
}

#[test]
fn program_paths_and_choice_tokens_match_model() {
    check(
        &serde_yaml::to_value(program()).unwrap(),
        PROGRAM_PARAMS,
        "program",
    );
}

#[test]
fn global_paths_and_choice_tokens_match_model() {
    check(
        &serde_yaml::to_value(global()).unwrap(),
        GLOBAL_PARAMS,
        "global",
    );
}

#[test]
fn paths_unique_help_present_and_lookup_works() {
    let mut seen = HashSet::new();
    for m in PROGRAM_PARAMS.iter().chain(GLOBAL_PARAMS) {
        assert!(seen.insert(m.path), "duplicate path {:?}", m.path);
        assert!(!m.help.is_empty(), "{:?} has no help", m.path);
        if let Kind::Choice { options } = m.kind {
            assert!(!options.is_empty(), "{:?} has no options", m.path);
        }
    }
    assert!(help_for("filter.cutoff").is_some());
    assert!(help_for("midi_channel").is_some());
    assert!(help_for("does.not.exist").is_none());
}
