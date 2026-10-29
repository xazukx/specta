//! Unsigned LEB128. A `varint` is capped at 32 bits (at most 5 bytes) and a `varint64` at
//! 64 bits (at most 10 bytes).

/// The maximum number of bytes a 32-bit varint may occupy.
pub(crate) const MAX_LEN: usize = 5;

/// Append `value` as a LEB128 varint.
pub(crate) fn write(out: &mut Vec<u8>, value: u32) {
    write64(out, value.into());
}

/// Append `value` as a LEB128 varint64.
pub(crate) fn write64(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

/// The number of bytes `value` takes as a varint or varint64.
pub(crate) fn len(value: u64) -> usize {
    (64 - (value | 1).leading_zeros() as usize).div_ceil(7)
}

/// Encode `value` into a stack buffer, returning the buffer and the number of bytes used.
pub(crate) fn encode(mut value: u32) -> ([u8; MAX_LEN], usize) {
    let mut buf = [0; MAX_LEN];
    let mut len = 0;
    while value >= 0x80 {
        buf[len] = (value as u8) | 0x80;
        value >>= 7;
        len += 1;
    }
    buf[len] = value as u8;
    (buf, len + 1)
}

/// Why a varint could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadError {
    /// The input ended before the final byte.
    Eof,
    /// Too many bytes, or a value that does not fit in the type.
    Overflow,
}

/// Read a varint from the start of `input`, returning the value and the number of bytes consumed.
pub(crate) fn read(input: &[u8]) -> Result<(u32, usize), ReadError> {
    // The 5th byte holds bits 28..32, so only its low nibble may be set.
    read_capped(input, MAX_LEN, 0x0F).map(|(value, len)| (value as u32, len))
}

/// Read a varint64 from the start of `input`, returning the value and the number of bytes consumed.
pub(crate) fn read64(input: &[u8]) -> Result<(u64, usize), ReadError> {
    // The 10th byte holds bit 63, so only its lowest bit may be set.
    read_capped(input, 10, 0x01)
}

/// Read at most `max_len` bytes. The last one must be at most `last_max`, which also means it
/// can't have its continuation bit set.
fn read_capped(input: &[u8], max_len: usize, last_max: u8) -> Result<(u64, usize), ReadError> {
    let mut value = 0u64;
    for (i, &byte) in input.iter().take(max_len).enumerate() {
        if i == max_len - 1 && byte > last_max {
            return Err(ReadError::Overflow);
        }
        value |= u64::from(byte & 0x7F) << (7 * i);
        if byte & 0x80 == 0 {
            return Ok((value, i + 1));
        }
    }
    Err(if input.len() < max_len {
        ReadError::Eof
    } else {
        ReadError::Overflow
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for value in [
            0,
            1,
            127,
            128,
            255,
            256,
            16_383,
            16_384,
            u32::MAX - 1,
            u32::MAX,
        ] {
            let mut out = Vec::new();
            write(&mut out, value);
            let (buf, len) = encode(value);
            assert_eq!(out, buf[..len]);
            assert_eq!(out.len(), super::len(value.into()));
            assert_eq!(read(&out), Ok((value, out.len())));
        }
        for value in [0, 1 << 49, (1 << 49) - 1, u64::from(u32::MAX) + 1, u64::MAX] {
            let mut out = Vec::new();
            write64(&mut out, value);
            assert_eq!(out.len(), len(value));
            assert_eq!(read64(&out), Ok((value, out.len())));
        }
    }

    #[test]
    fn rejects_malformed() {
        assert_eq!(read(&[]), Err(ReadError::Eof));
        assert_eq!(read(&[0x80, 0x80]), Err(ReadError::Eof));
        assert_eq!(
            read(&[0xFF, 0xFF, 0xFF, 0xFF, 0x10]),
            Err(ReadError::Overflow)
        );
        assert_eq!(
            read(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x00]),
            Err(ReadError::Overflow)
        );
        assert_eq!(read(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]), Ok((u32::MAX, 5)));

        let mut max = [0xFF; 10];
        max[9] = 0x01;
        assert_eq!(read64(&max), Ok((u64::MAX, 10)));
        max[9] = 0x02;
        assert_eq!(read64(&max), Err(ReadError::Overflow));
        assert_eq!(read64(&[0x80; 11]), Err(ReadError::Overflow));
        assert_eq!(read64(&[0x80; 9]), Err(ReadError::Eof));
    }
}
