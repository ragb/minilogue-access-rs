# minilogue-access-rs

A SysEx codec, CLI, and WASM bindings for the **Korg minilogue** (original
4-voice analog poly, 2016), plus byte-exact `.mnlgprog` / `.mnlglib` interop with
the Korg Sound Librarian format.

## Crates

| Crate | Purpose | Targets |
|---|---|---|
| `minilogue-core` | Pure codec: SysEx framing, 7→8 packing, typed program/global model, `.mnlgprog` ZIP I/O. No MIDI, no file I/O. | native + wasm32 |
| `minilogue` | CLI over `midir` (dump/sync/write/diff/show/lint/schema/mnlg). | native |
| `minilogue-wasm` | `wasm-bindgen` + `tsify-next` bindings for JS/TS. | wasm32 |

## Status

**Early build.** Protocol facts are in [docs/sysex-notes.md](docs/sysex-notes.md),
now confirmed against the real device (framing, ports, function codes, 7→8
packing, program/global sizes). Real captures live in
`minilogue-core/tests/fixtures/` (current program, global, two stored slots).

Done so far: the SysEx [`Frame`](minilogue-core/src/sysex.rs) codec, Korg
[7→8 packing](minilogue-core/src/pack.rs) (proptest invariant),
[`Function`](minilogue-core/src/function.rs) codes, the typed
[`GlobalArea`](minilogue-core/src/global.rs) (96 B) and
[`Program`](minilogue-core/src/program.rs) (448 B, with 10-bit param packing,
named enums, and the motion/note sequencer) — all green against a **byte-exact
round-trip of every device fixture**, with symbolic YAML and generated JSON
Schemas (`minilogue schema global|program`). See
[`examples/library_tour.rs`](minilogue-core/examples/library_tour.rs).

**Next:** `.mnlgprog`/`.mnlglib` ZIP interop, then the MIDI CLI (`dump`/`sync`/
`write` over the device's USB port 2) and the wasm editor. The CLI currently
exposes `ports` and `schema`.

## CLI (planned surface)

```
minilogue ports                                   # list MIDI ports
minilogue identity                                # probe model + channel
minilogue dump  --current      -o file.yaml       # edit buffer
minilogue dump  --program N    -o file.yaml       # stored slot 0..199
minilogue dump  --global       -o file.yaml
minilogue dump  --all          -o dir/            # global + all 200 slots
minilogue sync  --program N    -i file.yaml [--verify]
minilogue sync  --current      -i file.yaml
minilogue write N                                 # persist edit buffer to slot N
minilogue diff  a.yaml b.yaml
minilogue show  file.yaml
minilogue lint  file.yaml
minilogue schema program|global
minilogue mnlg  export file.yaml -o patch.mnlgprog
minilogue mnlg  import patch.mnlgprog -o file.yaml
minilogue mnlg  lib    dir/ -o bank.mnlglib
```

## Development

```
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Reverse-engineering scratch lives in [tools/explore/](tools/explore/) (throwaway
Python over `mido`). Captures are promoted from `tools/explore/captures/` (gitignored)
into `minilogue-core/tests/fixtures/` with a byte-exact round-trip test.

## License

MIT. See [LICENSE](LICENSE).
