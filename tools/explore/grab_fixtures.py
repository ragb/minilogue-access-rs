#!/usr/bin/env python3
"""Capture the standard minilogue fixture set in ONE process.

Opens the SysEx port (USB port 2) once and issues each dump request in turn —
avoids the Windows MME stall when the same MIDI port is closed and reopened in
quick succession. Saves each reply to captures/.

Usage:
    python grab_fixtures.py                 # current, global, program 0 and 49
    python grab_fixtures.py 0 49 100 150    # current, global, + those programs
"""
import os
import sys
import time

import mido

from probe import build, hexs, resolve

OUT_DIR = "captures"
CURRENT = (0x10, 0x40)
GLOBAL = (0x0E, 0x51)
PROGRAM = (0x1C, 0x4C)


def request(out, inp, func, data, reply_func, timeout_ms=2500):
    while inp.poll() is not None:
        pass
    out.send(mido.Message("sysex", data=build(func, data)))
    deadline = time.monotonic() + timeout_ms / 1000.0
    while time.monotonic() < deadline:
        m = inp.poll()
        if m is not None and m.type == "sysex":
            raw = [0xF0] + list(m.data) + [0xF7]
            if len(raw) > 6 and raw[6] == reply_func:
                return bytes(raw)
        time.sleep(0.001)
    return None


def save(name, raw):
    path = os.path.join(OUT_DIR, name)
    if raw is None:
        print(f"  {name}: NO REPLY")
        return
    with open(path, "wb") as f:
        f.write(raw)
    print(f"  {name}: {len(raw)} bytes (func 0x{raw[6]:02X}) -> {path}")


def main():
    progs = [int(x, 0) for x in sys.argv[1:]] or [0, 49]
    os.makedirs(OUT_DIR, exist_ok=True)

    in_name = resolve(mido.get_input_names(), "MIDIIN2", "minilogue")
    out_name = resolve(mido.get_output_names(), "MIDIOUT2", "minilogue")
    if not in_name or not out_name:
        print("no minilogue ports found", file=sys.stderr)
        sys.exit(1)
    print(f"in:  {in_name}\nout: {out_name}\n")

    with mido.open_input(in_name) as inp, mido.open_output(out_name) as out:
        save("current_program.syx", request(out, inp, CURRENT[0], [], CURRENT[1]))
        save("global.syx", request(out, inp, GLOBAL[0], [], GLOBAL[1]))
        for n in progs:
            data = [n & 0x7F, (n >> 7) & 0x01]
            save(f"program_{n:03d}.syx", request(out, inp, PROGRAM[0], data, PROGRAM[1]))


if __name__ == "__main__":
    main()
