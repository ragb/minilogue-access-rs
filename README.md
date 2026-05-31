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

**Working.** The protocol (in [docs/sysex-notes.md](docs/sysex-notes.md)) is fully
reverse-engineered and confirmed on hardware, and the core, CLI, and wasm
bindings are done and validated.

- **Core** — SysEx [`Frame`](minilogue-core/src/sysex.rs), Korg
  [7→8 packing](minilogue-core/src/pack.rs), the typed
  [`GlobalArea`](minilogue-core/src/global.rs) (96 B) and
  [`Program`](minilogue-core/src/program.rs) (448 B: 10-bit packing, named enums,
  motion/note sequencer), and [`.mnlgprog`/`.mnlglib`](minilogue-core/src/mnlg.rs)
  interop. Symbolic YAML + generated JSON Schemas.
- **CLI** — `ports`, `identity`, `dump`/`sync` over USB **port 2**, `show`,
  `schema`, and `mnlg import/export/lib`. Tested live against the device.
- **wasm** — [full TypeScript bindings](minilogue-wasm/src/lib.rs): decode/encode
  program & global dumps, request builders, `.mnlgprog`/`.mnlglib` import/export,
  YAML conversion — typed via `tsify`.

**Validation:** every codec path round-trips byte-exact against device captures
**and** all 200 programs of a genuine Korg Sound Librarian `.mnlglib`.

**Next:** the accessible (ARIA / keyboard / screen-reader) web editor on top of
the wasm bindings; minor flag-byte (66/72/73) semantic decode.

## CLI

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
