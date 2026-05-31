#!/usr/bin/env python3
"""NON-DESTRUCTIVE write-semantics probe.

Learns how the minilogue treats inbound program dumps WITHOUT risking data:
  * snapshots the live edit buffer and slot 49 first,
  * sends slot 49's OWN captured bytes back to it (identical data — if stored,
    the slot is unchanged; if only loaded to the edit buffer, no slot is touched),
  * watches for ACK (0x23) / NAK (0x24) / format error (0x26),
  * re-reads slot 49 and the edit buffer to see what actually moved,
  * restores the edit buffer if it changed.

Nothing different is ever written to flash, so no patch can be lost.
"""
import time

import mido

from grab_fixtures import request
from probe import resolve

CURRENT = (0x10, 0x40)
PROGRAM = (0x1C, 0x4C)
STATUS = {0x23: "DATA LOAD COMPLETED (ACK)", 0x24: "DATA LOAD ERROR (NAK)", 0x26: "DATA FORMAT ERROR"}


def send_raw_and_listen(out, inputs, raw, ms=1500):
    """Send a full F0..F7 frame; print every reply seen on any input."""
    for _, inp in inputs:
        while inp.poll() is not None:
            pass
    out.send(mido.Message("sysex", data=list(raw[1:-1])))
    deadline = time.monotonic() + ms / 1000.0
    seen = []
    while time.monotonic() < deadline:
        for name, inp in inputs:
            m = inp.poll()
            if m is not None and m.type == "sysex":
                r = [0xF0] + list(m.data) + [0xF7]
                func = r[6] if len(r) > 6 else None
                label = STATUS.get(func, f"func 0x{func:02X}" if func is not None else "?")
                print(f"    reply on {name!r} ({len(r)} B): {label}")
                seen.append(r)
        time.sleep(0.001)
    if not seen:
        print("    (no reply)")
    return seen


def main():
    in_names = [n for n in mido.get_input_names() if "minilogue" in n.lower()]
    out_name = resolve(mido.get_output_names(), "MIDIOUT2", "minilogue")
    inputs = [(n, mido.open_input(n)) for n in in_names]
    out = mido.open_output(out_name)
    # the port-2 input is where dumps come back
    p2 = next((io for io in inputs if "midiin2" in io[0].lower()), inputs[0])

    print(f"out: {out_name}")
    print("\n[1] snapshot edit buffer + slot 49")
    base_edit = request(out, p2[1], CURRENT[0], [], CURRENT[1])
    base_49 = request(out, p2[1], PROGRAM[0], [49 & 0x7F, 0], PROGRAM[1])
    print(f"    edit buffer: {len(base_edit)} B   slot 49: {len(base_49)} B")

    print("\n[2] send slot 49's own bytes back to the device (identical data)")
    send_raw_and_listen(out, inputs, base_49)
    time.sleep(0.4)  # allow any flash write to settle

    print("\n[3] re-read slot 49 and edit buffer; compare")
    after_49 = request(out, p2[1], PROGRAM[0], [49 & 0x7F, 0], PROGRAM[1])
    after_edit = request(out, p2[1], CURRENT[0], [], CURRENT[1])
    print(f"    slot 49 unchanged:     {after_49 == base_49}")
    print(f"    edit buffer unchanged: {after_edit == base_edit}")

    if after_edit != base_edit:
        print("\n[4] edit buffer changed -> restoring original")
        send_raw_and_listen(out, inputs, base_edit)
        time.sleep(0.3)
        restored = request(out, p2[1], CURRENT[0], [], CURRENT[1])
        print(f"    edit buffer restored:  {restored == base_edit}")
    else:
        print("\n[4] edit buffer untouched -> nothing to restore")

    print("\n[5] does a CURRENT-program load (0x40) get ACKed? (edit buffer = its own bytes)")
    send_raw_and_listen(out, inputs, base_edit)

    out.close()
    for _, inp in inputs:
        inp.close()


if __name__ == "__main__":
    main()
