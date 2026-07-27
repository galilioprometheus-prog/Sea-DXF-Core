use std::fmt;

use crate::{ByteSpan, DxfSourceId};

/// Stable identity of the document and schema field behind one semantic value.
///
/// Namespace and field identifiers point at static, generated schema metadata;
/// they do not duplicate normative source receipts in every cached value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfSemanticFieldProvenance {
    document_source_id: DxfSourceId,
    schema_namespace: &'static str,
    schema_field_id: &'static str,
}

impl DxfSemanticFieldProvenance {
    #[must_use]
    pub const fn new(
        document_source_id: DxfSourceId,
        schema_namespace: &'static str,
        schema_field_id: &'static str,
    ) -> Self {
        Self {
            document_source_id,
            schema_namespace,
            schema_field_id,
        }
    }

    #[must_use]
    pub const fn document_source_id(self) -> DxfSourceId {
        self.document_source_id
    }

    #[must_use]
    pub const fn schema_namespace(self) -> &'static str {
        self.schema_namespace
    }

    #[must_use]
    pub const fn schema_field_id(self) -> &'static str {
        self.schema_field_id
    }
}

/// Compact location of one raw group value in the authoritative source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfRawValueProvenance {
    group_occurrence: u32,
    value_span: ByteSpan,
}

impl DxfRawValueProvenance {
    /// Creates raw provenance if the occurrence fits SeaCad's bounded index.
    #[must_use]
    pub fn new(group_occurrence: u64, value_span: ByteSpan) -> Option<Self> {
        let group_occurrence = u32::try_from(group_occurrence).ok()?;
        Some(Self {
            group_occurrence,
            value_span,
        })
    }

    #[must_use]
    pub const fn group_occurrence(self) -> u64 {
        self.group_occurrence as u64
    }

    #[must_use]
    pub const fn value_span(self) -> ByteSpan {
        self.value_span
    }
}

/// Meaning of a typed semantic field without conflating absence and failure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSemanticValueState {
    /// The source contains the value explicitly.
    Explicit,
    /// The value comes from a reviewed schema default, not invented source bytes.
    Defaulted,
    /// The value is not present and no default applies.
    Absent,
    /// The field was present or required but could not be interpreted exactly.
    Invalid,
}

/// Source-anchored semantic value with an explicit four-state contract.
///
/// `T` is the usable value type and `I` is a typed invalidity reason. Raw bytes
/// remain authoritative; this type stores only derived values and provenance.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfSemanticValue<T, I> {
    Explicit {
        value: T,
        field: DxfSemanticFieldProvenance,
        raw: DxfRawValueProvenance,
    },
    Defaulted {
        value: T,
        field: DxfSemanticFieldProvenance,
    },
    Absent {
        field: DxfSemanticFieldProvenance,
    },
    Invalid {
        issue: I,
        field: DxfSemanticFieldProvenance,
        raw: Option<DxfRawValueProvenance>,
    },
}

impl<T, I> DxfSemanticValue<T, I> {
    #[must_use]
    pub fn explicit(
        value: T,
        field: DxfSemanticFieldProvenance,
        raw: DxfRawValueProvenance,
    ) -> Self {
        Self::Explicit { value, field, raw }
    }

    #[must_use]
    pub fn defaulted(value: T, field: DxfSemanticFieldProvenance) -> Self {
        Self::Defaulted { value, field }
    }

    #[must_use]
    pub fn absent(field: DxfSemanticFieldProvenance) -> Self {
        Self::Absent { field }
    }

    #[must_use]
    pub fn invalid(
        issue: I,
        field: DxfSemanticFieldProvenance,
        raw: Option<DxfRawValueProvenance>,
    ) -> Self {
        Self::Invalid { issue, field, raw }
    }

    #[must_use]
    pub const fn state(&self) -> DxfSemanticValueState {
        match self {
            Self::Explicit { .. } => DxfSemanticValueState::Explicit,
            Self::Defaulted { .. } => DxfSemanticValueState::Defaulted,
            Self::Absent { .. } => DxfSemanticValueState::Absent,
            Self::Invalid { .. } => DxfSemanticValueState::Invalid,
        }
    }

