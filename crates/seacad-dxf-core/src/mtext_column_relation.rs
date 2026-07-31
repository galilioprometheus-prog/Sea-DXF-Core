//! Cross-field mode relationships for modern embedded MTEXT columns.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfError, DxfIoOperation,
    DxfMTextColumnDoubleSemantic, DxfMTextColumnIssue, DxfMTextColumnSemanticDirectory,
    DxfMTextColumnSemantics, DxfMTextColumnType, DxfMTextEmbeddedColumnRole, DxfRawDocumentView,
    DxfRawValueProvenance, DxfSemanticFieldProvenance, DxfSemanticValue, DxfSourceId,
};

const NAMESPACE: &str = "entity.mtext.embedded_column_relation";

/// Usable MTEXT column mode after cross-field validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextColumnMode {
    NoColumns,
    Static,
    DynamicAutomatic,
    DynamicManual,
}

/// Why embedded MTEXT column fields do not form a usable mode.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfMTextColumnRelationIssue {
    Scalar(DxfMTextColumnIssue),
    MissingRequired {
        role: DxfMTextEmbeddedColumnRole,
    },
    PositiveCountRequired {
        mode: DxfMTextColumnMode,
    },
    PositiveSharedHeightRequired {
        mode: DxfMTextColumnMode,
    },
    AutomaticHeightRequiresDynamic,
    IndividualHeightsRequireDynamicManual,
    IndividualHeightCountMismatch {
        column_count: u16,
        height_count: u64,
    },
}

pub type DxfMTextColumnModeSemantic =
    DxfSemanticValue<DxfMTextColumnMode, DxfMTextColumnRelationIssue>;

/// One embedded column scalar set plus its cross-field mode result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfMTextColumnRelationSemantics {
    scalars: DxfMTextColumnSemantics,
    mode: DxfMTextColumnModeSemantic,
}

impl DxfMTextColumnRelationSemantics {
    #[must_use]
    pub const fn scalar_semantics(self) -> DxfMTextColumnSemantics {
        self.scalars
    }

    #[must_use]
    pub const fn mode(&self) -> &DxfMTextColumnModeSemantic {
        &self.mode
    }
}

/// Bounded cross-field projection retaining the complete scalar directory.
#[derive(Debug)]
pub struct DxfMTextColumnRelationDirectory {
    source_id: DxfSourceId,
    scalars: DxfMTextColumnSemanticDirectory,
    semantics: Box<[DxfMTextColumnRelationSemantics]>,
}

impl DxfMTextColumnRelationDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        let scalars = document.mtext_column_semantic_directory(cancellation)?;
        if scalars.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: scalars.source_id(),
            });
        }
        let mut semantics = Vec::new();
        for scalar in scalars.semantics().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            let heights = scalars
                .individual_heights(scalar)
                .ok_or_else(invalid_internal_data)?;
            semantics.try_reserve(1).map_err(|_| out_of_memory())?;
            semantics.push(DxfMTextColumnRelationSemantics {
                scalars: scalar,
                mode: project_mode(scalars.source_id(), scalar, heights),
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            scalars,
            semantics: semantics.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn scalar_directory(&self) -> &DxfMTextColumnSemanticDirectory {
        &self.scalars
    }

    #[must_use]
    pub fn semantics(&self) -> &[DxfMTextColumnRelationSemantics] {
        &self.semantics
    }

    #[must_use]
    pub fn semantics_for_marker_occurrence(
        &self,
        occurrence: u64,
    ) -> Option<DxfMTextColumnRelationSemantics> {
        self.semantics
            .binary_search_by_key(&occurrence, |item| {
                item.scalar_semantics().entry().marker().occurrence()
            })
            .ok()
            .and_then(|index| self.semantics.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn mtext_column_relation_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextColumnRelationDirectory, DxfError> {
        DxfMTextColumnRelationDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn mtext_column_relation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextColumnRelationDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_column_relation_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn mtext_column_relation_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfMTextColumnRelationDirectory, DxfError> {
        DxfRawDocumentView::from(self).mtext_column_relation_directory(cancellation)
    }
}

type ProjectionFailure = (DxfMTextColumnRelationIssue, Option<DxfRawValueProvenance>);

fn project_mode(
    source_id: DxfSourceId,
    scalars: DxfMTextColumnSemantics,
    heights: &[DxfMTextColumnDoubleSemantic],
) -> DxfMTextColumnModeSemantic {
    let field = DxfSemanticFieldProvenance::new(source_id, NAMESPACE, "mode");
    let raw = scalars.column_type().raw_provenance();
    match usable(
        scalars.column_type(),
        DxfMTextEmbeddedColumnRole::ColumnType,
    )
    .and_then(|column_type| validate_mode(column_type, scalars, heights))
    {
        Ok(mode) => match raw {
            Some(raw) => DxfSemanticValue::explicit(mode, field, raw),
            None => DxfSemanticValue::defaulted(mode, field),
        },
        Err((issue, issue_raw)) => DxfSemanticValue::invalid(issue, field, issue_raw.or(raw)),
    }
}

fn validate_mode(
    column_type: DxfMTextColumnType,
    scalars: DxfMTextColumnSemantics,
    heights: &[DxfMTextColumnDoubleSemantic],
) -> Result<DxfMTextColumnMode, ProjectionFailure> {
    match column_type {
        DxfMTextColumnType::NoColumns => Ok(DxfMTextColumnMode::NoColumns),
        DxfMTextColumnType::Static => validate_static(scalars, heights),
        DxfMTextColumnType::Dynamic => validate_dynamic(scalars, heights),
    }
}

fn validate_static(
    scalars: DxfMTextColumnSemantics,
    heights: &[DxfMTextColumnDoubleSemantic],
) -> Result<DxfMTextColumnMode, ProjectionFailure> {
    require_dimensions(scalars)?;
    let count = usable(
        scalars.column_count(),
        DxfMTextEmbeddedColumnRole::ColumnCount,
    )?;
    if count == 0 {
        return Err((
            DxfMTextColumnRelationIssue::PositiveCountRequired {
                mode: DxfMTextColumnMode::Static,
            },
            scalars.column_count().raw_provenance(),
        ));
    }
    let height = usable(
        scalars.shared_height(),
        DxfMTextEmbeddedColumnRole::SharedHeight,
    )?;
    if height.to_f64() == 0.0 {
        return Err((
            DxfMTextColumnRelationIssue::PositiveSharedHeightRequired {
                mode: DxfMTextColumnMode::Static,
            },
            scalars.shared_height().raw_provenance(),
        ));
    }
    if scalars.auto_height().value() == Some(&true) {
        return Err((
            DxfMTextColumnRelationIssue::AutomaticHeightRequiresDynamic,
            scalars.auto_height().raw_provenance(),
        ));
    }
    if !heights.is_empty() {
        return Err((
            DxfMTextColumnRelationIssue::IndividualHeightsRequireDynamicManual,
            heights[0].raw_provenance(),
        ));
    }
    Ok(DxfMTextColumnMode::Static)
}

fn validate_dynamic(
    scalars: DxfMTextColumnSemantics,
    heights: &[DxfMTextColumnDoubleSemantic],
) -> Result<DxfMTextColumnMode, ProjectionFailure> {
    require_dimensions(scalars)?;
    let automatic = usable(
        scalars.auto_height(),
        DxfMTextEmbeddedColumnRole::ColumnAutoHeight,
    )?;
    if automatic {
        if !heights.is_empty() {
            return Err((
                DxfMTextColumnRelationIssue::IndividualHeightsRequireDynamicManual,
                heights[0].raw_provenance(),
            ));
        }
        return Ok(DxfMTextColumnMode::DynamicAutomatic);
    }
    let count = usable(
        scalars.column_count(),
        DxfMTextEmbeddedColumnRole::ColumnCount,
    )?;
    if count == 0 {
        return Err((
            DxfMTextColumnRelationIssue::PositiveCountRequired {
                mode: DxfMTextColumnMode::DynamicManual,
            },
            scalars.column_count().raw_provenance(),
        ));
    }
    for height in heights {
        usable(height, DxfMTextEmbeddedColumnRole::ColumnHeight)?;
    }
    if heights.is_empty() {
        let shared = usable(
            scalars.shared_height(),
            DxfMTextEmbeddedColumnRole::SharedHeight,
        )?;
        if shared.to_f64() == 0.0 {
            return Err((
                DxfMTextColumnRelationIssue::PositiveSharedHeightRequired {
                    mode: DxfMTextColumnMode::DynamicManual,
                },
                scalars.shared_height().raw_provenance(),
            ));
        }
    } else if u64::from(count) != heights.len() as u64 {
        return Err((
            DxfMTextColumnRelationIssue::IndividualHeightCountMismatch {
                column_count: count,
                height_count: heights.len() as u64,
            },
            scalars.column_count().raw_provenance(),
        ));
    }
    Ok(DxfMTextColumnMode::DynamicManual)
}

fn require_dimensions(scalars: DxfMTextColumnSemantics) -> Result<(), ProjectionFailure> {
    usable(
        scalars.column_width(),
        DxfMTextEmbeddedColumnRole::ColumnWidth,
    )?;
    usable(
        scalars.column_gutter(),
        DxfMTextEmbeddedColumnRole::ColumnGutter,
    )?;
    Ok(())
}

fn usable<T: Copy>(
    value: &DxfSemanticValue<T, DxfMTextColumnIssue>,
    role: DxfMTextEmbeddedColumnRole,
) -> Result<T, ProjectionFailure> {
    if let Some(value) = value.value() {
        Ok(*value)
    } else if let Some(issue) = value.invalid_issue() {
        Err((
            DxfMTextColumnRelationIssue::Scalar(*issue),
            value.raw_provenance(),
        ))
    } else {
        Err((DxfMTextColumnRelationIssue::MissingRequired { role }, None))
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
