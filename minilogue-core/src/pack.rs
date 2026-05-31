//! Korg 7→8 bit data conversion.
//!
//! Program and global payloads are transmitted as `7bit × 8` groups: for every
//! 7 source bytes, one leading byte carries their high bits (bit *j* = bit 7 of
//! source byte *j*, bit 0 = MSB of the first), followed by the 7 low-7-bit
//! bytes. The final group may be short. Request frames are *not* packed.
//!
//! Device-verified: see `../../docs/sysex-notes.md` §4.

/// Pack 8-bit data into the Korg 7-bit wire format.
pub fn pack(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + data.len() / 7 + 1);
    for chunk in data.chunks(7) {
        let mut msbs = 0u8;
        for (j, &b) in chunk.iter().enumerate() {
            if b & 0x80 != 0 {
                msbs |= 1 << j;
            }
        }
        out.push(msbs);
        for &b in chunk {
            out.push(b & 0x7F);
        }
    }
    out
}

/// Unpack the Korg 7-bit wire format back into 8-bit data.
pub fn unpack(wire: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(wire.len());
    let mut i = 0;
    while i < wire.len() {
        let msbs = wire[i];
        i += 1;
        for j in 0..7 {
            if i >= wire.len() {
                break;
            }
            let mut b = wire[i];
            i += 1;
            if msbs & (1 << j) != 0 {
                b |= 0x80;
            }
            out.push(b);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn empty_round_trips() {
        assert_eq!(pack(&[]), Vec::<u8>::new());
        assert_eq!(unpack(&[]), Vec::<u8>::new());
    }

    #[test]
    fn worked_example_prog_marker() {
        // From a real current-program dump: packed `00 50 52 4F 47 ...`
        // (MSB byte 0x00 -> no high bits) unpacks to "PROG".
        assert_eq!(&unpack(&[0x00, 0x50, 0x52, 0x4F, 0x47]), b"PROG");
    }

    #[test]
    fn full_group_high_bits() {
        let src = [0x81u8, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87];
        let wire = pack(&src);
        assert_eq!(wire[0], 0x7F); // all seven high bits set
        assert_eq!(unpack(&wire), src);
    }

    proptest! {
        #[test]
        fn unpack_is_left_inverse_of_pack(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
            prop_assert_eq!(unpack(&pack(&data)), data);
        }
    }
}
