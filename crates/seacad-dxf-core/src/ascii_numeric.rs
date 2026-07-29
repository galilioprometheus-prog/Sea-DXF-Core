//! Strict ASCII numeric token parsing without DXF document concerns.

/// Exact failure while interpreting one ASCII numeric value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfAsciiNumericIssue {
    Empty,
    InvalidSyntax { token_offset: u64 },
    OutOfRange,
}

pub(crate) fn parse_i16(token: &[u8]) -> Result<i16, DxfAsciiNumericIssue> {
    let value = parse_signed_integer(token, i16::MAX as u64, (i16::MAX as u64) + 1)?;
    i16::try_from(value).map_err(|_| DxfAsciiNumericIssue::OutOfRange)
}

pub(crate) fn parse_i32(token: &[u8]) -> Result<i32, DxfAsciiNumericIssue> {
    let value = parse_signed_integer(token, i32::MAX as u64, (i32::MAX as u64) + 1)?;
    i32::try_from(value).map_err(|_| DxfAsciiNumericIssue::OutOfRange)
}

fn parse_signed_integer(
    token: &[u8],
    positive_limit: u64,
    negative_limit: u64,
) -> Result<i64, DxfAsciiNumericIssue> {
    if token.is_empty() {
        return Err(DxfAsciiNumericIssue::Empty);
    }
    let (negative, digits, offset) = match token[0] {
        b'+' => (false, &token[1..], 1_usize),
        b'-' => (true, &token[1..], 1_usize),
        _ => (false, token, 0_usize),
    };
    if digits.is_empty() {
        return Err(invalid_syntax(offset));
    }
    let mut magnitude = 0_u64;
    for (index, byte) in digits.iter().copied().enumerate() {
        if !byte.is_ascii_digit() {
            return Err(invalid_syntax(offset + index));
        }
        magnitude = magnitude
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(byte - b'0')))
            .ok_or(DxfAsciiNumericIssue::OutOfRange)?;
        let limit = if negative {
            negative_limit
        } else {
            positive_limit
        };
        if magnitude > limit {
            return Err(DxfAsciiNumericIssue::OutOfRange);
        }
    }
    let magnitude = i64::try_from(magnitude).map_err(|_| DxfAsciiNumericIssue::OutOfRange)?;
    Ok(if negative { -magnitude } else { magnitude })
}

pub(crate) fn parse_f64(token: &[u8]) -> Result<f64, DxfAsciiNumericIssue> {
    validate_double_syntax(token)?;
    let text = std::str::from_utf8(token).map_err(|_| invalid_syntax(0))?;
    let value = text
        .parse::<f64>()
        .map_err(|_| DxfAsciiNumericIssue::OutOfRange)?;
    if !value.is_finite() || (value == 0.0 && mantissa_has_nonzero_digit(token)) {
        return Err(DxfAsciiNumericIssue::OutOfRange);
    }
    Ok(value)
}

fn mantissa_has_nonzero_digit(token: &[u8]) -> bool {
    token
        .iter()
        .copied()
        .take_while(|byte| !matches!(byte, b'e' | b'E'))
        .any(|byte| matches!(byte, b'1'..=b'9'))
}

fn validate_double_syntax(token: &[u8]) -> Result<(), DxfAsciiNumericIssue> {
    if token.is_empty() {
        return Err(DxfAsciiNumericIssue::Empty);
    }
    let mut index = usize::from(matches!(token[0], b'+' | b'-'));
    let mut mantissa_digits = 0_usize;
    while matches!(token.get(index), Some(byte) if byte.is_ascii_digit()) {
        mantissa_digits += 1;
        index += 1;
    }
    if token.get(index) == Some(&b'.') {
        index += 1;
        while matches!(token.get(index), Some(byte) if byte.is_ascii_digit()) {
            mantissa_digits += 1;
            index += 1;
        }
    }
    if mantissa_digits == 0 {
        return Err(invalid_syntax(index));
    }
    if matches!(token.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(token.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent_start = index;
        while matches!(token.get(index), Some(byte) if byte.is_ascii_digit()) {
            index += 1;
        }
        if index == exponent_start {
            return Err(invalid_syntax(index));
        }
    }
    if index != token.len() {
        return Err(invalid_syntax(index));
    }
    Ok(())
}

fn invalid_syntax(offset: usize) -> DxfAsciiNumericIssue {
    match u64::try_from(offset) {
        Ok(token_offset) => DxfAsciiNumericIssue::InvalidSyntax { token_offset },
        Err(_) => DxfAsciiNumericIssue::OutOfRange,
    }
}

#[cfg(test)]
mod tests {
    use super::{DxfAsciiNumericIssue, parse_f64, parse_i16, parse_i32};

    #[test]
    fn parses_signed_i16_boundaries_and_reports_exact_failures() {
        assert_eq!(parse_i16(b"-32768"), Ok(i16::MIN));
        assert_eq!(parse_i16(b"+32767"), Ok(i16::MAX));
        assert_eq!(parse_i16(b""), Err(DxfAsciiNumericIssue::Empty));
        assert_eq!(
            parse_i16(b"12x"),
            Err(DxfAsciiNumericIssue::InvalidSyntax { token_offset: 2 })
        );
        assert_eq!(parse_i16(b"32768"), Err(DxfAsciiNumericIssue::OutOfRange));
    }

    #[test]
    fn parses_signed_i32_boundaries_and_reports_exact_failures() {
        assert_eq!(parse_i32(b"-2147483648"), Ok(i32::MIN));
        assert_eq!(parse_i32(b"+2147483647"), Ok(i32::MAX));
        assert_eq!(parse_i32(b""), Err(DxfAsciiNumericIssue::Empty));
        assert_eq!(
            parse_i32(b"12x"),
            Err(DxfAsciiNumericIssue::InvalidSyntax { token_offset: 2 })
        );
        assert_eq!(
            parse_i32(b"2147483648"),
            Err(DxfAsciiNumericIssue::OutOfRange)
        );
    }

    #[test]
    fn parses_f64_bits_and_rejects_invalid_or_non_finite_values() {
        assert_eq!(parse_f64(b"-0").map(f64::to_bits), Ok((-0.0_f64).to_bits()));
        assert_eq!(parse_f64(b"1.25e2"), Ok(125.0));
        assert_eq!(
            parse_f64(b"."),
            Err(DxfAsciiNumericIssue::InvalidSyntax { token_offset: 1 })
        );
        assert_eq!(
            parse_f64(b"1e"),
            Err(DxfAsciiNumericIssue::InvalidSyntax { token_offset: 2 })
        );
        assert_eq!(parse_f64(b"1e9999"), Err(DxfAsciiNumericIssue::OutOfRange));
        assert_eq!(parse_f64(b"1e-9999"), Err(DxfAsciiNumericIssue::OutOfRange));
    }
}