    #[must_use]
    pub const fn field_provenance(&self) -> DxfSemanticFieldProvenance {
        match self {
            Self::Explicit { field, .. }
            | Self::Defaulted { field, .. }
            | Self::Absent { field }
            | Self::Invalid { field, .. } => *field,
        }
    }

    #[must_use]
    pub const fn raw_provenance(&self) -> Option<DxfRawValueProvenance> {
        match self {
            Self::Explicit { raw, .. } => Some(*raw),
            Self::Invalid { raw, .. } => *raw,
            Self::Defaulted { .. } | Self::Absent { .. } => None,
        }
    }

    #[must_use]
    pub const fn value(&self) -> Option<&T> {
        match self {
            Self::Explicit { value, .. } | Self::Defaulted { value, .. } => Some(value),
            Self::Absent { .. } | Self::Invalid { .. } => None,
        }
    }

    #[must_use]
    pub const fn invalid_issue(&self) -> Option<&I> {
        match self {
            Self::Invalid { issue, .. } => Some(issue),
            Self::Explicit { .. } | Self::Defaulted { .. } | Self::Absent { .. } => None,
        }
    }

    #[must_use]
    pub fn as_ref(&self) -> DxfSemanticValue<&T, &I> {
        match self {
            Self::Explicit { value, field, raw } => DxfSemanticValue::explicit(value, *field, *raw),
            Self::Defaulted { value, field } => DxfSemanticValue::defaulted(value, *field),
            Self::Absent { field } => DxfSemanticValue::absent(*field),
            Self::Invalid { issue, field, raw } => DxfSemanticValue::invalid(issue, *field, *raw),
        }
    }

    #[must_use]
    pub fn map_value<U>(self, map: impl FnOnce(T) -> U) -> DxfSemanticValue<U, I> {
        match self {
            Self::Explicit { value, field, raw } => {
                DxfSemanticValue::explicit(map(value), field, raw)
            }
            Self::Defaulted { value, field } => DxfSemanticValue::defaulted(map(value), field),
            Self::Absent { field } => DxfSemanticValue::absent(field),
            Self::Invalid { issue, field, raw } => DxfSemanticValue::invalid(issue, field, raw),
        }
    }

    #[must_use]
    pub fn map_invalid<J>(self, map: impl FnOnce(I) -> J) -> DxfSemanticValue<T, J> {
        match self {
            Self::Explicit { value, field, raw } => DxfSemanticValue::explicit(value, field, raw),
            Self::Defaulted { value, field } => DxfSemanticValue::defaulted(value, field),
            Self::Absent { field } => DxfSemanticValue::absent(field),
            Self::Invalid { issue, field, raw } => {
                DxfSemanticValue::invalid(map(issue), field, raw)
            }
        }
    }
}

