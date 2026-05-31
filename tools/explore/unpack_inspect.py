#!/usr/bin/env python3
"""Decode a captured minilogue .syx dump: strip framing, 7->8 unpack, print bytes.

Usage:
    python unpack_inspect.py captures/current_program.syx

Korg 7->8 unpack: data arrives in groups of 8 wire bytes. The first byte of each
group carries the MSBs (bit j -> bit7 of data byte j); the next up to 7 bytes are
the low 7 bits.
"""
import sys


def unpack(wire: bytes) -> bytes:
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


def main():
    if len(sys.argv) < 2:
        print("usage: unpack_inspect.py FILE.syx")
        sys.exit(1)

    raw = open(sys.argv[1], "rb").read()
    if not raw or raw[0] != 0xF0 or raw[-1] != 0xF7:
        print("not a SysEx frame (missing F0/F7)")
        sys.exit(1)

    print(f"frame:  {len(raw)} bytes")
    print("header:", " ".join(f"{b:02X}" for b in raw[:7]))
    func = raw[6]
    print(f"function: 0x{func:02X}")

    # Framing: F0 42 3g 00 01 2C <func> [pp PP] <packed...> F7.
    # PROGRAM DATA DUMP (0x4C) inserts a 2-byte program number before the payload.
    payload_start = 7
    if func == 0x4C:
        prog = raw[7] | (raw[8] << 7)
        print(f"program number: {prog}  (pp PP = {raw[7]:02X} {raw[8]:02X})")
        payload_start = 9
    packed = raw[payload_start:-1]
    print(f"packed payload: {len(packed)} bytes")
    data = unpack(packed)
    print(f"unpacked:       {len(data)} bytes")

    marker = "".join(chr(b) if 32 <= b < 127 else "." for b in data[:4])
    print(f"marker[0:4]:  {data[:4].hex(' ').upper()}  ({marker!r})")
    name = data[4:16].split(b"\x00")[0].decode("ascii", "replace")
    print(f"name[4:16]:   {name!r}")
    print()

    for off in range(0, len(data), 16):
        row = data[off : off + 16]
        hexpart = " ".join(f"{b:02X}" for b in row)
        asc = "".join(chr(b) if 32 <= b < 127 else "." for b in row)
        print(f"{off:04X}  {hexpart:<47}  {asc}")


if __name__ == "__main__":
    main()
