//! Exact scalar representations used by typed HEADER semantics.

/// Exact IEEE-754 binary64 payload without `f64` equality or hashing ambiguity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDouble(u64);

impl DxfDouble {
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    #[must_use]
    pub fn from_f64(value: f64) -> Self {
        Self(value.to_bits())
    }

    #[must_use]
    pub const fn to_bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub fn to_f64(self) -> f64 {
        f64::from_bits(self.0)
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.to_f64().is_finite()
    }
}

/// Finite day value split at the integer boundary without calendar conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfDayParts {
    whole_days: i64,
    fractional_day: DxfDouble,
}

impl DxfDayParts {
    fn from_raw(raw: DxfDouble) -> Option<Self> {
        let value = raw.to_f64();
        let whole = value.trunc();
        const I64_MAX_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;
        if !value.is_finite() || whole < i64::MIN as f64 || whole >= I64_MAX_EXCLUSIVE {
            return None;
        }
        Some(Self {
            whole_days: whole as i64,
            fractional_day: DxfDouble::from_f64(value - whole),
        })
    }

    #[must_use]
    pub const fn whole_days(self) -> i64 {
        self.whole_days
    }

    /// Fraction following Autodesk's truncation-at-zero representation.
    #[must_use]
    pub const fn fractional_day(self) -> DxfDouble {
        self.fractional_day
    }
}

/// Exact DXF Julian-date scalar; no timezone or calendar interpretation is inferred.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfJulianDate(DxfDouble);

impl DxfJulianDate {
    #[must_use]
    pub const fn from_raw(raw: DxfDouble) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> DxfDouble {
        self.0
    }

    /// Splits a finite, `i64`-bounded scalar into whole and fractional days.
    #[must_use]
    pub fn day_parts(self) -> Option<DxfDayParts> {
        DxfDayParts::from_raw(self.0)
    }
}

/// Exact DXF elapsed-days scalar; it is not an absolute date or timezone value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfElapsedDays(DxfDouble);

impl DxfElapsedDays {
    #[must_use]
    pub const fn from_raw(raw: DxfDouble) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> DxfDouble {
        self.0
    }

    /// Splits a finite, `i64`-bounded scalar into whole and fractional days.
    #[must_use]
    pub fn day_parts(self) -> Option<DxfDayParts> {
        DxfDayParts::from_raw(self.0)
    }
}
