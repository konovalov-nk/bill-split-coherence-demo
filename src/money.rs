//! EUR amounts as integer minor units (cents).
//!
//! Spec: `SPEC-eur-minor-units`.
//! Floating point never enters the domain model: parsing and formatting are
//! exact decimal <-> integer conversions.

use std::fmt;

/// Errors produced by exact euro parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoneyError {
    /// Input was empty after trimming.
    Empty,
    /// Input was not a plain decimal amount.
    InvalidFormat,
    /// Input had more than two fractional digits.
    SubCentPrecision,
    /// Negative amounts are outside this domain.
    Negative,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::Empty => "amount is empty",
            Self::InvalidFormat => "amount is not a plain decimal number",
            Self::SubCentPrecision => "amount has sub-cent precision",
            Self::Negative => "amount is negative",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for MoneyError {}

/// Parse a euro decimal string into integer cents without floating point.
///
/// `"1000.00" -> 100000`, `"333.33" -> 33333`, `"0.01" -> 1`.
pub fn parse_eur_to_cents(input: &str) -> Result<i64, MoneyError> {
    let trimmed = input.trim();
    let trimmed = trimmed.strip_suffix('€').unwrap_or(trimmed).trim();
    if trimmed.is_empty() {
        return Err(MoneyError::Empty);
    }

    let (negative, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed),
    };

    let mut parts = digits.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next();
    if parts.next().is_some() {
        return Err(MoneyError::InvalidFormat);
    }
    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
        return Err(MoneyError::InvalidFormat);
    }

    let whole_cents = whole
        .parse::<i64>()
        .map_err(|_| MoneyError::InvalidFormat)?
        .checked_mul(100)
        .ok_or(MoneyError::InvalidFormat)?;

    let fraction_cents = match fraction {
        None => 0,
        Some("") => return Err(MoneyError::InvalidFormat),
        Some(frac) => {
            if !frac.chars().all(|c| c.is_ascii_digit()) {
                return Err(MoneyError::InvalidFormat);
            }
            match frac.len() {
                1 => frac.parse::<i64>().map_err(|_| MoneyError::InvalidFormat)? * 10,
                2 => frac.parse::<i64>().map_err(|_| MoneyError::InvalidFormat)?,
                _ => return Err(MoneyError::SubCentPrecision),
            }
        }
    };

    let cents = whole_cents
        .checked_add(fraction_cents)
        .ok_or(MoneyError::InvalidFormat)?;

    if negative {
        if cents == 0 {
            Ok(0)
        } else {
            Err(MoneyError::Negative)
        }
    } else {
        Ok(cents)
    }
}

/// Format integer cents as a two-decimal euro string (`33334 -> "333.34"`).
#[must_use]
pub fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
    format!("{sign}{}.{:02}", abs / 100, abs % 100)
}
