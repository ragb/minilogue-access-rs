//! Typed global area: byte-exact round-trip + YAML round-trip against the fixture.

use std::path::Path;

use minilogue_core::global::{GlobalArea, KnobMode, MidiRoute, ParameterDisplay};
use minilogue_core::pack::unpack;
use minilogue_core::yaml::{global_from_yaml_str, global_to_yaml_string};
use minilogue_core::Frame;

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

fn global_payload() -> Vec<u8> {
    let frame = Frame::decode(&fixture("global.syx")).unwrap();
    unpack(&frame.data)
}

#[test]
fn global_bytes_round_trip_exact() {
    let payload = global_payload();
    let area = GlobalArea::from_bytes(&payload).expect("decode global");
    assert_eq!(area.to_bytes().unwrap().as_slice(), payload.as_slice());
}

#[test]
fn global_decodes_expected_values() {
    let area = GlobalArea::from_bytes(&global_payload()).unwrap();
    assert_eq!(area.midi_channel, 1); // device is on channel 1
    assert_eq!(area.brightness, 10); // wire 9 -> 1..=10
    assert_eq!(area.knob_mode, KnobMode::Jump);
    assert_eq!(area.midi_route, MidiRoute::UsbAndMidi);
    assert_eq!(area.parameter_display, ParameterDisplay::Normal);
    assert!(area.local_sw);
    assert_eq!(area.favorites.len(), 16);
    assert_eq!(area.favorites[0], 0);
    assert_eq!(area.favorites[1], 8);
}

#[test]
fn global_yaml_round_trips() {
    let area = GlobalArea::from_bytes(&global_payload()).unwrap();
    let yaml = global_to_yaml_string(&area).unwrap();
    let back = global_from_yaml_str(&yaml).unwrap();
    assert_eq!(back, area);
    // sanity: named enums render symbolically, not as integers
    assert!(yaml.contains("knob_mode: jump"), "yaml was:\n{yaml}");
}
