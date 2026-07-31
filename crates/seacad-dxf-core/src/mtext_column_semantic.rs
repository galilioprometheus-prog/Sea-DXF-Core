//! Fail-closed scalar semantics for modern MTEXT embedded columns.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfIoOperation, DxfMTextEmbeddedColumnDirectory, DxfMTextEmbeddedColumnEntry,
    DxfMTextEmbeddedColumnRole, DxfMTextFlatColumnDirectory, DxfMTextFlatColumnEntry,
    DxfMTextXDataColumnDirectory, DxfMTextXDataColumnEntry, DxfRawDocumentView, DxfRawGroup,
    DxfRawRecord, DxfRawValueProvenance, DxfSemanticValue, DxfSourceId, DxfTextSymbolNumericIssue,
    mtext_column_semantic_project::project_entry,
};

/// Autodesk MTEXT column type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextColumnType {
    NoColumns,
    Static,
    Dynamic,
}

impl DxfMTextColumnType {
    #[must_use]
    pub const fn code(self) -> i16 {
        match self {
            Self::NoColumns => 0,
            Self::Static => 1,
            Self::Dynamic => 2,
        }
    }
}

/// Why an embedded MTEXT column scalar is unusable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextColumnIssue {
    Numeric(DxfTextSymbolNumericIssue),
    MissingColumnType,
    MultipleValues {
        role: DxfMTextEmbeddedColumnRole,
        occurrence_count: u64,
    },
    UnsupportedColumnType {
        code: i16,
    },
    NegativeCount {
        count: i16,
    },
    BooleanOutOfDomain {
        role: DxfMTextEmbeddedColumnRole,
        code: i16,
    },
    NonPositiveValue {
        role: DxfMTextEmbeddedColumnRole,
        value: DxfDouble,
    },
    NegativeValue {
        role: DxfMTextEmbeddedColumnRole,
        value: DxfDouble,
    },
}

pub type DxfMTextColumnTypeSemantic = DxfSemanticValue<DxfMTextColumnType, DxfMTextColumnIssue>;
pub type DxfMTextColumnCountSemantic = DxfSemanticValue<u16, DxfMTextColumnIssue>;
pub type DxfMTextColumnBooleanSemantic = DxfSemanticValue<bool, DxfMTextColumnIssue>;
pub type DxfMTextColumnDoubleSemantic = DxfSemanticValue<DxfDouble, DxfMTextColumnIssue>;

/// Whether one source envelope frames MTEXT height values unambiguously.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextColumnHeightDisposition {
    /// Embedded/XDATA framing assigns any height values an exact role.
    Unambiguous,
    /// Direct/flat storage contains no group-50 height candidate.
    NoHeightEvidence,
    /// Direct group 50 may be rotation, shared height, or an individual height.
    AmbiguousDirectGroup50 {
        occurrence_count: u64,
        first_raw: DxfRawValueProvenance,
    },
}

/// Physical source envelope behind one unified MTEXT column projection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextColumnSourceEntry {
    Flat(DxfMTextFlatColumnEntry),
    Embedded(DxfMTextEmbeddedColumnEntry),
    AcadXData(DxfMTextXDataColumnEntry),
}

impl DxfMTextColumnSourceEntry {
    #[must_use]
    pub const fn record(self) -> DxfRawRecord {
        match self {
            Self::Flat(entry) => entry.record(),
            Self::Embedded(entry) => entry.record(),
            Self::AcadXData(entry) => entry.record(),
        }
    }

    #[must_use]
    pub const fn marker(self) -> DxfRawGroup {
        match self {
            Self::Flat(entry) => entry.marker(),
            Self::Embedded(entry) => entry.marker(),
            Self::AcadXData(entry) => entry.begin(),
        }
    }
}

