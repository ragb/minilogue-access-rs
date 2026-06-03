# Integration prompt: add the minilogue to the **able-midi** editor

Paste this into a fresh Claude Code session opened in the `able-midi` editor
repo (the SvelteKit app at `midi-ccess/`, package name `able-midi`). It adds the
Korg minilogue as a device, reusing the `minilogue-wasm` codec built in
`minilogue-access-rs`. Do the work in **that** repo; this repo only ships the
wasm.

---

## Bootstrap prompt

> Add the **Korg minilogue** as a device in this able-midi editor, mirroring how
> `re202`, `gr55`, and `ml10x` are integrated. The Rust codec is already built
> and published: the `ragb/minilogue-access-rs` repo's CI uploads a
> `minilogue-wasm-pkg` artifact (wasm-pack `--target web` output) with full
> TypeScript types. It exposes decode/encode for program & global SysEx dumps,
> request-frame builders, `.mnlgprog`/`.mnlglib` import/export, YAML conversion,
> and — unlike the other devices — a **parameter metadata + help catalog**
> (`programParamCatalog()`, `globalParamCatalog()`, `helpFor(path)`) so most of
> the UI can be generated from one source of truth. Build the device module,
> codec facade, store, and an accessible route, register it, and wire the wasm
> fetch. Keep everything screen-reader-first (ARIA names, keyboard nav,
> value-change announcements via the existing `announcements` store).

---

## What the editor already gives you (reuse it)

- **Device contract** `src/lib/devices/types.ts` → `DeviceModule` (`id`, `name`,
  `route`, `identity.request()/matches()`, `inbound()`, optional `yaml`,
  optional `handshake()`).
- **Registry** `src/lib/devices/registry.ts` — add the module to `MODULES`.
- **wasm fetch** `scripts/fetch-wasm.mjs` — pulls each device's CI artifact into
  `vendor/wasm/<id>/` via `gh run download`. Loaded through the `@wasm/<id>`
  alias with `vite-plugin-wasm` + `vite-plugin-top-level-await`.
- **Accessible components** `src/lib/components/`: `RangeField`, `NumberField`,
  `EnumSelect`, `SwitchField`, `StringArrayField`, `HelpButton`, `HelpDialog`,
  `SlotPicker`, `YamlPanel`, `Tabs`, `StatusRegion`, `PortPicker`,
  `ConnectionDialog`.
- **Stores** `src/lib/stores/`: `connection.svelte.ts`, `announcements.svelte.ts`
  (`announce(...)`), `prefs.svelte.ts`, plus per-device runes stores to mirror.
- **MIDI** `src/lib/midi/`: `access.ts`, `session.ts`, `sysex-stream.ts`,
  `messages.ts`.

## Steps

1. **Fetch the wasm.** In `scripts/fetch-wasm.mjs` add to `TARGETS`:
   `{ id: 'minilogue', repo: 'ragb/minilogue-access-rs', artifact: 'minilogue-wasm-pkg' }`.
   Run `node scripts/fetch-wasm.mjs --only minilogue` → populates
   `vendor/wasm/minilogue/`. Confirm the `@wasm/minilogue` alias resolves (same
   mechanism as `@wasm/re202`; it's defined in `svelte.config.js`/vite — add a
   `minilogue` entry if the aliases are enumerated).

2. **Codec facade** `src/lib/devices/minilogue/codec.ts` — thin wrapper over
   `@wasm/minilogue`, exactly like `re202/codec.ts`: lazy `ensureCodec()` calling
   `init()`, then re-export the typed helpers:
   `decodeProgramDump`, `encodeCurrentProgram`, `encodeProgramWrite(program, slot, channel)`,
   `currentProgramRequest`, `programRequest`, `decodeGlobalDump`, `encodeGlobal`,
   `globalRequest`, `importMnlgprog`, `exportMnlgprog`, `importLibrary`,
   `exportLibrary`, `programToYaml/programFromYaml`, `globalToYaml/globalFromYaml`,
   `programParamCatalog()`, `globalParamCatalog()`, `helpFor(path)`, `modelId()`.
   Types `Program`, `GlobalArea`, `MnlgProgram`, `ProgInfo`, `Library`, plus the
   enum unions (`Wave`, `VoiceMode`, …) all come from `@wasm/minilogue`.

