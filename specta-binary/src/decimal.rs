//! `Decimal` values: a float as `±mantissa × 10^exponent`.
//!
//! Writers use the shortest mantissa that reads back as the same float, choosing the even one
//! when two are equally close (like JavaScript's `Number.prototype.toString()`), and fall back to
//! `Float64` when the decimal would take more than [`FLOAT64_LEN`] bytes. Two shortest
//! mantissas can only tie with 16 or more digits, which always falls back, so for `f64` Rust's
//! `{:e}` and JavaScript's `String(value)` agree on every `Decimal` they write.

use std::{
    fmt::{self, Write as _},
    str::FromStr,
};

use crate::varint;

/// The size of a `Float64` value: the tag and 8 bytes. Writers use `Decimal` when it is no larger.
pub(crate) const FLOAT64_LEN: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Decimal {
    pub(crate) negative: bool,
    pub(crate) mantissa: u64,
    pub(crate) exponent: i32,
}

impl Decimal {
    /// The shortest decimal that reads back as `value` (an `f32` or `f64`), or `None` for
    /// `NaN` and `±Infinity`.
    pub(crate) fn shortest<F: fmt::LowerExp + Into<f64> + Copy>(value: F) -> Option<Self> {
        let exact: f64 = value.into();
        if !exact.is_finite() {
            return None;
        }
        // `{:e}` gives the shortest digits, like `-1.2345e-7`.
        let mut sci = Buf::default();
        write!(sci, "{value:e}").ok()?;
        let (digits, exponent) = sci.as_str().trim_start_matches('-').split_once('e')?;
        let (int, fraction) = digits.split_once('.').unwrap_or((digits, ""));

        let mut decimal = Self {
            negative: exact.is_sign_negative(),
            // At most 17 digits, so this can't overflow.
            mantissa: int
                .bytes()
                .chain(fraction.bytes())
                .fold(0, |mantissa, digit| mantissa * 10 + u64::from(digit - b'0')),
            exponent: exponent.parse::<i32>().ok()? - fraction.len() as i32,
        };
        decimal.round_ties_to_even(exact.abs());
        Some(decimal)
    }

    /// `{:e}` breaks an exact tie between two shortest mantissas away from zero, where
    /// JavaScript and `serde_json` pick the even one. Only `f32` gets here with a tie: an `f64`
    /// tie needs 16 or more digits, which is written as `Float64`.
    fn round_ties_to_even(&mut self, exact: f64) {
        if self.mantissa.is_multiple_of(2) {
            return;
        }
        for (neighbor, midpoint) in [
            (self.mantissa - 1, 2 * self.mantissa - 1),
            (self.mantissa + 1, 2 * self.mantissa + 1),
        ] {
            // Halfway to the neighbor is `midpoint / 2 × 10^exponent`.
            if is_exactly(exact, u128::from(midpoint) * 5, self.exponent - 1) {
                self.mantissa = neighbor;
                return;
            }
        }
    }

    /// The `varint` written before the mantissa: the zigzag-encoded exponent, then the sign bit.
    pub(crate) fn head(&self) -> u32 {
        let zigzag = ((self.exponent << 1) ^ (self.exponent >> 31)) as u32;
        (zigzag << 1) | u32::from(self.negative)
    }

    pub(crate) fn from_head(head: u32, mantissa: u64) -> Self {
        let zigzag = head >> 1;
        Self {
            negative: head & 1 == 1,
            mantissa,
            exponent: (zigzag >> 1) as i32 ^ -((zigzag & 1) as i32),
        }
    }

    /// The size on the wire, including the tag.
    pub(crate) fn encoded_len(&self) -> usize {
        1 + varint::len(self.head().into()) + varint::len(self.mantissa)
    }

    /// The nearest `f32` or `f64` to the exact decimal.
    pub(crate) fn to_float<T: FromStr + Default>(self) -> T {
        let mut text = Buf::default();
        let sign = if self.negative { "-" } else { "" };
        // Fits in `Buf`, so this can't fail, and the standard parser rounds correctly.
        match write!(text, "{sign}{}e{}", self.mantissa, self.exponent) {
            Ok(()) => text.as_str().parse().unwrap_or_default(),
            Err(_) => T::default(),
        }
    }
}

/// Whether the finite, non-negative `value` is exactly `odd × 10^exponent`, for an odd `odd`.
fn is_exactly(value: f64, odd: u128, exponent: i32) -> bool {
    // `value` is `mantissa × 2^power`.
    let bits = value.to_bits();
    let fraction = bits & ((1 << 52) - 1);
    let (mantissa, power) = match (bits >> 52) as i32 {
        0 => (fraction, -1074),
        biased => (fraction | (1 << 52), biased - 1075),
    };
    if mantissa == 0 {
        return false;
    }
    let zeros = mantissa.trailing_zeros();
    let (mantissa, power) = (u128::from(mantissa >> zeros), power + zeros as i32);

    // With both sides written as an odd number times a power of two (10^e = 5^e × 2^e),
    // the odd numbers and the powers must each match.
    let pow5 = 5u128.checked_pow(exponent.unsigned_abs());
    power == exponent
        && match exponent >= 0 {
            true => pow5.and_then(|p| odd.checked_mul(p)) == Some(mantissa),
            false => pow5.and_then(|p| mantissa.checked_mul(p)) == Some(odd),
        }
}

