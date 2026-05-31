#!/usr/bin/env python3
"""Capture the standard minilogue fixture dumps with friendly filenames.

Examples:
    python capture.py --list-ports
    python capture.py --port minilogue current    -o captures/current_program.syx
    python capture.py --port minilogue global     -o captures/global.syx
    python capture.py --port minilogue program 0  -o captures/program_000.syx
    python capture.py --port minilogue program 49 -o captures/program_049.syx

`program N` uses N in 0..199. The on-screen program shown as "1" is N=0.
"""
import argparse
import sys

import mido

from probe import build, find_port, hexs, listen


def resolve(names, *needles):
    """First port whose name contains one of the needles (substring), in order."""
    for nd in needles:
        if nd:
            p = find_port(names, nd)
            if p:
                return p
    return None

# (request function code, expected reply function code)
CURRENT = (0x10, 0x40)
GLOBAL = (0x0E, 0x51)
PROGRAM = (0x1C, 0x4C)


def program_number_bytes(n: int):
    """0..199 -> [pp, PP]: pp = low 7 bits, PP = bit 7."""
    return [n & 0x7F, (n >> 7) & 0x01]


def capture(in_name, out_name, channel, func, data, reply_func, timeout_ms):
    payload = build(func, data, channel)
    print("send:", "F0", hexs(payload), "F7")
    with mido.open_input(in_name) as inp, mido.open_output(out_name) as out:
        while inp.poll() is not None:
            pass
        out.send(mido.Message("sysex", data=payload))
        for m in listen(inp, timeout_ms):
            if m.type != "sysex":
                continue
            raw = [0xF0] + list(m.data) + [0xF7]
            # reply function byte is index 6 (F0 42 3g 00 01 2C <func> ...)
            if len(raw) > 6 and raw[6] == reply_func:
                return bytes(raw)
    return None


def main():
    ap = argparse.ArgumentParser(description="Capture minilogue fixture dumps")
    ap.add_argument("--list-ports", action="store_true")
    ap.add_argument("--port")
    ap.add_argument("--in-port")
    ap.add_argument("--out-port")
    ap.add_argument("--channel", type=int, default=1)
    ap.add_argument("--timeout-ms", type=int, default=2000)
    ap.add_argument("-o", "--out", help="output .syx path")
    ap.add_argument("what", nargs="?", choices=["current", "global", "program"])
    ap.add_argument("number", nargs="?", type=int, help="program number 0..199 (for `program`)")
    args = ap.parse_args()

    if args.list_ports:
        print("Inputs:")
        for n in mido.get_input_names():
            print("  ", n)
        print("Outputs:")
        for n in mido.get_output_names():
            print("  ", n)
        return

    if not args.what:
        ap.error("specify one of: current | global | program N  (or --list-ports)")

    # minilogue bulk SysEx travels over USB port 2 (MIDIOUT2/MIDIIN2). Default
    # there, falling back to the plain "minilogue" port. Explicit flags win.
    in_name = resolve(mido.get_input_names(), args.in_port, args.port, "MIDIIN2", "minilogue")
    out_name = resolve(mido.get_output_names(), args.out_port, args.port, "MIDIOUT2", "minilogue")
    if not in_name or not out_name:
        print("Could not find MIDI ports. Run with --list-ports.", file=sys.stderr)
        sys.exit(1)
    print(f"in:  {in_name}\nout: {out_name}")

    if args.what == "current":
        func, reply = CURRENT
        data = []
    elif args.what == "global":
        func, reply = GLOBAL
        data = []
    else:
        if args.number is None:
            ap.error("`program` needs a number 0..199")
        func, reply = PROGRAM
        data = program_number_bytes(args.number)

    raw = capture(in_name, out_name, args.channel, func, data, reply, args.timeout_ms)
    if raw is None:
        print(f"no reply with function 0x{reply:02X} within timeout.")
        sys.exit(2)
    print(f"got {len(raw)} bytes (reply func 0x{reply:02X})")
    if args.out:
        with open(args.out, "wb") as f:
            f.write(raw)
        print(f"saved -> {args.out}")
    else:
        print(hexs(raw))


if __name__ == "__main__":
    main()
