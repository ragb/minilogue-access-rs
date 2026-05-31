#!/usr/bin/env python3
"""Send a Korg minilogue SysEx function frame and print any replies.

Frame on the wire:  F0 42 3g 00 01 2C <function> [data...] F7
where g is the MIDI channel nibble (channel 1 -> 0x30). There is NO checksum.

Examples:
    python probe.py --list-ports
    python probe.py --port minilogue --func 0x10                # current program dump request
    python probe.py --port minilogue --func 0x1C --data 00 00   # program 0 dump request (pp PP)
    python probe.py --port minilogue --func 0x0E                # global dump request
    python probe.py --port minilogue --func 0x10 --save captures/current.syx
"""
import argparse
import sys
import time

import mido

KORG_ID = 0x42
MODEL_ID = 0x2C
HEADER = [0x00, 0x01, MODEL_ID]  # format(2) + model id; function byte follows.


def channel_byte(ch1: int) -> int:
    """1-based UI channel -> wire nibble. Channel 1 -> 0x30."""
    return 0x30 | ((ch1 - 1) & 0x0F)


def build(func: int, data, ch1: int = 1):
    """SysEx *payload* (between F0 and F7) for mido.Message(type='sysex')."""
    return [KORG_ID, channel_byte(ch1)] + HEADER + [func] + list(data)


def hexs(bs) -> str:
    return " ".join(f"{b:02X}" for b in bs)


def find_port(names, needle):
    needle = needle.lower()
    for n in names:
        if needle in n.lower():
            return n
    return None


def resolve(names, *needles):
    """First port whose name contains one of the needles (substring), in order."""
    for nd in needles:
        if nd:
            p = find_port(names, nd)
            if p:
                return p
    return None


def listen(inp, ms):
    out, deadline = [], time.monotonic() + ms / 1000.0
    while time.monotonic() < deadline:
        m = inp.poll()
        if m is None:
            time.sleep(0.001)
            continue
        out.append(m)
    return out


def main():
    ap = argparse.ArgumentParser(description="Korg minilogue SysEx prober")
    ap.add_argument("--list-ports", action="store_true")
    ap.add_argument("--port", help="substring matched against both in and out port names")
    ap.add_argument("--in-port")
    ap.add_argument("--out-port")
    ap.add_argument("--channel", type=int, default=1, help="1-based MIDI channel (default 1)")
    ap.add_argument("--func", type=lambda x: int(x, 0), help="function code, e.g. 0x10")
    ap.add_argument("--data", nargs="*", default=[], help="extra data bytes, e.g. 00 00")
    ap.add_argument("--timeout-ms", type=int, default=1000)
    ap.add_argument("--save", help="write first SysEx reply to this .syx file")
    args = ap.parse_args()

    if args.list_ports:
        print("Inputs:")
        for n in mido.get_input_names():
            print("  ", n)
        print("Outputs:")
        for n in mido.get_output_names():
            print("  ", n)
        return

    if args.func is None:
        ap.error("--func is required (or use --list-ports)")

    # Default to USB port 2 (MIDIOUT2/MIDIIN2) where minilogue bulk SysEx lives.
    in_name = resolve(mido.get_input_names(), args.in_port, args.port, "MIDIIN2", "minilogue")
    out_name = resolve(mido.get_output_names(), args.out_port, args.port, "MIDIOUT2", "minilogue")
    if not in_name or not out_name:
        print("Could not find MIDI ports. Run with --list-ports.", file=sys.stderr)
        sys.exit(1)
    print(f"in:  {in_name}")
    print(f"out: {out_name}")

    data = [int(x, 0) for x in args.data]
    payload = build(args.func, data, args.channel)
    print("send:", "F0", hexs(payload), "F7")

    with mido.open_input(in_name) as inp, mido.open_output(out_name) as out:
        while inp.poll() is not None:  # drain stale input
            pass
        out.send(mido.Message("sysex", data=payload))
        replies = [m for m in listen(inp, args.timeout_ms) if m.type == "sysex"]

    if not replies:
        print("no SysEx reply within timeout.")
        return
    for i, m in enumerate(replies):
        raw = [0xF0] + list(m.data) + [0xF7]
        print(f"reply[{i}] ({len(raw)} bytes): {hexs(raw)}")
    if args.save:
        raw = bytes([0xF0] + list(replies[0].data) + [0xF7])
        with open(args.save, "wb") as f:
            f.write(raw)
        print(f"saved {len(raw)} bytes -> {args.save}")


if __name__ == "__main__":
    main()
