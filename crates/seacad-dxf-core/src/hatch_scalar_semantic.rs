//! Singleton selection, reviewed defaults, and domains for HATCH scalar fields.

use std::io;

use crate::{
    DXF_HATCH_SCALAR_ROLES, DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfDouble, DxfError, DxfHatchScalarCard, DxfHatchScalarCardDirectory,
    DxfHatchScalarCardState, DxfHatchScalarEntry, DxfHatchScalarIssue, DxfHatchScalarOccurrence,
    DxfHatchScalarRole, DxfHatchScalarValue, DxfIoOperation, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const HATCH_NAMESPACE: &str = "entity.hatch";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchScalarSemanticIssue {
    MultipleValues { occurrence_count: u32 },
    InvalidAsciiNumber(DxfAsciiNumericIssue),
    NonFiniteDouble(DxfDouble),
    ValueOutOfDomain(DxfHatchScalarValue),
}

pub type DxfHatchScalarSemanticValue =
    DxfSemanticValue<DxfHatchScalarValue, DxfHatchScalarSemanticIssue>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchScalarSemanticEntry {
    ordinal: u32,
    scalar_entry: DxfHatchScalarEntry,
    role: DxfHatchScalarRole,
    semantic: DxfHatchScalarSemanticValue,
}

impl DxfHatchScalarSemanticEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn scalar_entry(self) -> DxfHatchScalarEntry {
        self.scalar_entry
    }

    #[must_use]
    pub const fn role(self) -> DxfHatchScalarRole {
        self.role
    }

    #[must_use]
    pub const fn semantic(&self) -> &DxfHatchScalarSemanticValue {
        &self.semantic
    }
}

/// Twenty-five stable singleton semantics per exact `AcDbHatch` subclass.
#[derive(Debug)]
pub struct DxfHatchScalarSemanticDirectory {
    source_id: DxfSourceId,
    cards: DxfHatchScalarCardDirectory,
    entries: Box<[DxfHatchScalarSemanticEntry]>,
}

