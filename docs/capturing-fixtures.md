# Capturing minilogue fixtures (screen-reader friendly)

Ground-truth captures we need before writing the codec. Every step ends with
"report back: …" — that's what to tell the assistant. No step requires reading
the minilogue's display.

## Part A — Get the minilogue onto the computer as a MIDI port

Right now the computer does **not** see a port named "minilogue" (it sees
Komplete Kontrol, a Focusrite USB MIDI interface, and the GS Wavetable synth).
The synth must appear as its own USB port before we can capture anything.

1. Find the **USB Type B** socket on the minilogue's rear panel (square-ish
   connector, same shape as a printer cable's far end). Connect a USB cable from
   there directly to a USB port on the computer (not through a hub if avoidable).
2. Find the **power switch** on the minilogue's rear panel, near the power
   socket, and switch it on. The synth is on when its keys play sound.
3. Wait about 10 seconds for Windows to enumerate the device.
4. Report back: nothing yet — go straight to step 5.
5. The assistant will re-run the port list (`cargo run --bin minilogue -- ports`).
   Report back: read out the list of input and output port names it prints, and
   say whether one of them now contains the word "minilogue".

If "minilogue" still does not appear after a known-good USB cable and a power
cycle, it may be wired by 5-pin DIN into the Focusrite instead of USB. Tell the
assistant which cable type you used and we'll adapt (DIN routing uses the
"Focusrite USB MIDI" port instead).

Also, one-time, report back the **firmware version**: hold the minilogue's
`SHIFT` button and, while holding it, press the key/button labelled for the
global menu, then step to the firmware item. (We'll do this interactively — for
now just note we need it, since program layout can differ by firmware.)

## Part B — Capture the SysEx dumps

Once a "minilogue" port exists, from `tools/explore/` with the venv active
(`pip install -r requirements.txt` first):

1. `python probe.py --list-ports` — report back the exact port name.
2. `python capture.py --port minilogue current -o captures/current_program.syx`
   Report back: the byte count it prints (e.g. "got 512 bytes").
3. `python unpack_inspect.py captures/current_program.syx`
   Report back: the "unpacked" byte count, the `marker[0:4]` value (should be
   `'PROG'`), and the `name` value.
4. `python capture.py --port minilogue global -o captures/global.syx`, then
   `unpack_inspect.py` on it. Report back the marker (should be `'GLOB'`) and
   size.
5. `python capture.py --port minilogue program 0 -o captures/program_000.syx`
   and `program 49` (a slot you've edited). Report back each byte count.

We want at least: current program, two stored slots (one factory, one you've
tweaked), one slot with a deliberately programmed note + motion sequence, and the
global dump.

## Part C — Get a real .mnlgprog / .mnlglib (for file-format round-tripping)

These come from the **Korg Sound Librarian**, not the synth.

1. Install the minilogue Sound Librarian and the KORG USB-MIDI driver from
   <https://www.korg.com/us/support/download/software/0/544/3083/>.
2. In the Librarian, export one program as a `.mnlgprog` and a small set as a
   `.mnlglib`. Report back the file paths and their byte sizes.
3. Alternatively, download a factory/bonus library from
   <https://www.korg.com/us/products/synthesizers/minilogue/librarian_contents.php>
   and report back the downloaded file's name and size.

Drop these into `minilogue-core/tests/fixtures/` (or hand the paths to the
assistant). Cross-reference for the internal layout: gazzar/loguetools.
