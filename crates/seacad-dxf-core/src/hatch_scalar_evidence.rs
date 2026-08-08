//! Typed evidence for unambiguous scalar fields in exact HATCH subclasses.

use std::io;

use crate::{
    DxfAsciiNumericIssue, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken,
    DxfDouble, DxfError, DxfFillMeshEvidenceDirectory, DxfFillMeshFamily, DxfFillMeshRange,
    DxfFillMeshSubclassEntry, DxfIoOperation, DxfRawDocumentView, DxfRawGroup, DxfSourceId,
    raw_double::decode_raw_double, raw_integer::decode_raw_i16, raw_integer::decode_raw_i32,
};

pub const DXF_HATCH_SCALAR_ROLES: [DxfHatchScalarRole; 25] = [
    DxfHatchScalarRole::ElevationZ,
    DxfHatchScalarRole::ExtrusionX,
    DxfHatchScalarRole::ExtrusionY,
    DxfHatchScalarRole::ExtrusionZ,
    DxfHatchScalarRole::PatternName,
    DxfHatchScalarRole::SolidFillFlag,
    DxfHatchScalarRole::AssociativityFlag,
    DxfHatchScalarRole::BoundaryPathCount,
    DxfHatchScalarRole::HatchStyle,
    DxfHatchScalarRole::PatternType,
    DxfHatchScalarRole::PatternAngle,
    DxfHatchScalarRole::PatternScale,
    DxfHatchScalarRole::PatternDoubleFlag,
    DxfHatchScalarRole::PatternLineCount,
    DxfHatchScalarRole::PixelSize,
    DxfHatchScalarRole::SeedPointCount,
    DxfHatchScalarRole::GradientKind,
    DxfHatchScalarRole::GradientReserved,
    DxfHatchScalarRole::GradientColorMode,
    DxfHatchScalarRole::GradientColorCount,
    DxfHatchScalarRole::GradientRotation,
    DxfHatchScalarRole::GradientShift,
    DxfHatchScalarRole::GradientTint,
    DxfHatchScalarRole::GradientReservedValue,
    DxfHatchScalarRole::GradientName,
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchScalarRole {
    ElevationZ,
    ExtrusionX,
    ExtrusionY,
    ExtrusionZ,
    PatternName,
    SolidFillFlag,
    AssociativityFlag,
    BoundaryPathCount,
    HatchStyle,
    PatternType,
    PatternAngle,
    PatternScale,
    PatternDoubleFlag,
    PatternLineCount,
    PixelSize,
    SeedPointCount,
    GradientKind,
    GradientReserved,
    GradientColorMode,
    GradientColorCount,
    GradientRotation,
    GradientShift,
    GradientTint,
    GradientReservedValue,
    GradientName,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchScalarValue {
    Text,
    Double(DxfDouble),
    Int16(i16),
    Int32(i32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchScalarIssue {
    InvalidAsciiNumber(DxfAsciiNumericIssue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchScalarOccurrence {
    group: DxfRawGroup,
    role: DxfHatchScalarRole,
    value: Result<DxfHatchScalarValue, DxfHatchScalarIssue>,
}

impl DxfHatchScalarOccurrence {
    #[must_use]
    pub const fn group(self) -> DxfRawGroup {
        self.group
    }

    #[must_use]
    pub const fn role(self) -> DxfHatchScalarRole {
        self.role
    }

    pub const fn value(self) -> Result<DxfHatchScalarValue, DxfHatchScalarIssue> {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchScalarEntry {
    subclass: DxfFillMeshSubclassEntry,
    subclass_ordinal: u32,
    occurrence_range: DxfFillMeshRange,
}

impl DxfHatchScalarEntry {
    #[must_use]
    pub const fn subclass(self) -> DxfFillMeshSubclassEntry {
        self.subclass
    }

    #[must_use]
    pub const fn subclass_ordinal(self) -> u64 {
        self.subclass_ordinal as u64
    }

    #[must_use]
    pub const fn occurrence_range(self) -> DxfFillMeshRange {
        self.occurrence_range
    }
}

/// Source-ordered values whose codes are unambiguous across the HATCH grammar.
#[derive(Debug)]
pub struct DxfHatchScalarDirectory {
    source_id: DxfSourceId,
    evidence: DxfFillMeshEvidenceDirectory,
    entries: Box<[DxfHatchScalarEntry]>,
    occurrences: Box<[DxfHatchScalarOccurrence]>,
}

impl DxfHatchScalarDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let evidence = document.fill_mesh_evidence_directory(cancellation)?;
        ensure_source(document.source_id(), evidence.source_id())?;
        let mut entries = Vec::new();
        let mut occurrences = Vec::new();

        for (subclass_index, subclass) in evidence.subclasses().iter().copied().enumerate() {
            ensure_not_cancelled(cancellation)?;
            if subclass.family() != DxfFillMeshFamily::Hatch {
                continue;
            }
            let subclass_ordinal = compact_len(subclass_index)?;
            let start = compact_len(occurrences.len())?;
            let fields = evidence
                .fields_for_subclass(subclass_index as u64)
                .ok_or_else(invalid_internal_data)?;
            for field in fields.iter().copied() {
                ensure_not_cancelled(cancellation)?;
                let Some(role) = role_for_code(field.group().group_code().value()) else {
                    continue;
                };
                let value = decode_value(document, field.group(), role, cancellation)?;
                occurrences.try_reserve(1).map_err(|_| out_of_memory())?;
                occurrences.push(DxfHatchScalarOccurrence {
                    group: field.group(),
                    role,
                    value,
                });
            }
            let end = compact_len(occurrences.len())?;
            entries.try_reserve(1).map_err(|_| out_of_memory())?;
            entries.push(DxfHatchScalarEntry {
                subclass,
                subclass_ordinal,
                occurrence_range: DxfFillMeshRange::new(start, end)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            evidence,
            entries: entries.into_boxed_slice(),
            occurrences: occurrences.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn evidence_directory(&self) -> &DxfFillMeshEvidenceDirectory {
        &self.evidence
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchScalarEntry] {
        &self.entries
    }

    #[must_use]
    pub fn occurrences(&self) -> &[DxfHatchScalarOccurrence] {
        &self.occurrences
    }

    #[must_use]
    pub fn entry_for_subclass(&self, ordinal: u64) -> Option<DxfHatchScalarEntry> {
        self.entries
            .binary_search_by_key(&ordinal, |entry| entry.subclass_ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> &[DxfHatchScalarEntry] {
        let start = self
            .entries
            .partition_point(|entry| entry.subclass().entity().record().ordinal() < raw);
        let end = self
            .entries
            .partition_point(|entry| entry.subclass().entity().record().ordinal() <= raw);
        self.entries.get(start..end).unwrap_or_default()
    }

    #[must_use]
    pub fn occurrences_for_subclass(&self, ordinal: u64) -> Option<&[DxfHatchScalarOccurrence]> {
        let entry = self.entry_for_subclass(ordinal)?;
        slice_for_range(&self.occurrences, entry.occurrence_range())
    }

    #[must_use]
    pub fn occurrence_for_group(&self, group_occurrence: u64) -> Option<DxfHatchScalarOccurrence> {
        let index = self
            .occurrences
            .partition_point(|entry| entry.group().occurrence() < group_occurrence);
        self.occurrences
            .get(index)
            .copied()
            .filter(|entry| entry.group().occurrence() == group_occurrence)
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_scalar_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchScalarDirectory, DxfError> {
        DxfHatchScalarDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_scalar_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_scalar_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchScalarDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_scalar_directory(cancellation)
    }
}

fn decode_value(
    document: DxfRawDocumentView<'_>,
    group: DxfRawGroup,
    role: DxfHatchScalarRole,
    cancellation: &DxfCancellationToken,
) -> Result<Result<DxfHatchScalarValue, DxfHatchScalarIssue>, DxfError> {
    let value = match value_kind(role) {
        DxfHatchScalarValueKind::Text => Ok(DxfHatchScalarValue::Text),
        DxfHatchScalarValueKind::Double => {
            decode_raw_double(document, group, cancellation)?.map(DxfHatchScalarValue::Double)
        }
        DxfHatchScalarValueKind::Int16 => {
            decode_raw_i16(document, group, cancellation)?.map(DxfHatchScalarValue::Int16)
        }
        DxfHatchScalarValueKind::Int32 => {
            decode_raw_i32(document, group, cancellation)?.map(DxfHatchScalarValue::Int32)
        }
    };
    Ok(value.map_err(DxfHatchScalarIssue::InvalidAsciiNumber))
}

#[derive(Clone, Copy)]
enum DxfHatchScalarValueKind {
    Text,
    Double,
    Int16,
    Int32,
}

const fn value_kind(role: DxfHatchScalarRole) -> DxfHatchScalarValueKind {
    use DxfHatchScalarRole::*;
    match role {
        PatternName | GradientName => DxfHatchScalarValueKind::Text,
        SolidFillFlag | AssociativityFlag | HatchStyle | PatternType | PatternDoubleFlag
        | PatternLineCount => DxfHatchScalarValueKind::Int16,
        BoundaryPathCount | SeedPointCount | GradientKind | GradientReserved
        | GradientColorMode | GradientColorCount => DxfHatchScalarValueKind::Int32,
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
        | GradientReservedValue => DxfHatchScalarValueKind::Double,
    }
}

const fn role_for_code(code: i16) -> Option<DxfHatchScalarRole> {
    use DxfHatchScalarRole::*;
    match code {
        30 => Some(ElevationZ),
        210 => Some(ExtrusionX),
        220 => Some(ExtrusionY),
        230 => Some(ExtrusionZ),
        2 => Some(PatternName),
        70 => Some(SolidFillFlag),
        71 => Some(AssociativityFlag),
        91 => Some(BoundaryPathCount),
        75 => Some(HatchStyle),
        76 => Some(PatternType),
        52 => Some(PatternAngle),
        41 => Some(PatternScale),
        77 => Some(PatternDoubleFlag),
        78 => Some(PatternLineCount),
        47 => Some(PixelSize),
        98 => Some(SeedPointCount),
        450 => Some(GradientKind),
        451 => Some(GradientReserved),
        452 => Some(GradientColorMode),
        453 => Some(GradientColorCount),
        460 => Some(GradientRotation),
        461 => Some(GradientShift),
        462 => Some(GradientTint),
        463 => Some(GradientReservedValue),
        470 => Some(GradientName),
        _ => None,
    }
}

fn slice_for_range<T>(values: &[T], range: DxfFillMeshRange) -> Option<&[T]> {
    let start = usize::try_from(range.start()).ok()?;
    let end = usize::try_from(range.end()).ok()?;
    values.get(start..end)
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