impl<T, I> fmt::Debug for DxfSemanticValue<T, I> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("DxfSemanticValue");
        debug.field("state", &self.state());
        debug.field("field_provenance", &self.field_provenance());
        if let Some(raw) = self.raw_provenance() {
            debug.field("raw_provenance", &raw);
        }
        debug.finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    use super::{
        DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSemanticValueState,
    };
    use crate::{ByteSpan, DxfSourceId};

    #[test]
    fn four_states_keep_exact_provenance_without_inventing_raw_bytes() -> Result<(), Box<dyn Error>>
    {
        let field = field_provenance();
        let raw = raw_provenance()?;

        let explicit = DxfSemanticValue::<_, &'static str>::explicit(7_u32, field, raw);
        assert_eq!(explicit.state(), DxfSemanticValueState::Explicit);
        assert_eq!(explicit.value(), Some(&7));
        assert_eq!(explicit.invalid_issue(), None);
        assert_eq!(explicit.raw_provenance(), Some(raw));

        let defaulted = DxfSemanticValue::<_, &'static str>::defaulted(11_u32, field);
        assert_eq!(defaulted.state(), DxfSemanticValueState::Defaulted);
        assert_eq!(defaulted.value(), Some(&11));
        assert_eq!(defaulted.raw_provenance(), None);

        let absent = DxfSemanticValue::<u32, &'static str>::absent(field);
        assert_eq!(absent.state(), DxfSemanticValueState::Absent);
        assert_eq!(absent.value(), None);
        assert_eq!(absent.raw_provenance(), None);

        let invalid = DxfSemanticValue::<u32, _>::invalid("wrong_group", field, Some(raw));
        assert_eq!(invalid.state(), DxfSemanticValueState::Invalid);
        assert_eq!(invalid.value(), None);
        assert_eq!(invalid.invalid_issue(), Some(&"wrong_group"));
        assert_eq!(invalid.raw_provenance(), Some(raw));

        let missing = DxfSemanticValue::<u32, _>::invalid("missing_required", field, None);
        assert_eq!(missing.raw_provenance(), None);
        for value in [explicit.field_provenance(), missing.field_provenance()] {
            assert_eq!(value, field);
            assert_eq!(value.document_source_id(), field.document_source_id());
            assert_eq!(value.schema_namespace(), "header");
            assert_eq!(value.schema_field_id(), "acadver");
        }
        Ok(())
    }

    #[test]
    fn transforms_preserve_state_and_provenance() -> Result<(), Box<dyn Error>> {
        let field = field_provenance();
        let raw = raw_provenance()?;
        let explicit = DxfSemanticValue::<_, &'static str>::explicit(7_u32, field, raw);
        let mapped = explicit.map_value(|value| value.to_string());
        assert_eq!(mapped.value().map(String::as_str), Some("7"));
        assert_eq!(mapped.raw_provenance(), Some(raw));
        let borrowed = mapped.as_ref();
        assert_eq!(borrowed.value().copied().map(String::as_str), Some("7"));

        let invalid = DxfSemanticValue::<u32, _>::invalid("bad", field, Some(raw));
        let mapped_issue = invalid.map_invalid(str::len);
        assert_eq!(mapped_issue.invalid_issue(), Some(&3));
        assert_eq!(mapped_issue.state(), DxfSemanticValueState::Invalid);
        assert_eq!(mapped_issue.field_provenance(), field);
        Ok(())
    }

    #[test]
    fn occurrence_is_bounded_and_debug_redacts_value_and_issue() -> Result<(), Box<dyn Error>> {
        let span = ByteSpan::new(10, 20).ok_or(io::Error::other("span"))?;
        assert!(DxfRawValueProvenance::new(u64::from(u32::MAX), span).is_some());
        assert!(DxfRawValueProvenance::new(u64::from(u32::MAX) + 1, span).is_none());

        let field = field_provenance();
        let raw = raw_provenance()?;
        let explicit = DxfSemanticValue::<_, String>::explicit("secret-value", field, raw);
        let invalid = DxfSemanticValue::<String, _>::invalid("secret-issue", field, Some(raw));
        let explicit_debug = format!("{explicit:?}");
        let invalid_debug = format!("{invalid:?}");
        assert!(!explicit_debug.contains("secret-value"));
        assert!(!invalid_debug.contains("secret-issue"));
        assert!(explicit_debug.contains("Explicit"));
        assert!(invalid_debug.contains("Invalid"));
        Ok(())
    }

    #[test]
    fn public_values_are_send_sync_and_conditionally_copy() -> Result<(), Box<dyn Error>> {
        assert_send_sync::<DxfSemanticValue<String, String>>();
        assert_copy::<DxfSemanticValue<u32, u16>>();
        let value =
            DxfSemanticValue::<_, u16>::explicit(5_u32, field_provenance(), raw_provenance()?);
        assert_eq!(value, value);
        Ok(())
    }

    fn field_provenance() -> DxfSemanticFieldProvenance {
        DxfSemanticFieldProvenance::new(
            DxfSourceId::from([0x5a; DxfSourceId::BYTE_LEN]),
            "header",
            "acadver",
        )
    }

    fn raw_provenance() -> Result<DxfRawValueProvenance, io::Error> {
        let span = ByteSpan::new(24, 30).ok_or(io::Error::other("span"))?;
        DxfRawValueProvenance::new(5, span).ok_or(io::Error::other("occurrence"))
    }

    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_copy<T: Copy>() {}
}
