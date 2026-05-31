#!/usr/bin/env python3
"""Capture the current edit buffer and diff it against a saved baseline.

Used to discover which program byte/bit a given front-panel knob controls.

    python diff_param.py baseline          # snapshot current edit buffer
    python diff_param.py diff   <label>     # snapshot again, print byte/bit diffs

Reads the SysEx port (USB port 2). Non-destructive (dump requests only).
"""
import sys

import mido

from grab_fixtures import request
from probe import resolve

CURRENT = (0x10, 0x40)
BASELINE = "captures/_param_baseline.syx"


def unpack(wire):
    out = bytearray()
    i = 0
    while i < len(wire):
        msbs = wire[i]
        i += 1
        for j in range(7):
            if i >= len(wire):
                break
            b = wire[i]
            i += 1
            if (msbs >> j) & 1:
                b |= 0x80
            out.append(b)
    return bytes(out)


def grab():
    in_name = resolve(mido.get_input_names(), "MIDIIN2", "minilogue")
    out_name = resolve(mido.get_output_names(), "MIDIOUT2", "minilogue")
    with mido.open_input(in_name) as inp, mido.open_output(out_name) as out:
        raw = request(out, inp, CURRENT[0], [], CURRENT[1])
    if raw is None:
        print("no reply", file=sys.stderr)
        sys.exit(2)
    return unpack(raw[7:-1])  # 0x40 frame: payload starts at index 7


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "diff"
    if mode == "baseline":
        data = grab()
        with open(BASELINE, "wb") as f:
            f.write(data)
        print(f"baseline saved: {len(data)} unpacked bytes  ({BASELINE})")
        return

    label = sys.argv[2] if len(sys.argv) > 2 else "(unlabelled)"
    base = open(BASELINE, "rb").read()
    now = grab()
    print(f"diff for: {label}")
    changed = False
    for off in range(min(len(base), len(now))):
        if base[off] != now[off]:
            changed = True
            x, y = base[off], now[off]
            bits = " ".join(
                f"b{k}:{(x>>k)&1}->{(y>>k)&1}" for k in range(8) if (x >> k) & 1 != (y >> k) & 1
            )
            print(f"  offset {off:3d} (0x{off:02X}): {x:02X} -> {y:02X}   [{bits}]")
    if not changed:
        print("  (no change — did the knob move enough?)")


if __name__ == "__main__":
    main()