/// A stack buffer for formatting, large enough for `-1.2345678901234567e-308` and
/// `-18446744073709551615e-1073741824`.
struct Buf {
    bytes: [u8; 40],
    len: usize,
}

impl Default for Buf {
    fn default() -> Self {
        Self {
            bytes: [0; 40],
            len: 0,
        }
    }
}

impl Buf {
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).unwrap_or_default()
    }
}

impl fmt::Write for Buf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        self.bytes
            .get_mut(self.len..end)
            .ok_or(fmt::Error)?
            .copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(value: impl fmt::LowerExp + Into<f64> + Copy) -> (bool, u64, i32) {
        let d = Decimal::shortest(value).expect("finite");
        (d.negative, d.mantissa, d.exponent)
    }

    #[test]
    fn shortest_mantissas() {
        for (value, expected) in [
            (1.5, (false, 15, -1)),
            (0.1, (false, 1, -1)),
            (-0.25, (true, 25, -2)),
            (100.0, (false, 1, 2)),
            (19.99, (false, 1999, -2)),
            (1e21, (false, 1, 21)),
            (-1e-7, (true, 1, -7)),
            (5e-324, (false, 5, -324)),
            (0.1 + 0.2, (false, 30_000_000_000_000_004, -17)),
            (f64::MAX, (false, 17_976_931_348_623_157, 292)),
            (0.0, (false, 0, 0)),
            (-0.0, (true, 0, 0)),
        ] {
            assert_eq!(parts(value), expected, "{value:e}");
        }
        assert_eq!(parts(0.1f32), (false, 1, -1));
        assert_eq!(parts(f32::MAX), (false, 34_028_235, 31));
        assert_eq!(Decimal::shortest(f64::NAN), None);
        assert_eq!(Decimal::shortest(f32::NEG_INFINITY), None);
    }

    #[test]
    fn ties_go_to_the_even_mantissa() {
        // `-522429.125f32` is exactly halfway between `-522429.12` and `-522429.13`.
        assert_eq!(parts(-(522_429.0f32 + 0.125)), (true, 52_242_912, -2));
        assert_eq!(parts(360_073.0f32 + 0.125), (false, 36_007_312, -2));
        // `f64` ties have at least 16 digits.
        assert_eq!(
            parts(2f64.powi(50) + 0.25),
            (false, 11_258_999_068_426_242, -1)
        );
        assert_eq!(
            parts(2f64.powi(50) + 0.75),
            (false, 11_258_999_068_426_248, -1)
        );
        // Not ties: the nearest shortest mantissa wins.
        assert_eq!(
            parts(2f64.powi(49) + 0.125),
            (false, 5_629_499_534_213_121, -1)
        );
        assert_eq!(parts(522_429.0f32 + 0.15625), (false, 52_242_916, -2));
    }

    #[test]
    fn heads_round_trip() {
        for (negative, exponent) in [
            (false, 0),
            (true, 0),
            (false, -1),
            (true, 21),
            (false, -340),
            (true, 308),
        ] {
            let decimal = Decimal {
                negative,
                mantissa: 7,
                exponent,
            };
            assert_eq!(Decimal::from_head(decimal.head(), 7), decimal);
        }
        // 1.5 is 3 bytes: the tag, the head and the mantissa.
        assert_eq!(
            Decimal::shortest(1.5).map(|d| (d.head(), d.encoded_len())),
            Some((2, 3))
        );
        assert_eq!(Decimal::from_head(u32::MAX, u64::MAX).exponent, -(1 << 30));
    }

    #[test]
    fn reads_back_correctly_rounded() {
        let read = |negative, mantissa, exponent| Decimal {
            negative,
            mantissa,
            exponent,
        };
        assert_eq!(read(false, 15, -1).to_float::<f64>(), 1.5);
        assert_eq!(read(false, 1, -1).to_float::<f32>(), 0.1);
        assert!(read(true, 0, 0).to_float::<f64>().is_sign_negative());
        assert_eq!(read(false, u64::MAX, 400).to_float::<f64>(), f64::INFINITY);
        assert_eq!(read(false, u64::MAX, -(1 << 30)).to_float::<f64>(), 0.0);
        // Parsed straight into an `f32`: going through an `f64` would round twice and give 1.0.
        assert_eq!(
            read(false, 10_000_000_596_046_448, -16).to_float::<f32>(),
            1.000_000_1
        );
    }
}
