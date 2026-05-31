# minilogue explore tools

Throwaway Python scripts for probing the Korg minilogue and capturing fixtures.
Not shipped — the Rust crates are the product; this is the lab notebook.

## Setup (Windows PowerShell)

```powershell
py -3 -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
```

`python-rtmidi` ships precompiled wheels, so no build toolchain is needed.

## Scripts

- `probe.py` — send one Korg SysEx function frame, print replies, optionally save
  the first reply to a `.syx`. Frame shape: `F0 42 3g 00 01 2C <func> [data] F7`
  (no checksum). Start here.
- `capture.py` — capture the standard fixture set with friendly filenames
  (current program, a stored slot, global).
- `unpack_inspect.py` — load a captured `.syx`, strip framing, run the Korg 7→8
  unpack, and print the unpacked bytes with offsets + ASCII. Your eyeball-debug
  tool for the program layout before the Rust struct exists.

## Workflow

1. `python probe.py --list-ports` — find the minilogue's port name.
2. `python capture.py --port <name> current -o captures/current_program.syx`
3. `python unpack_inspect.py captures/current_program.syx` — confirm size,
   `"PROG"` marker, and the 12-char name; eyeball the offset table.
4. Record findings in `../../docs/sysex-notes.md`.
5. Promote a clean capture into `../../minilogue-core/tests/fixtures/` with a
   byte-exact round-trip test once the codec exists.

Captures land in `captures/` (gitignored) until promoted.
