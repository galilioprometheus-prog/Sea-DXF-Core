//! Fail-closed dialect admission for one prepared entity draft identity.

use std::io;

use crate::{
    DxfAcadVersion, DxfAcadVersionState, DxfAsciiRawDocument, DxfBinaryRawDocument,
    DxfCancellationToken, DxfEntityApplicability, DxfEntityApplicabilityDescriptor,
    DxfEntityDraftIdentityPlan, DxfEntityDraftName, DxfEntityNameClassification, DxfError,
    DxfHandle, DxfIoOperation, DxfNamedSymbolTableEntry, DxfRawDocumentView, DxfSourceId,
    DxfTransactionPlan,
};

/// Typed reason why a prepared entity name cannot be admitted for one document.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfEntityDraftApplicabilityIssue {
    VersionUnavailable {
        state: DxfAcadVersionState,
    },
    NotApplicable {
        classification: DxfEntityNameClassification,
        version: DxfAcadVersion,
        minimum_version: Option<DxfAcadVersion>,
        maximum_version: Option<DxfAcadVersion>,
    },
    NotYetReviewed {
        classification: DxfEntityNameClassification,
        version: DxfAcadVersion,
    },
}

/// One prepared draft identity admitted by reviewed generated applicability.
#[derive(Debug)]
pub struct DxfEntityDraftApplicabilityPlan {
    identity: DxfEntityDraftIdentityPlan,
    version: DxfAcadVersion,
    descriptor: &'static DxfEntityApplicabilityDescriptor,
}

impl DxfEntityDraftApplicabilityPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.identity.source_id()
    }

    #[must_use]
    pub const fn name(&self) -> DxfEntityDraftName {
        self.identity.name()
    }

    #[must_use]
    pub const fn classification(&self) -> DxfEntityNameClassification {
        self.identity.name().classification()
    }

    #[must_use]
    pub const fn version(&self) -> DxfAcadVersion {
        self.version
    }

    #[must_use]
    pub const fn descriptor(&self) -> &'static DxfEntityApplicabilityDescriptor {
        self.descriptor
    }

    #[must_use]
    pub const fn handle(&self) -> DxfHandle {
        self.identity.handle()
    }

    #[must_use]
    pub const fn owner_block_record(&self) -> DxfNamedSymbolTableEntry {
        self.identity.owner_block_record()
    }

    #[must_use]
    pub const fn transaction(&self) -> &DxfTransactionPlan {
        self.identity.transaction()
    }

    #[must_use]
    pub const fn identity(&self) -> &DxfEntityDraftIdentityPlan {
        &self.identity
    }

    #[must_use]
    pub fn into_identity(self) -> DxfEntityDraftIdentityPlan {
        self.identity
    }
}

impl DxfRawDocumentView<'_> {
    /// Admits a prepared identity only when its generated name is reviewed for
    /// the document's single supported dialect declaration.
    pub fn prepare_entity_draft_applicability(
        self,
        identity: DxfEntityDraftIdentityPlan,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityDraftApplicabilityPlan, DxfEntityDraftApplicabilityIssue>, DxfError>
    {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), identity.source_id())?;
        identity.transaction().validate_source_precondition(self)?;
        let version = match self.acad_version_report().state() {
            DxfAcadVersionState::Supported(version) => version,
            state => {
                return Ok(Err(DxfEntityDraftApplicabilityIssue::VersionUnavailable {
                    state,
                }));
            }
        };
        let classification = identity.name().classification();
        let descriptor = classification
            .applicability_descriptor()
            .ok_or_else(invalid_internal_data)?;
        if descriptor.classification() != classification {
            return Err(invalid_internal_data());
        }
        match descriptor.applicability(version) {
            DxfEntityApplicability::Applicable => {}
            DxfEntityApplicability::NotApplicable => {
                return Ok(Err(DxfEntityDraftApplicabilityIssue::NotApplicable {
                    classification,
                    version,
                    minimum_version: descriptor.minimum_version(),
                    maximum_version: descriptor.maximum_version(),
                }));
            }
            DxfEntityApplicability::NotYetReviewed => {
                return Ok(Err(DxfEntityDraftApplicabilityIssue::NotYetReviewed {
                    classification,
                    version,
                }));
            }
        }
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(DxfEntityDraftApplicabilityPlan {
            identity,
            version,
            descriptor,
        }))
    }
}

impl DxfAsciiRawDocument<'_> {
    pub fn prepare_entity_draft_applicability(
        &self,
        identity: DxfEntityDraftIdentityPlan,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityDraftApplicabilityPlan, DxfEntityDraftApplicabilityIssue>, DxfError>
    {
        DxfRawDocumentView::from(self).prepare_entity_draft_applicability(identity, cancellation)
    }
}

impl DxfBinaryRawDocument<'_> {
    pub fn prepare_entity_draft_applicability(
        &self,
        identity: DxfEntityDraftIdentityPlan,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfEntityDraftApplicabilityPlan, DxfEntityDraftApplicabilityIssue>, DxfError>
    {
        DxfRawDocumentView::from(self).prepare_entity_draft_applicability(identity, cancellation)
    }
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
    DxfError::from_io(
        DxfIoOperation::Write,
        &io::Error::from(io::ErrorKind::InvalidData),
    )
}
