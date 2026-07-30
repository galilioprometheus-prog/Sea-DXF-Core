//! Bounded rectangular-array placement for eligible INSERT transforms.

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfInsertAffineTransform, DxfInsertTransformApplicationIssue, DxfInsertTransformDirectory,
    DxfInsertTransformEntry, DxfInsertTransformIssue, DxfRawDocumentView, DxfSourceId,
    insert_transform::OcsBasis,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertArrayIssue {
    TransformUnavailable(DxfInsertTransformIssue),
    ArraySemanticsUnavailable,
    NonPositiveColumnCount { value: i16 },
    NonPositiveRowCount { value: i16 },
    NonFiniteSpacing,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfInsertArrayApplicationIssue {
    NonFinitePlacement,
}

/// One row/column member of an INSERT rectangular array.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertArrayInstance {
    column_index: u16,
    row_index: u16,
    transform: DxfInsertAffineTransform,
}

impl DxfInsertArrayInstance {
    #[must_use]
    pub const fn column_index(self) -> u16 {
        self.column_index
    }

    #[must_use]
    pub const fn row_index(self) -> u16 {
        self.row_index
    }

    #[must_use]
    pub const fn transform(self) -> DxfInsertAffineTransform {
        self.transform
    }
}

/// Constant-space rectangular-array layout for one INSERT record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertArrayLayout {
    base_transform: DxfInsertAffineTransform,
    column_count: u16,
    row_count: u16,
    column_step_wcs: [DxfDouble; 3],
    row_step_wcs: [DxfDouble; 3],
}

impl DxfInsertArrayLayout {
    #[must_use]
    pub const fn base_transform(self) -> DxfInsertAffineTransform {
        self.base_transform
    }

    #[must_use]
    pub const fn column_count(self) -> u16 {
        self.column_count
    }

    #[must_use]
    pub const fn row_count(self) -> u16 {
        self.row_count
    }

    #[must_use]
    pub const fn instance_count(self) -> u64 {
        self.column_count as u64 * self.row_count as u64
    }

    #[must_use]
    pub const fn column_step_wcs(self) -> [DxfDouble; 3] {
        self.column_step_wcs
    }

    #[must_use]
    pub const fn row_step_wcs(self) -> [DxfDouble; 3] {
        self.row_step_wcs
    }

