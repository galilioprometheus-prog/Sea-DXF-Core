//! Exact HATCH extrusion tuples over reviewed scalar semantics.

use std::io;

use crate::{
    DxfAsciiRawDocument, DxfBinaryRawDocument, DxfCancellationToken, DxfDouble, DxfError,
    DxfHatchScalarEntry, DxfHatchScalarRole, DxfHatchScalarSemanticDirectory, DxfHatchScalarValue,
    DxfIoOperation, DxfRawDocumentView, DxfSemanticValueState, DxfSourceId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchExtrusionInputKind {
    Explicit,
    Defaulted,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchExtrusionComponent {
    value: DxfDouble,
    input_kind: DxfHatchExtrusionInputKind,
}

impl DxfHatchExtrusionComponent {
    #[must_use]
    pub const fn value(self) -> DxfDouble {
        self.value
    }

    #[must_use]
    pub const fn input_kind(self) -> DxfHatchExtrusionInputKind {
        self.input_kind
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchExtrusion {
    x: DxfHatchExtrusionComponent,
    y: DxfHatchExtrusionComponent,
    z: DxfHatchExtrusionComponent,
}

impl DxfHatchExtrusion {
    #[must_use]
    pub const fn x(self) -> DxfHatchExtrusionComponent {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> DxfHatchExtrusionComponent {
        self.y
    }

    #[must_use]
    pub const fn z(self) -> DxfHatchExtrusionComponent {
        self.z
    }

    #[must_use]
    pub const fn values(self) -> [DxfDouble; 3] {
        [self.x.value(), self.y.value(), self.z.value()]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchExtrusionUnavailableComponents {
    mask: u8,
}

impl DxfHatchExtrusionUnavailableComponents {
    const X: u8 = 1;
    const Y: u8 = 2;
    const Z: u8 = 4;

    #[must_use]
    pub const fn x(self) -> bool {
        self.mask & Self::X != 0
    }

    #[must_use]
    pub const fn y(self) -> bool {
        self.mask & Self::Y != 0
    }

    #[must_use]
    pub const fn z(self) -> bool {
        self.mask & Self::Z != 0
    }

    #[must_use]
    pub const fn count(self) -> u32 {
        self.mask.count_ones()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfHatchExtrusionIssue {
    ComponentsUnavailable(DxfHatchExtrusionUnavailableComponents),
    ZeroVector,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfHatchExtrusionEntry {
    ordinal: u32,
    scalar_entry: DxfHatchScalarEntry,
    extrusion: Result<DxfHatchExtrusion, DxfHatchExtrusionIssue>,
}

impl DxfHatchExtrusionEntry {
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal as u64
    }

    #[must_use]
    pub const fn scalar_entry(self) -> DxfHatchScalarEntry {
        self.scalar_entry
    }

    pub const fn extrusion(self) -> Result<DxfHatchExtrusion, DxfHatchExtrusionIssue> {
        self.extrusion
    }
}

/// One exact, non-normalized extrusion result per HATCH subclass.
#[derive(Debug)]
pub struct DxfHatchExtrusionDirectory {
    source_id: DxfSourceId,
    semantics: DxfHatchScalarSemanticDirectory,
    entries: Box<[DxfHatchExtrusionEntry]>,
}

impl DxfHatchExtrusionDirectory {
    fn from_document(
        document: DxfRawDocumentView<'_>,
        cancellation: &DxfCancellationToken,
    ) -> Result<Self, DxfError> {
        ensure_not_cancelled(cancellation)?;
        let semantics = document.hatch_scalar_semantic_directory(cancellation)?;
        ensure_source(document.source_id(), semantics.source_id())?;
        let mut entries = Vec::new();
        entries
            .try_reserve(
                semantics
                    .card_directory()
                    .evidence_directory()
                    .entries()
                    .len(),
            )
            .map_err(|_| out_of_memory())?;
        for scalar_entry in semantics
            .card_directory()
            .evidence_directory()
            .entries()
            .iter()
            .copied()
        {
            ensure_not_cancelled(cancellation)?;
            entries.push(DxfHatchExtrusionEntry {
                ordinal: compact_len(entries.len())?,
                scalar_entry,
                extrusion: extrusion_for(&semantics, scalar_entry)?,
            });
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Self {
            source_id: document.source_id(),
            semantics,
            entries: entries.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn semantic_directory(&self) -> &DxfHatchScalarSemanticDirectory {
        &self.semantics
    }

    #[must_use]
    pub fn entries(&self) -> &[DxfHatchExtrusionEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, ordinal: u64) -> Option<DxfHatchExtrusionEntry> {
        self.entries.get(usize::try_from(ordinal).ok()?).copied()
    }

    #[must_use]
    pub fn entry_for_subclass(&self, ordinal: u64) -> Option<DxfHatchExtrusionEntry> {
        self.entries
            .binary_search_by_key(&ordinal, |entry| entry.scalar_entry().subclass_ordinal())
            .ok()
            .and_then(|index| self.entries.get(index).copied())
    }

    #[must_use]
    pub fn entries_for_raw_record(&self, raw: u64) -> &[DxfHatchExtrusionEntry] {
        let start = self.entries.partition_point(|entry| {
            entry.scalar_entry().subclass().entity().record().ordinal() < raw
        });
        let end = self.entries.partition_point(|entry| {
            entry.scalar_entry().subclass().entity().record().ordinal() <= raw
        });
        self.entries.get(start..end).unwrap_or_default()
    }
}

impl DxfRawDocumentView<'_> {
    pub fn hatch_extrusion_directory(
        self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchExtrusionDirectory, DxfError> {
        DxfHatchExtrusionDirectory::from_document(self, cancellation)
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn hatch_extrusion_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchExtrusionDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_extrusion_directory(cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn hatch_extrusion_directory(
        &self,
        cancellation: &DxfCancellationToken,
    ) -> Result<DxfHatchExtrusionDirectory, DxfError> {
        DxfRawDocumentView::from(self).hatch_extrusion_directory(cancellation)
    }
}

fn extrusion_for(
    semantics: &DxfHatchScalarSemanticDirectory,
    scalar_entry: DxfHatchScalarEntry,
) -> Result<Result<DxfHatchExtrusion, DxfHatchExtrusionIssue>, DxfError> {
    let subclass = scalar_entry.subclass_ordinal();
    let x = component(semantics, subclass, DxfHatchScalarRole::ExtrusionX)?;
    let y = component(semantics, subclass, DxfHatchScalarRole::ExtrusionY)?;
    let z = component(semantics, subclass, DxfHatchScalarRole::ExtrusionZ)?;
    let mut mask = 0_u8;
    if x.is_none() {
        mask |= DxfHatchExtrusionUnavailableComponents::X;
    }
    if y.is_none() {
        mask |= DxfHatchExtrusionUnavailableComponents::Y;
    }
    if z.is_none() {
        mask |= DxfHatchExtrusionUnavailableComponents::Z;
    }
    if mask != 0 {
        return Ok(Err(DxfHatchExtrusionIssue::ComponentsUnavailable(
            DxfHatchExtrusionUnavailableComponents { mask },
        )));
    }
    let extrusion = DxfHatchExtrusion {
        x: x.ok_or_else(invalid_internal_data)?,
        y: y.ok_or_else(invalid_internal_data)?,
        z: z.ok_or_else(invalid_internal_data)?,
    };
    if extrusion
        .values()
        .into_iter()
        .all(|value| value.to_f64() == 0.0)
    {
        Ok(Err(DxfHatchExtrusionIssue::ZeroVector))
    } else {
        Ok(Ok(extrusion))
    }
}

fn component(
    semantics: &DxfHatchScalarSemanticDirectory,
    subclass: u64,
    role: DxfHatchScalarRole,
) -> Result<Option<DxfHatchExtrusionComponent>, DxfError> {
    let semantic = semantics
        .entry_for_role(subclass, role)
        .ok_or_else(invalid_internal_data)?;
    let input_kind = match semantic.semantic().state() {
        DxfSemanticValueState::Explicit => DxfHatchExtrusionInputKind::Explicit,
        DxfSemanticValueState::Defaulted => DxfHatchExtrusionInputKind::Defaulted,
        DxfSemanticValueState::Absent | DxfSemanticValueState::Invalid => return Ok(None),
    };
    let Some(DxfHatchScalarValue::Double(value)) = semantic.semantic().value().copied() else {
        return Err(invalid_internal_data());
    };
    if !value.is_finite() {
        return Err(invalid_internal_data());
    }
    Ok(Some(DxfHatchExtrusionComponent { value, input_kind }))
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