impl DxfHatchScalarSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let cards = document.hatch_scalar_card_directory(cancellation)?;
        ensure_source(document.source_id(), cards.source_id())?;
        let capacity = cards
            .evidence_directory()
            .entries()
            .len()
            .checked_mul(DXF_HATCH_SCALAR_ROLES.len())
            .ok_or_else(invalid_internal_data)?;
        let mut entries = Vec::new();
        entries.try_reserve(capacity).map_err(|_| out_of_memory())?;
        for scalar_entry in cards.evidence_directory().entries().iter().copied() {
            for role in DXF_HATCH_SCALAR_ROLES {
                ensure_not_cancelled(cancellation)?;
                entries.push(DxfHatchScalarSemanticEntry {
                    ordinal: compact_len(entries.len())?,
                    scalar_entry,
                    role,
                    semantic: scalar_semantic(&cards, scalar_entry, role)?,
                });
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            cards,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn card_directory(&self) -> &DxfHatchScalarCardDirectory {
        &self.cards
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchScalarSemanticEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchScalarSemanticEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entries_for_subclass(&self, ordinal: u64) -> Option<&[DxfHatchScalarSemanticEntry]> {
        self.cards
            .evidence_directory()
            .entry_for_subclass(ordinal)?;
        let start = self
            .entries
            .partition_point(|entry| entry.scalar_entry().subclass_ordinal() < ordinal);
        let end = self
            .entries
            .partition_point(|entry| entry.scalar_entry().subclass_ordinal() <= ordinal);
        self.entries.get(start..end)
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> &[DxfHatchScalarSemanticEntry] {
        let start = self.entries.partition_point(|entry| {
            entry.scalar_entry().subclass().entity().record().ordinal() < raw
        });
        let end = self.entries.partition_point(|entry| {
            entry.scalar_entry().subclass().entity().record().ordinal() <= raw
        });
        self.entries.get(start..end).unwrap_or_default()
    }

    #[must_use]
    pub fn entry_for_role(
        &self,
        subclass_ordinal: u64,
        role: DxfHatchScalarRole,
    ) -> Option<DxfHatchScalarSemanticEntry> {
        self.entries_for_subclass(subclass_ordinal)?
            .iter()
            .copied()
            .find(|entry| entry.role() == role)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_scalar_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchScalarSemanticDirectory, DxfError> {
        DxfHatchScalarSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_scalar_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchScalarSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_scalar_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_scalar_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchScalarSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_scalar_semantic_directory(cancellation)
    }
}

fn scalar_semantic(
    cards: &DxfHatchScalarCardDirectory,
    scalar_entry: DxfHatchScalarEntry,
    role: DxfHatchScalarRole,
) -> Result<DxfHatchScalarSemanticValue, DxfError> {
    let field = DxfSemanticFieldProvenance::new(cards.source_id(), HATCH_NAMESPACE, field_id(role));
    let card = cards
        .card_for_role(scalar_entry.subclass_ordinal(), role)
        .ok_or_else(invalid_internal_data)?;
    match card.state() {
        DxfHatchScalarCardState::Absent => Ok(default_for(role)
            .map(|value| DxfSemanticValue::defaulted(value, field))
            .unwrap_or_else(|| DxfSemanticValue::absent(field))),
        DxfHatchScalarCardState::Multiple { occurrence_count } => Ok(DxfSemanticValue::invalid(
            DxfHatchScalarSemanticIssue::MultipleValues { occurrence_count },
            field,
            None,
        )),
        DxfHatchScalarCardState::Unique => {
            let occurrence = unique_occurrence(cards, card)?;
            let raw = raw_provenance(occurrence)?;
            Ok(match occurrence.value() {
                Ok(value) => match semantic_value(role, value)? {
                    Ok(value) => DxfSemanticValue::explicit(value, field, raw),
                    Err(issue) => DxfSemanticValue::invalid(issue, field, Some(raw)),
                },
                Err(DxfHatchScalarIssue::InvalidAsciiNumber(issue)) => DxfSemanticValue::invalid(
                    DxfHatchScalarSemanticIssue::InvalidAsciiNumber(issue),
                    field,
                    Some(raw),
                ),
            })
        }
    }
}

fn semantic_value(
    role: DxfHatchScalarRole,
    value: DxfHatchScalarValue,
) -> Result<Result<DxfHatchScalarValue, DxfHatchScalarSemanticIssue>, DxfError> {
    if !has_expected_wire(role, value) {
        return Err(invalid_internal_data());
    }
    if let DxfHatchScalarValue::Double(number) = value
        && !number.is_finite()
    {
        return Ok(Err(DxfHatchScalarSemanticIssue::NonFiniteDouble(number)));
    }
    if value_in_domain(role, value) {
        Ok(Ok(value))
    } else {
        Ok(Err(DxfHatchScalarSemanticIssue::ValueOutOfDomain(value)))
    }
}

const fn has_expected_wire(role: DxfHatchScalarRole, value: DxfHatchScalarValue) -> bool {
    use DxfHatchScalarRole::*;
    match role {
        PatternName | GradientName => matches!(value, DxfHatchScalarValue::Text),
        SolidFillFlag | AssociativityFlag | HatchStyle | PatternType | PatternDoubleFlag
        | PatternLineCount => matches!(value, DxfHatchScalarValue::Int16(_)),
        BoundaryPathCount | SeedPointCount | GradientKind | GradientReserved
        | GradientColorMode | GradientColorCount => matches!(value, DxfHatchScalarValue::Int32(_)),
        ElevationZ
        | ExtrusionX
        | ExtrusionY
        | ExtrusionZ
        | PatternAngle
        | PatternScale
        | PixelSize
        | GradientRotation
        | GradientShift
        | GradientTint
        | GradientReservedValue => matches!(value, DxfHatchScalarValue::Double(_)),
    }
}

fn value_in_domain(role: DxfHatchScalarRole, value: DxfHatchScalarValue) -> bool {
    use DxfHatchScalarRole::*;
    match (role, value) {
        (
            SolidFillFlag | AssociativityFlag | PatternDoubleFlag,
            DxfHatchScalarValue::Int16(value),
        ) => (0..=1).contains(&value),
        (HatchStyle | PatternType, DxfHatchScalarValue::Int16(value)) => (0..=2).contains(&value),
        (PatternLineCount, DxfHatchScalarValue::Int16(value)) => value >= 0,
        (BoundaryPathCount | SeedPointCount, DxfHatchScalarValue::Int32(value)) => value >= 0,
        (GradientKind | GradientColorMode, DxfHatchScalarValue::Int32(value)) => {
            (0..=1).contains(&value)
        }
        (GradientReserved, DxfHatchScalarValue::Int32(value)) => value == 0,
        (GradientColorCount, DxfHatchScalarValue::Int32(value)) => matches!(value, 0 | 2),
        (GradientShift | GradientTint, DxfHatchScalarValue::Double(value)) => {
            (0.0..=1.0).contains(&value.to_f64())
        }
        (GradientReservedValue, DxfHatchScalarValue::Double(value)) => {
            matches!(value.to_f64(), 0.0 | 1.0)
        }
        _ => true,
    }
}

fn default_for(role: DxfHatchScalarRole) -> Option<DxfHatchScalarValue> {
    let value = match role {
        DxfHatchScalarRole::ExtrusionX | DxfHatchScalarRole::ExtrusionY => 0.0,
        DxfHatchScalarRole::ExtrusionZ => 1.0,
        _ => return None,
    };
    Some(DxfHatchScalarValue::Double(DxfDouble::from_f64(value)))
}

fn unique_occurrence(
    cards: &DxfHatchScalarCardDirectory,
    card: DxfHatchScalarCard,
) -> Result<DxfHatchScalarOccurrence, DxfError> {
    let [member] = cards
        .members_for_card(card.ordinal())
        .ok_or_else(invalid_internal_data)?
    else {
        return Err(invalid_internal_data());
    };
    cards
        .occurrence_for_member(*member)
        .ok_or_else(invalid_internal_data)
}

fn raw_provenance(occurrence: DxfHatchScalarOccurrence) -> Result<DxfRawValueProvenance, DxfError> {
    DxfRawValueProvenance::new(
        occurrence.group().occurrence(),
        occurrence.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)
}

const fn field_id(role: DxfHatchScalarRole) -> &'static str {
    use DxfHatchScalarRole::*;
    match role {
        ElevationZ => "elevation_z",
        ExtrusionX => "extrusion_x",
        ExtrusionY => "extrusion_y",
        ExtrusionZ => "extrusion_z",
        PatternName => "pattern_name",
        SolidFillFlag => "solid_fill_flag",
        AssociativityFlag => "associativity_flag",
        BoundaryPathCount => "boundary_path_count",
        HatchStyle => "hatch_style",
        PatternType => "pattern_type",
        PatternAngle => "pattern_angle",
        PatternScale => "pattern_scale",
        PatternDoubleFlag => "pattern_double_flag",
        PatternLineCount => "pattern_line_count",
        PixelSize => "pixel_size",
        SeedPointCount => "seed_point_count",
        GradientKind => "gradient_kind",
        GradientReserved => "gradient_reserved",
        GradientColorMode => "gradient_color_mode",
        GradientColorCount => "gradient_color_count",
        GradientRotation => "gradient_rotation",
        GradientShift => "gradient_shift",
        GradientTint => "gradient_tint",
        GradientReservedValue => "gradient_reserved_value",
        GradientName => "gradient_name",
    }
}

fn compact_len(len: usize) -> Result<u32, DxfError> {
    u32::try_from(len).map_err(|_| invalid_internal_data())
}

fn ensure_source(expected: DxfSourceId, observed: DxfSourceId) -> Result<(), DxfError> {
    if expected == observed {
        Ok(())
    } else {
        Err(DxfError::SourceIdentityMismatch { expected, observed })
    }
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    io_error(io::ErrorKind::InvalidData)
}

fn out_of_memory() -> DxfError {
    io_error(io::ErrorKind::OutOfMemory)
}

fn io_error(kind: io::ErrorKind) -> DxfError {
    DxfError::from_io(DxfIoOperation::Read, &io::Error::from(kind))
}