/// Typed scalar projection for one MTEXT column source envelope.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextColumnSemantics {
    pub(super) entry: DxfMTextColumnSourceEntry,
    pub(super) column_type: DxfMTextColumnTypeSemantic,
    pub(super) column_count: DxfMTextColumnCountSemantic,
    pub(super) column_width: DxfMTextColumnDoubleSemantic,
    pub(super) column_gutter: DxfMTextColumnDoubleSemantic,
    pub(super) auto_height: DxfMTextColumnBooleanSemantic,
    pub(super) flow_reversed: DxfMTextColumnBooleanSemantic,
    pub(super) shared_height: DxfMTextColumnDoubleSemantic,
    pub(super) height_disposition: DxfMTextColumnHeightDisposition,
    pub(super) height_start: u32,
    pub(super) height_end: u32,
}

impl DxfMTextColumnSemantics {
    #[must_use]
    pub const fn entry(self) -> DxfMTextColumnSourceEntry {
        self.entry
    }

    #[must_use]
    pub const fn column_type(&self) -> &DxfMTextColumnTypeSemantic {
        &self.column_type
    }

    #[must_use]
    pub const fn column_count(&self) -> &DxfMTextColumnCountSemantic {
        &self.column_count
    }

    #[must_use]
    pub const fn column_width(&self) -> &DxfMTextColumnDoubleSemantic {
        &self.column_width
    }

    #[must_use]
    pub const fn column_gutter(&self) -> &DxfMTextColumnDoubleSemantic {
        &self.column_gutter
    }

    #[must_use]
    pub const fn auto_height(&self) -> &DxfMTextColumnBooleanSemantic {
        &self.auto_height
    }

    #[must_use]
    pub const fn flow_reversed(&self) -> &DxfMTextColumnBooleanSemantic {
        &self.flow_reversed
    }

    #[must_use]
    pub const fn shared_height(&self) -> &DxfMTextColumnDoubleSemantic {
        &self.shared_height
    }

    #[must_use]
    pub const fn height_disposition(self) -> DxfMTextColumnHeightDisposition {
        self.height_disposition
    }

    #[must_use]
    pub const fn individual_height_count(self) -> u64 {
        (self.height_end - self.height_start) as u64
    }
}

/// Modern MTEXT column evidence plus its bounded scalar projection.
#[derive(Debug)]
pub struct DxfMTextColumnSemanticDirectory {
    source_id: DxfSourceId,
    flat_evidence: DxfMTextFlatColumnDirectory,
    embedded_evidence: DxfMTextEmbeddedColumnDirectory,
    xdata_evidence: DxfMTextXDataColumnDirectory,
    semantics: Box<[DxfMTextColumnSemantics]>,
    individual_heights: Box<[DxfMTextColumnDoubleSemantic]>,
}

impl DxfMTextColumnSemanticDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let flat_evidence = document.mtext_flat_column_directory(cancellation)?;
        let embedded_evidence = document.mtext_embedded_column_directory(cancellation)?;
        let xdata_evidence = document.mtext_xdata_column_directory(cancellation)?;
        for observed in [
            flat_evidence.source_id(),
            embedded_evidence.source_id(),
            xdata_evidence.source_id(),
        ] {
            if observed != document.source_id() {
                return Err(DxfError::SourceIdentityMismatch {
                    expected: document.source_id(),
                    observed,
                });
            }
        }
        let mut sources = Vec::new();
        sources
            .try_reserve(
                flat_evidence
                    .entries()
                    .len()
                    .saturating_add(embedded_evidence.entries().len())
                    .saturating_add(xdata_evidence.entries().len()),
            )
            .map_err(|_| out_of_memory())?;
        sources.extend(
            flat_evidence
                .entries()
                .iter()
                .copied()
                .map(DxfMTextColumnSourceEntry::Flat),
        );
        sources.extend(
            embedded_evidence
                .entries()
                .iter()
                .copied()
                .map(DxfMTextColumnSourceEntry::Embedded),
        );
        sources.extend(
            xdata_evidence
                .entries()
                .iter()
                .copied()
                .map(DxfMTextColumnSourceEntry::AcadXData),
        );
        sources.sort_by_key(|entry| entry.marker().occurrence());
        let mut semantics = Vec::new();
        let mut individual_heights = Vec::new();
        for entry in sources {
            ensure_not_cancelled(cancellation)?;
            semantics.try_reserve(1).map_err(|_| out_of_memory())?;
            semantics.push(match entry {
                DxfMTextColumnSourceEntry::Flat(source) => {
                    let values = flat_evidence
                        .values_for_entry(source)
                        .ok_or_else(invalid_internal_data)?;
                    project_entry(
                        document.source_id(),
                        entry,
                        values,
                        flat_height_disposition(values)?,
                        &mut individual_heights,
                    )?
                }
                DxfMTextColumnSourceEntry::Embedded(source) => project_entry(
                    document.source_id(),
                    entry,
                    embedded_evidence
                        .values_for_entry(source)
                        .ok_or_else(invalid_internal_data)?,
                    DxfMTextColumnHeightDisposition::Unambiguous,
                    &mut individual_heights,
                )?,
                DxfMTextColumnSourceEntry::AcadXData(source) => project_entry(
                    document.source_id(),
                    entry,
                    xdata_evidence
                        .values_for_entry(source)
                        .ok_or_else(invalid_internal_data)?,
                    DxfMTextColumnHeightDisposition::Unambiguous,
                    &mut individual_heights,
                )?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            flat_evidence,
            embedded_evidence,
            xdata_evidence,
            semantics: semantics.into_boxed_slice(),
            individual_heights: individual_heights.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn flat_evidence_directory(&self) -> &DxfMTextFlatColumnDirectory {
        &self.flat_evidence
    }

    #[must_use]
    pub const fn evidence_directory(&self) -> &DxfMTextEmbeddedColumnDirectory {
        &self.embedded_evidence
    }

    #[must_use]
    pub const fn xdata_evidence_directory(&self) -> &DxfMTextXDataColumnDirectory {
        &self.xdata_evidence
    }

    #[must_use]
    pub fn semantics(&self) -> &[DxfMTextColumnSemantics] {
        &self.semantics
    }

    #[must_use]
    pub fn individual_heights(
        &self,
        semantics: DxfMTextColumnSemantics,
    ) -> Option<&[DxfMTextColumnDoubleSemantic]> {
        self.individual_heights.get(
            usize::try_from(semantics.height_start).ok()?
                ..usize::try_from(semantics.height_end).ok()?,
        )
    }

    #[must_use]
    pub fn semantics_for_marker_occurrence(
        &self,
        occurrence: u64,
    ) -> Option<DxfMTextColumnSemantics> {
        self.semantics
            .binary_search_by_key(&occurrence, |item| item.entry().marker().occurrence())
            .ok()
            .and_then(|index| self.semantics.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_column_semantic_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextColumnSemanticDirectory, DxfError> {
        DxfMTextColumnSemanticDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_column_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextColumnSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_column_semantic_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_column_semantic_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextColumnSemanticDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_column_semantic_directory(cancellation)
    }
}

fn flat_height_disposition(
    values: &[crate::DxfMTextFlatColumnValue],
) -> Result<DxfMTextColumnHeightDisposition, DxfError> {
    let mut matching = values
        .iter()
        .copied()
        .filter(|value| value.role() == crate::DxfMTextFlatColumnRole::RotationOrColumnHeight);
    let Some(first) = matching.next() else {
        return Ok(DxfMTextColumnHeightDisposition::NoHeightEvidence);
    };
    let first_raw = DxfRawValueProvenance::new(
        first.group().occurrence(),
        first.group().value_payload_span(),
    )
    .ok_or_else(invalid_internal_data)?;
    Ok(DxfMTextColumnHeightDisposition::AmbiguousDirectGroup50 {
        occurrence_count: 1 + matching.count() as u64,
        first_raw,
    })
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