3. **Device module** `src/lib/devices/minilogue/module.ts`:
   - `id: 'minilogue'`, `name: 'Korg minilogue'`, `route: '/minilogue/'`.
   - `identity.request()` → `new Uint8Array([0xf0,0x7e,0x7f,0x06,0x01,0xf7])`.
   - `identity.matches(reply)` → `F0 7E dd 06 02 42 2C 01 …`: manufacturer `0x42`
     at byte 5, family `2C 01` at bytes 6–7.
   - `inbound(bytes)` → classify by the function byte at index 6 of a
     `F0 42 3g 00 01 2C <func> … F7` frame (`0x40` current, `0x4C` program,
     `0x51` global, `0x23` ACK, `0x24/0x26` errors); use the codec to decode.
   - `yaml`: wire `programToYaml/fromYaml`.

4. **⚠ Port selection (device-specific).** The minilogue exposes **two** USB MIDI
   ports; **bulk SysEx only works on port 2** (`MIDIOUT2`/`MIDIIN2 (minilogue)`).
   Port 1 silently ignores dump requests. The connection flow must let the user
   pick — or auto-prefer — the port whose name contains `MIDIOUT2`/`MIDIIN2`.
   Default MIDI channel is 1 (wire channel byte `0x30`).

5. **Store** `src/lib/stores/minilogue.svelte.ts` (Svelte 5 runes), mirroring
   `re202.svelte.ts`: hold the current `Program` draft + `GlobalArea`, the
   selected slot (0–199), connection wiring, and actions: `requestCurrent`,
   `requestProgram(n)`, `requestGlobal`, `sendCurrent`, `writeSlot(n)`
   (`encodeProgramWrite`, expect `0x23` ACK), `sendGlobal`, plus
   `updateParam(path, value)` that sets a value by catalog path and
   `announce(...)`s the new value+label for the screen reader.

6. **Route** `src/routes/minilogue/+page.svelte` — generate the UI from the
   catalog. For each entry from `programParamCatalog()` (and `globalParamCatalog()`):
   - `kind.type === 'range'` → `RangeField`/`NumberField` (min/max/unit, `label`).
   - `kind.type === 'choice'` → `EnumSelect` (options `[{value,label}]`).
   - `kind.type === 'bool'` → `SwitchField`.
   - `kind.type === 'text'` → text input (program name, 12 chars).
   - `meta.level === true` (range entries only) → a magnitude the editor should
     opt into percentage display when the user's `levelDisplay` preference is
     set to percent (as GR-55/CK already do); render those with
     `<RangeField kind="level">`. The flag is omitted when false, so treat a
     missing `level` as plain (raw value) — centred/bipolar fields (pitch,
     tune, swing) and discrete values stay raw.
   - `helpFor(path)` → `HelpButton`/`HelpDialog` per control.
   Group controls with `Tabs` by `meta.group` (VCO 1, VCO 2, VCO 2 Modulation,
   Mixer, Filter, Amp EG, EG, LFO, Delay, Voice, Sequencer; Global on its own
   tab). Use `SlotPicker` for the 200 slots, `YamlPanel` for `.mnlgprog`/
   `.mnlglib` + YAML import/export, `StatusRegion` for announcements.

7. **Register** in `registry.ts` (`minilogueModule`) and add a home-page entry.

## Device facts to honor

- **200 program slots** (0–199); writing a slot = send a `0x4C` PROGRAM dump
  (`encodeProgramWrite`); the device persists and ACKs `0x23`. Loading the edit
  buffer = `0x40` (`encodeCurrentProgram`).
- **Program = 448 bytes**, global = 96 bytes; the codec handles 7→8 packing.
- **Named enums** render with friendly labels from the catalog (`octave: "16'"`,
  `filter type: "2-pole (12 dB)"`, etc.) — don't re-derive them.
- **Sequencer**: the 16-step on/off, BPM/length/swing/gate/resolution, and 4
  motion lanes are modelled; the per-step 20-byte event payload is preserved
  verbatim (`Step.event`) but not yet sub-decoded, so offer step on/off + the
  sequencer header params now and leave per-step note editing for later.
- **Not yet in the catalog** (pending an on-device knob-diff decode): `bend_range`,
  `slider_assign`, `keyboard_octave` (they carry flag bits). Skip them in the UI
  for now; they still round-trip (preserved raw).

## Verify

- `npm run dev`, open `/minilogue/`, connect the synth on **port 2**, run the
  identity probe (should match), dump the current program, edit a few params
  (confirm the screen reader announces label + new value), write to a scratch
  slot and read it back to confirm.
- Import the `.mnlglib` produced by the Korg Sound Librarian and confirm all 200
  programs load; export a `.mnlgprog` and re-import it.
- `npm run check` (svelte-check) clean.
