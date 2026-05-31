# minilogue SysEx & program-data notes

Running log of protocol findings for the **original Korg minilogue** (4-voice
analog poly, 2016 — *not* the xd, monologue, or prologue). Append-only-ish:
mark refuted entries, don't delete. Confidence: **HIGH** (from Korg's MIDI
Implementation, verbatim), **MED** (cross-referenced from open-source editors),
**LOW** (guess / forum / to-confirm).

> Methodology reminder: the spec is dense and easy to misread. Every claim here
> gets confirmed against a **captured fixture** before code rests on it. During
> research a subagent confused the minilogue with the minilogue **xd** (336-byte
> program, 448-byte `.prog_bin`, Cycling '74 tutorial) — all xd, all wrong for
> this device. Fixtures are ground truth.

## 1. Primary spec

- Korg minilogue **MIDI Implementation**, Rev 1.00, 2016-02-10.
- Mirror used: <https://gist.github.com/eric-wood/1d5916895b3da0aee183> (verbatim
  copy of the official Korg document). Trust level: **HIGH** for framing,
  function codes, sizes; the parameter offset table needs fixture confirmation.
- Sound Librarian + factory libraries: <https://www.korg.com/us/products/synthesizers/minilogue/librarian_contents.php>
- Cross-reference editors/tools (verify, don't trust):
  - jeffkistler/minilogue-editor (JS/React, WebMIDI) — codec + offset tables.
  - gazzar/loguetools (Python) — original-minilogue `.mnlgprog`/`.mnlglib` tooling.

## 2. Framing (HIGH, header trailing byte to confirm)

Korg "extended" format. **No checksum** — integrity comes from 7→8 unpacking
(data bytes must stay ≤ 0x7F).

| Field | Value | Notes |
|---|---|---|
| SOX | `F0` | |
| Manufacturer | `42` | Korg |
| Channel byte | `0x30 \| (ch-1)` | ch is 1-based in the UI. Channel 1 → `0x30`. Broadcast = `0x7F`. |
| Format header | `00 01` | constant |
| Model ID | `2C` | **minilogue**. monologue=`44`, minilogue xd=`51`, prologue=`4B`. One byte. |
| Function | 1 byte | the message identity — there is **no Roland-style address tree**. |
| Data | function-specific | program/global dumps are 7→8 packed; request frames carry raw bytes. |
| EOX | `F7` | |

Known-good request on the wire (**device-confirmed 2026-05-31**):
current-program dump request = `F0 42 30 00 01 2C 10 F7`.

> **RESOLVED (header), 2026-05-31:** header is exactly `F0 42 3g 00 01 2C`
> followed by the function byte — **7 bytes incl. function, no trailing `01`**.
> Confirmed from real reply frames: current dump = `F0 42 30 00 01 2C 40 …`,
> global = `… 2C 51 …`, program = `… 2C 4C …`. The `01` in Korg's doc text was
> the family-MSB notation, not an on-wire byte.

> **DEVICE QUIRK (USB), 2026-05-31:** on this unit the minilogue enumerates two
> USB MIDI ports — `minilogue` (port 1) and `MIDIIN2/MIDIOUT2 (minilogue)`
> (port 2). **Bulk SysEx dumps only work on port 2**: requests sent to the plain
> `minilogue` output get no reply; requests on `MIDIOUT2` reply on `MIDIIN2`.
> Port 1 is keyboard/realtime. The Rust `MidiSession` must select the port-2
> device for dumps.

> **DEVICE QUIRK (channel), 2026-05-31:** broadcast channel `0x7F` was **ignored**
> for dump requests — the channel nibble must match the device's global MIDI
> channel (default 1 → `0x30`). (Differs from RE-202, which honored `0x7F`.)

`const MODEL_ID: u8 = 0x2C;` lives in exactly one place so sibling models stay a
future feature-flag, not a rewrite.

## 3. Function code map (HIGH)

| Code | Dir | Meaning | Notes |
|---|---|---|---|
| `10` | host→synth | CURRENT PROGRAM DATA DUMP REQUEST | reply `40` |
| `40` | synth→host | CURRENT PROGRAM DATA DUMP | edit buffer / live patch |
| `1C` | host→synth | PROGRAM DATA DUMP REQUEST (1 prog) | + 2-byte program number; reply `4C` |
| `4C` | synth→host | PROGRAM DATA DUMP (1 prog) | requested slot |
| `0E` | host→synth | GLOBAL DATA DUMP REQUEST | reply `51` |
| `51` | synth→host | GLOBAL DATA DUMP | |
| `23` | synth→host | DATA LOAD COMPLETED (ACK) | |
| `24` | synth→host | DATA LOAD ERROR (NAK) | |
| `26` | synth→host | DATA FORMAT ERROR | |

Program number in the `1C` request is split across two bytes: `pp` = number &
0x7F (LSB, 0–127), `PP` = (number >> 7) & 1 (MSB bit). Range 0–199.

**Device-confirmed 2026-05-31** (functions `40`, `4C`, `51` all observed live):
- `40` CURRENT PROGRAM DUMP reply = 520 B = `F0 42 30 00 01 2C 40` + 512 packed + `F7`.
- `51` GLOBAL DUMP reply = 118 B = header + 110 packed + `F7`.
- `4C` PROGRAM DUMP reply = 522 B: the **2-byte program number (`pp PP`) is echoed
  right after the function byte**, *before* the 512-byte packed payload. So the
  packed payload starts at frame index 9 for `4C`, but index 7 for `40`/`51`.

> **RESOLVED (write/load), 2026-05-31** (non-destructive probe — sent captured
> slot data back identically):
> - Inbound **`4C` PROGRAM DATA DUMP** (with a 2-byte program number) is **written
>   to that slot** and ACKed with `23`. The edit buffer is left untouched, so
>   `4C` targets the slot directly — **this is the program-write mechanism**; no
>   separate `11`-style write-request exists.
> - Inbound **`40` CURRENT PROGRAM DATA DUMP** loads the **edit buffer**, ACK `23`.
> - ACK = 8-byte `F0 42 3g 00 01 2C 23 F7` (DATA LOAD COMPLETED). NAK would be
>   `24`, format error `26`.
> - Still unverified (needs a destructive test on a scratch slot + power cycle):
>   that a `4C` write survives power-off. The `23` ACK ("LOAD COMPLETED") implies
>   a flash store; defer the power-cycle confirmation unless needed.

> **RESOLVED (identity), 2026-05-31:** the **Universal Identity Request**
> `F0 7E 7F 06 01 F7` works — but **only on USB port 2** (`MIDIOUT2`→`MIDIIN2`),
> same as bulk SysEx. Reply (15 bytes):
> `F0 7E 00 06 02 42 2C 01 00 00 01 00 15 00 F7`
> = non-realtime / device ch `00` / general-info / identity-reply / Korg `42` /
> family `2C 01` / member `00 00` / version `01 00 15 00` (4 bytes; likely the
> firmware version — correlate with the device's displayed version). No Korg
> SEARCH DEVICE message is needed.

## 4. 7→8 bit packing (HIGH)

`DATA (1 set = 8bit × 7 bytes) → MIDI DATA (1 set = 7bit × 8 bytes)`.

Pack: take 7 source bytes `s0..s6`. Emit one MSB byte whose bit *j* = bit7 of
`s_j` (bit 0 = MSB of s0). Then emit `s0 & 0x7F .. s6 & 0x7F`. Last group may be
short — pack what you have. Unpack is the inverse.

Applies to **program data** and **global data**, not to request frames. Will live
in `pack.rs` with proptest `unpack(pack(x)) == x`.

**Device-confirmed 2026-05-31:** the Python reference unpacker in
`tools/explore/unpack_inspect.py` correctly recovers `"PROG"`/`"GLOB"` markers
and 12-char names from real `40`/`4C`/`51` frames (448 and 96 unpacked bytes
exactly). Worked example — current dump payload begins `00 50 52 4F 47 …`: MSB
byte `00` → no high bits → data `50 52 4F 47` = `PROG`.

## 5. Program data layout (sizes HIGH; offsets MED — confirm against fixtures)

- **448 bytes unpacked / 512 packed** (`448 = 7×64 → 8×64 = 512`).
- Header `"PROG"` at 0–3. Program **name** at 4–15 (12 ASCII chars).
- Synth params 20–73 (consolidated blueprint, from og.py struct + Korg doc):
  - **10-bit params** (value = `(b2_9 << 2) | low2`, range 0–1023): upper byte at
    `b2_9` offset, low 2 bits at byte/bits below:
    - 20 vco1_pitch (low 52 b0–1), 21 vco1_shape (52 b2–3),
      22 vco2_pitch (53 b0–1), 23 vco2_shape (53 b2–3),
      24 cross_mod_depth (54 b0–1), 25 vco2_pitch_eg_int (54 b2–3),
      26 vco1_level (54 b4–5), 27 vco2_level (54 b6–7),
      28 noise_level (55 b2–3), 29 cutoff (55 b4–5), 30 resonance (55 b6–7),
      31 cutoff_eg_int (56 b0–1),
      34–37 amp_eg A/D/S/R (57 b0–1/2–3/4–5/6–7),
      38–41 mod_eg A/D/S/R (58 b0–1/2–3/4–5/6–7),
      42 lfo_rate (59 b0–1), 43 lfo_int (59 b2–3),
      49 delay_hi_pass (62 b2–3), 50 delay_time (62 b4–5),
      51 delay_feedback (62 b6–7), 70 voice_mode_depth (64 b4–5).
  - **Categorical bits**: 52 b4–5 vco1_octave (0=16'…3=2'), 52 b6–7 vco1_wave
    (0=SQR,1=TRI,2=SAW); 53 same for vco2; 55 b0 sync, b1 ring; 56 b2–3
    cutoff_velocity (0/50/100%), b4–5 cutoff_kbd_track, b6 cutoff_type
    (0=2-pole,1=4-pole); 59 b4–5 lfo_target (0=cutoff,1=shape,2=pitch), b6–7
    lfo_eg (0=off,1=rate,2=int); 60 b0–1 lfo_wave (SQR/TRI/SAW), b6–7
    delay_routing (0=bypass,1=pre,2=post); 64 b0–2 voice_mode (POLY/DUO/UNISON/
    MONO/CHORD/DELAY/ARP/SIDECHAIN).
  - **Scalars**: 33 amp_velocity, 61 portamento_time, 66 bend_range (±1–12),
    71 program_level (77–127 = −25..+25 dB), 72 slider_assign (0–28), 73
    keyboard_octave (0–4 = −2..+2).
  - **RESOLVED (bytes 58–60), device-confirmed 2026-05-31:** turned LFO RATE (and
    INT) to max and diffed the edit buffer. `lfo_rate` upper = byte 42, `lfo_int`
    upper = byte 43, and **byte 59 went `03`→`0F`** — bits 0–1 = `lfo_rate` low2,
    bits 2–3 = `lfo_int` low2. So the low bits are in **byte 59** (Korg-doc /
    struct-name reading), confirming the table above; og.py's *normalisation
    table* (which said byte 60) is the buggy source. Byte 59 b4–5 = lfo_target,
    b6–7 = lfo_eg; byte 60 b0–1 = lfo_wave, b6–7 = delay_routing; byte 58 =
    mod_eg low2 (no collision, per og.py).
- **Sequencer** block header `"SEQD"` at 96. BPM (100–101, value 100–3000 =
  10.0–300.0), step length (103), swing (104), default gate time (105), step
  resolution (106), step on/off + switch flags (108–111), **4 motion slots**
  (112–119) + motion step flags (120–127), and **step event data 128–447 =
  20 bytes/step × 16 steps = 320 bytes** (~70% of the program).

The motion + note sequencer is the single biggest substructure. YAML surface
decision: **inline, fully expanded** (all 16 steps + 4 motion lanes as nested
lists — verbose but greppable/diffable). Exact 20-byte step layout TBD from a
fixture with a deliberately distinctive programmed sequence.

**Cross-reference (MED):** `gazzar/loguetools` `loguetools/og.py` holds the
original-minilogue patch struct (`minilogue_og_patch_struct`) and a
normalisation table (`minilogue_og_patch_normalisation`) that documents the
10-bit param packing: each param's upper 8 bits live at a `*_b2_9` offset (20–51)
and its low 2 bits in a shared byte at 52–62. E.g. VCO1 PITCH = byte 20 (high 8)
+ byte 52 bits 0–1; VCO1 OCTAVE = byte 52 bits 4–5; VCO1 WAVE = byte 52 bits 6–7.
Validate every offset against a fixture before trusting it.

**Sequencer decode (fixture-confirmed 2026-05-31, `current_with_sequence.syx`):**
- 96–99 `"SEQD"`.
- 100–101 **BPM**, little-endian, value 100–3000 = 10.0–300.0. Fixture `B0 04` =
  1200 = 120.0 BPM.
- 102 ? (`02`); 103 **step length** 1–16 (fixture `10` = 16); 104 **swing**
  signed −75..+75 (fixture `00`); 105 **default gate time** 0–72 = 0–100% (fixture
  `36` = 54); 106 **step resolution** 0–4 (fixture `00`); 107 ? (`00`).
- 108–109 **step on/off bitmap**, 16 bits LE (fixture `1D 00` = `0x001D` = steps
  1,3,4,5). 110–111 a second 16-bit bitmap, also `0x001D` here (motion-on or
  active-step — distinguish later).
- 112–127 **motion slots** params + step flags (all zero in fixture — no motion
  recorded).
- 128–447 **step event data, 20 bytes/step × 16 steps**. Only the on-steps carry
  data; within a step, note values sit at 4-byte slots (+0,+4,+8…). Fixture step
  roots: s1 `3B`(59), s3 `3C`(60), s4 `3E`(62), s5 `41`(65) — a rising line. The
  exact 20-byte sub-layout (notes vs velocity/gate/tie per slot; the constant
  `36` at +8) needs one more targeted capture to pin down, but the step framing
  (20 B/step from 128, gated by the 108–109 bitmap) is confirmed.

## 6. Global data layout (sizes HIGH; offsets MED)

- **96 bytes unpacked / 110 packed.** Header `"GLOB"` at 0–3.
- Master tune (4, −50..+50), transpose (5, −12..+12), velocity curve (6, 0–8),
  knob mode (7, 0–2), audio in (8), clock source (9, 0–2), sync in/out unit +
  polarity (10–13), MIDI route (16), MIDI channel (17, 0–15 = ch 1–16), local
  (18), Rx/Tx short-message enables (19–20), brightness (24), auto power off
  (25), parameter display (26), oscilloscope (27), **favorites 1–8 (64–71,
  values 0–199)**.

## 7. `.mnlgprog` / `.mnlglib` container (MED — confirm from a real export)

ZIP archives. Expected members (verify exact names + XML schema from a real file
and from gazzar/loguetools):
- `Prog_000.prog_bin` — the 448-byte unpacked program payload (same bytes as a
  `40`/`4C` dump after 7→8 unpacking).
- `Prog_000.prog_info` — small XML (program name, author/comment, category,
  favorite flag).
- `FavoriteData.fav_data` (libraries) — favorite-slot data.
- `FileInformation.xml` (libraries) — index of contained programs.

Round-trip acceptance: a Librarian export, imported then re-exported by our tool,
must be **byte-exact**.

## 8. Slots

200 programs (0–199). Per the manual, factory content in the lower slots, user
slots above.

## 9. Refuted / dead ends

- ~~minilogue program data is 336 bytes; `.prog_bin` is 448 bytes~~ — that's the
  minilogue **xd**. Original minilogue: program data **is** 448 bytes; packed
  dump 512 bytes. (Research subagent contamination, 2026-05-31.)
- ~~Model ID `0x51`~~ — that's the minilogue **xd** model ID *and* the original's
  GLOBAL DATA DUMP function code. Original minilogue model ID = `0x2C`.

## 10. Captured fixtures (2026-05-31)

Promoted to `minilogue-core/tests/fixtures/` (firmware version TBD — to annotate):

| File | Func | Frame | Unpacked | Name |
|---|---|---|---|---|
| `current_program.syx` | `40` | 520 B | 448 B | `TriBell` (edit buffer) |
| `global.syx` | `51` | 118 B | 96 B | — |
| `program_000.syx` | `4C` | 522 B | 448 B | `PolyLogue` |
| `program_049.syx` | `4C` | 522 B | 448 B | `QueBass` |

Still wanted: a slot with a deliberately distinctive note + motion sequence
(to decode the 20-byte/step layout), and a `.mnlgprog`/`.mnlglib` export.

## 11. Open questions to settle

1. ~~Header byte count before the function code~~ — **RESOLVED**: 7-byte header,
   no trailing `01` (§2).
2. ~~PROGRAM WRITE mechanism~~ — **RESOLVED** (§3): send a `4C` PROGRAM DATA
   DUMP with the program number; device writes the slot and ACKs `23`. (Flash
   persistence across power cycle still optional-to-verify.)
3. ~~Identity: does Universal Identity Request work?~~ — **RESOLVED** (§3): yes,
   on USB port 2; reply records family `2C 01` + version bytes. Version-byte
   encoding still to map to the displayed firmware number.
4. Exact 20-byte sequencer step-event layout (tie/rest/note/velocity/gate).
   Currently preserved verbatim in `Step::event`; structure known (notes at
   4-byte slots) but sub-fields not decoded.
5. ~~The 10-bit param low-2-bit packing positions (52–62)~~ — **RESOLVED**:
   modelled in `program.rs`, byte-exact against all fixtures; LFO bits
   device-confirmed (§5).
6. **Flag bytes 66 (bend_range), 72 (slider_assign), 73 (keyboard_octave)** read
   outside their documented ranges (e.g. 204, 77, 0xFB) → they carry flag bits in
   the upper portion. Preserved raw and round-trip exact, but need a knob-diff to
   decode cleanly (same method that settled the LFO bits).
6. Firmware version of the unit on hand, and whether layout changed across
   firmware (annotate fixtures once known).
7. `.mnlgprog`/`.mnlglib` exact member names + `prog_info` XML schema.
