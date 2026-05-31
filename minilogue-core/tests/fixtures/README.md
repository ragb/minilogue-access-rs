# minilogue fixtures

Raw byte captures used by `minilogue-core` round-trip tests. Ground truth — the
data model is verified against these, not the other way around.

## SysEx dumps (`.syx`)

A single SysEx frame each (`F0 42 3g 00 01 2C <func> ... F7`). Naming:
`<what>[_<detail>].syx`, e.g.:

- `current_program.syx` — function `40`, the edit buffer.
- `program_000_factory.syx` — a stored factory slot (function `4C`).
- `program_049_edited.syx` — a user-edited slot with non-default values.
- `program_with_sequencer.syx` — a slot with a deliberately distinctive note +
  motion sequence (to decode the 20-byte/step layout).
- `global.syx` — function `51`.
- `identity_reply.syx` — reply to the Universal Identity Request, if supported.

Record the unit's **firmware version** in this file next to each capture, since
the program-data layout may differ across firmware.

## Librarian containers (`.mnlgprog`, `.mnlglib`)

Real exports from the Korg Sound Librarian (or a factory library), used to verify
byte-exact `.mnlgprog`/`.mnlglib` round-tripping.

## Adding a fixture

When you add a fixture, add a test asserting
`from_bytes(fixture).to_bytes() == fixture` (byte-exact). Exploratory captures
stay in `tools/explore/captures/` (gitignored) until promoted here.
