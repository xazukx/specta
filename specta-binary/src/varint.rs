//! Unsigned LEB128, capped at 32 bits (at most 5 bytes).

/// The maximum number of bytes a varint may occupy.
pub(crate) const MAX_LEN: usize = 5;

/// Append `value` as a LEB128 varint.
pub(crate) fn write(out: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
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
    /// More than 5 bytes, or a value that does not fit in 32 bits.
    Overflow,
}

/// Read a varint from the start of `input`, returning the value and the number of bytes consumed.
pub(crate) fn read(input: &[u8]) -> Result<(u32, usize), ReadError> {
    let mut value = 0u32;
    for (i, &byte) in input.iter().take(MAX_LEN).enumerate() {
        // The 5th byte holds bits 28..32, so only its low nibble may be set and it must be the last.
        if i == MAX_LEN - 1 && byte > 0x0F {
            return Err(ReadError::Overflow);
        }
        value |= u32::from(byte & 0x7F) << (7 * i);
        if byte & 0x80 == 0 {
            return Ok((value, i + 1));
        }
    }
    Err(if input.len() < MAX_LEN {
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
        for value in [0, 1, 127, 128, 255, 256, 16_383, 16_384, u32::MAX - 1, u32::MAX] {
            let mut out = Vec::new();
            write(&mut out, value);
            let (buf, len) = encode(value);
            assert_eq!(out, buf[..len]);
            assert_eq!(read(&out), Ok((value, out.len())));
        }
    }

    #[test]
    fn rejects_malformed() {
        assert_eq!(read(&[]), Err(ReadError::Eof));
        assert_eq!(read(&[0x80, 0x80]), Err(ReadError::Eof));
        assert_eq!(read(&[0xFF, 0xFF, 0xFF, 0xFF, 0x10]), Err(ReadError::Overflow));
        assert_eq!(read(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x00]), Err(ReadError::Overflow));
        assert_eq!(read(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]), Ok((u32::MAX, 5)));
    }
}
