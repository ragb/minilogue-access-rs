#!/usr/bin/env python3
"""Probe device identity. Read-only — sends inquiry messages, never writes.

Tries:
  1. Universal Identity Request: F0 7E 7F 06 01 F7  (reply F0 7E cc 06 02 ...).
Sends on every minilogue output, listens on every minilogue input.
"""
import time

import mido

from probe import hexs


def main():
    in_names = [n for n in mido.get_input_names() if "minilogue" in n.lower()]
    out_names = [n for n in mido.get_output_names() if "minilogue" in n.lower()]
    print("inputs :", in_names)
    print("outputs:", out_names)

    inputs = [(n, mido.open_input(n)) for n in in_names]
    inquiry = [0x7E, 0x7F, 0x06, 0x01]  # payload between F0 and F7

    for out_name in out_names:
        out = mido.open_output(out_name)
        for _, inp in inputs:
            while inp.poll() is not None:
                pass
        print(f"\n--- Universal Identity Request on {out_name!r}: F0 {hexs(inquiry)} F7 ---")
        out.send(mido.Message("sysex", data=inquiry))
        deadline = time.monotonic() + 1.5
        got = False
        while time.monotonic() < deadline:
            for name, inp in inputs:
                m = inp.poll()
                if m is not None and m.type == "sysex":
                    raw = [0xF0] + list(m.data) + [0xF7]
                    print(f"  REPLY on {name!r} ({len(raw)} bytes): {hexs(raw)}")
                    got = True
            time.sleep(0.001)
        if not got:
            print("  (no reply)")
        out.close()

    for _, inp in inputs:
        inp.close()


if __name__ == "__main__":
    main()
