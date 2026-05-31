#!/usr/bin/env python3
"""Throwaway: find which port/channel elicits a CURRENT PROGRAM dump reply.

Opens every minilogue input at once, then sends the dump request on each
minilogue output, trying channel 1 (0x30) and broadcast (0x7F). Prints every
SysEx seen on any input, with the port it arrived on.
"""
import time

import mido

KORG, MODEL = 0x42, 0x2C


def hexs(bs):
    return " ".join(f"{b:02X}" for b in bs)


def req(chan_byte, func=0x10):
    return [KORG, chan_byte, 0x00, 0x01, MODEL, func]


in_names = [n for n in mido.get_input_names() if "minilogue" in n.lower()]
out_names = [n for n in mido.get_output_names() if "minilogue" in n.lower()]
print("inputs :", in_names)
print("outputs:", out_names)

inputs = [(n, mido.open_input(n)) for n in in_names]

for out_name in out_names:
    out = mido.open_output(out_name)
    for chan_byte in (0x30, 0x7F):
        for _, inp in inputs:
            while inp.poll() is not None:
                pass
        payload = req(chan_byte)
        print(f"\n--- send on {out_name!r} chan=0x{chan_byte:02X}: F0 {hexs(payload)} F7 ---")
        out.send(mido.Message("sysex", data=payload))
        deadline = time.monotonic() + 1.5
        got = False
        while time.monotonic() < deadline:
            for name, inp in inputs:
                m = inp.poll()
                if m is not None and m.type == "sysex":
                    raw = [0xF0] + list(m.data) + [0xF7]
                    print(f"  REPLY on {name!r} ({len(raw)} bytes): {hexs(raw[:10])} ...")
                    got = True
            time.sleep(0.001)
        if not got:
            print("  (no sysex)")
    out.close()

for _, inp in inputs:
    inp.close()