    pub fn instance(
        self,
        column_index: u16,
        row_index: u16,
    ) -> Result<Option<DxfInsertArrayInstance>, DxfInsertArrayApplicationIssue> {
        if column_index >= self.column_count || row_index >= self.row_count {
            return Ok(None);
        }
        let column = f64::from(column_index);
        let row = f64::from(row_index);
        let column_step = self.column_step_wcs.map(DxfDouble::to_f64);
        let row_step = self.row_step_wcs.map(DxfDouble::to_f64);
        let offset = [
            column_step[0] * column + row_step[0] * row,
            column_step[1] * column + row_step[1] * row,
            column_step[2] * column + row_step[2] * row,
        ];
        let transform = self
            .base_transform
            .offset_wcs(offset)
            .map_err(map_application_issue)?;
        Ok(Some(DxfInsertArrayInstance {
            column_index,
            row_index,
            transform,
        }))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfInsertArrayEntry {
    transform_entry: DxfInsertTransformEntry,
    layout: Result<DxfInsertArrayLayout, DxfInsertArrayIssue>,
}

impl DxfInsertArrayEntry {
    #[must_use]
    pub const fn transform_entry(self) -> DxfInsertTransformEntry {
        self.transform_entry
    }

    pub const fn layout(self) -> Result<DxfInsertArrayLayout, DxfInsertArrayIssue> {
        self.layout
    }
}

#[derive(Debug)]
pub struct DxfInsertArrayDirectory {
    source_id: DxfSourceId,
    transforms: DxfInsertTransformDirectory,
    entries: Box<[DxfInsertArrayEntry]>,
}

impl DxfInsertArrayDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let transforms = document.insert_transform_directory(cancellation)?;
        if transforms.source_id() != document.source_id() {
            return Err(DxfError::SourceIdentityMismatch {
                expected: document.source_id(),
                observed: transforms.source_id(),
            });
        }
        let mut entries = Vec::new();
        entries
            .try_reserve(transforms.entries().len())
            .map_err(|_| out_of_memory())?;
        for transform_entry in transforms.entries().iter().copied() {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfInsertArrayEntry {
                transform_entry,
                layout: derive_layout(&transforms, transform_entry)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            transforms,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn transform_directory(&self) -> &DxfInsertTransformDirectory {
        &self.transforms
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfInsertArrayEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry_for_insert_raw_ordinal(
        &self,
        raw_record_ordinal: u64,
    ) -> Option<DxfInsertArrayEntry> {
        self.entries
            .binary_search_by_key(&raw_record_ordinal, |entry| {
                entry
                    .transform_entry()
                    .eligibility()
                    .resolution()
                    .insert()
                    .record()
                    .ordinal()
            })
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }
}

impl DxfRawDocumentView<'_> {
    pub fn insert_array_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertArrayDirectory, DxfError> {
        DxfInsertArrayDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn insert_array_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertArrayDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_array_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn insert_array_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfInsertArrayDirectory, DxfError> {
        DxfRawDocumentView::from(self).insert_array_directory(cancellation)
    }
}

fn derive_layout(
    transforms: &DxfInsertTransformDirectory,
    entry: DxfInsertTransformEntry,
) -> Result<Result<DxfInsertArrayLayout, DxfInsertArrayIssue>, DxfError> {
    let base_transform = match entry.transform() {
        Ok(transform) => transform,
        Err(issue) => {
            return Ok(Err(DxfInsertArrayIssue::TransformUnavailable(issue)));
        }
    };
    let insert = entry.eligibility().resolution().insert();
    let semantics = transforms
        .eligibility_directory()
        .resolution_directory()
        .insert_semantic_directory()
        .semantics_for_entry(insert)?;
    let Some(semantics) = semantics else {
        return Err(invalid_internal_data());
    };
    let Some([column_count, row_count]) = semantics.array_count_values() else {
        return Ok(Err(DxfInsertArrayIssue::ArraySemanticsUnavailable));
    };
    let Some(spacing) = semantics.array_spacing_values() else {
        return Ok(Err(DxfInsertArrayIssue::ArraySemanticsUnavailable));
    };
    let Some(rotation) = semantics.rotation_degrees_value() else {
        return Err(invalid_internal_data());
    };
    let Some(extrusion) = semantics.extrusion_value() else {
        return Err(invalid_internal_data());
    };
    if column_count <= 0 {
        return Ok(Err(DxfInsertArrayIssue::NonPositiveColumnCount {
            value: column_count,
        }));
    }
    if row_count <= 0 {
        return Ok(Err(DxfInsertArrayIssue::NonPositiveRowCount {
            value: row_count,
        }));
    }
    let spacing = spacing.map(DxfDouble::to_f64);
    if !spacing.iter().all(|value| value.is_finite()) {
        return Ok(Err(DxfInsertArrayIssue::NonFiniteSpacing));
    }
    let extrusion = extrusion.map(DxfDouble::to_f64);
    let basis = OcsBasis::from_extrusion(extrusion).map_err(|_| invalid_internal_data())?;
    let axes = basis.rotated_xy(rotation.to_f64().to_radians());
    let steps = [
        axes[0].map(|component| component * spacing[0]),
        axes[1].map(|component| component * spacing[1]),
    ];
    Ok(Ok(DxfInsertArrayLayout {
        base_transform,
        column_count: u16::try_from(column_count).map_err(|_| invalid_internal_data())?,
        row_count: u16::try_from(row_count).map_err(|_| invalid_internal_data())?,
        column_step_wcs: steps[0].map(canonical_double),
        row_step_wcs: steps[1].map(canonical_double),
    }))
}

fn map_application_issue(
    _issue: DxfInsertTransformApplicationIssue,
) -> DxfInsertArrayApplicationIssue {
    DxfInsertArrayApplicationIssue::NonFinitePlacement
}

fn canonical_double(value: f64) -> DxfDouble {
    DxfDouble::from_f64(if value == 0.0 { 0.0 } else { value })
}

fn ensure_not_cancelled(cancellation: &DxfCancellationToken) -> Result<(), DxfError> {
    if cancellation.is_cancelled() {
        Err(DxfError::Cancelled)
    } else {
        Ok(())
    }
}

fn invalid_internal_data() -> DxfError {
    DxfError::from_io(
        crate::DxfIoOperation::Read,
        &std::io::Error::from(std::io::ErrorKind::InvalidData),
    )
}

fn out_of_memory() -> DxfError {
    DxfError::from_io(
        crate::DxfIoOperation::Read,
        &std::io::Error::from(std::io::ErrorKind::OutOfMemory),
    )
}
