//! Cross-document POINT semantic projection into one destination draft record.

use std::io;

use crate::entity_edit_session::{PointCloneSnapshot, prepare_point_clone_snapshot};
use crate::{
    DxfAcadVersion, DxfCancellationToken, DxfEntityCloneIssue, DxfEntityDraftApplicabilityPlan,
    DxfEntityDraftRecordIssue, DxfEntityDraftRecordPlan, DxfEntityField, DxfEntityKey,
    DxfEntityLineweight, DxfEntityPlacementTarget, DxfError, DxfHandle, DxfIoOperation,
    DxfRawDocumentView, DxfResourceProfile, DxfSourceId, DxfTextTranscodeIssue,
    DxfTextTranscodePlan, DxfTextTranscodeReceipt,
};

/// Caller-owned destination bindings for common fields whose identities are
/// document-local.
#[derive(Clone, Copy)]
pub struct DxfPointCloneDestinationBindings<'a> {
    layer: &'a [u8],
    layout: Option<&'a [u8]>,
    linetype: Option<&'a [u8]>,
    material: Option<DxfHandle>,
    plot_style: Option<DxfHandle>,
    xdata_composition: bool,
}

impl<'a> DxfPointCloneDestinationBindings<'a> {
    #[must_use]
    pub const fn new(layer: &'a [u8]) -> Self {
        Self {
            layer,
            layout: None,
            linetype: None,
            material: None,
            plot_style: None,
            xdata_composition: false,
        }
    }

    #[must_use]
    pub const fn with_layout(mut self, layout: &'a [u8]) -> Self {
        self.layout = Some(layout);
        self
    }

    #[must_use]
    pub const fn with_linetype(mut self, linetype: &'a [u8]) -> Self {
        self.linetype = Some(linetype);
        self
    }

    #[must_use]
    pub const fn with_material(mut self, material: DxfHandle) -> Self {
        self.material = Some(material);
        self
    }

    #[must_use]
    pub const fn with_plot_style(mut self, plot_style: DxfHandle) -> Self {
        self.plot_style = Some(plot_style);
        self
    }

    #[must_use]
    pub const fn layer(self) -> &'a [u8] {
        self.layer
    }

    #[must_use]
    pub const fn layout(self) -> Option<&'a [u8]> {
        self.layout
    }

    #[must_use]
    pub const fn linetype(self) -> Option<&'a [u8]> {
        self.linetype
    }

    #[must_use]
    pub const fn material(self) -> Option<DxfHandle> {
        self.material
    }

    #[must_use]
    pub const fn plot_style(self) -> Option<DxfHandle> {
        self.plot_style
    }

    pub(crate) const fn with_xdata_composition(mut self) -> Self {
        self.xdata_composition = true;
        self
    }

    pub(crate) const fn includes_xdata_composition(self) -> bool {
        self.xdata_composition
    }
}

impl std::fmt::Debug for DxfPointCloneDestinationBindings<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DxfPointCloneDestinationBindings")
            .field("layer_byte_count", &self.layer.len())
            .field("has_layout", &self.layout.is_some())
            .field("has_linetype", &self.linetype.is_some())
            .field("has_material", &self.material.is_some())
            .field("has_plot_style", &self.plot_style.is_some())
            .finish()
    }
}

/// Exact destination-dialect adaptations applied without changing POINT
/// semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfPointCloneDialectAdaptations(u8);

impl DxfPointCloneDialectAdaptations {
    const LEGACY_PLACEMENT_LAYOUT: u8 = 1;
    const OMITTED_BY_LAYER_LINEWEIGHT: u8 = 2;

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn legacy_placement_owns_layout(self) -> bool {
        self.0 & Self::LEGACY_PLACEMENT_LAYOUT != 0
    }

    #[must_use]
    pub const fn omitted_by_layer_lineweight(self) -> bool {
        self.0 & Self::OMITTED_BY_LAYER_LINEWEIGHT != 0
    }
}

/// Typed reason why a source POINT cannot become a destination draft record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DxfPointCloneDraftProjectionIssue {
    SameDocument {
        source_id: DxfSourceId,
    },
    Source(DxfEntityCloneIssue),
    DestinationBindingRequired {
        field: DxfEntityField,
        source_handle: Option<DxfHandle>,
    },
    UnexpectedDestinationBinding {
        field: DxfEntityField,
    },
    DestinationFieldNotRepresentable {
        field: DxfEntityField,
        source_version: DxfAcadVersion,
        destination_version: DxfAcadVersion,
    },
    TextTranscode {
        field: DxfEntityField,
        issue: DxfTextTranscodeIssue,
    },
    Destination(DxfEntityDraftRecordIssue),
}

/// One source semantic snapshot projected into a canonical destination POINT
/// draft record without insertion or XDATA composition.
pub struct DxfPointCloneDraftProjectionPlan {
    source_id: DxfSourceId,
    source_key: DxfEntityKey,
    source_version: DxfAcadVersion,
    source_placement: DxfEntityPlacementTarget,
    source_owner: Option<DxfHandle>,
    adaptations: DxfPointCloneDialectAdaptations,
    color_name_transcode: Option<DxfTextTranscodeReceipt>,
    destination: DxfEntityDraftRecordPlan,
}

impl DxfPointCloneDraftProjectionPlan {
    #[must_use]
    pub const fn source_id(&self) -> DxfSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn source_key(&self) -> DxfEntityKey {
        self.source_key
    }

    #[must_use]
    pub const fn source_version(&self) -> DxfAcadVersion {
        self.source_version
    }

    #[must_use]
    pub const fn source_placement(&self) -> DxfEntityPlacementTarget {
        self.source_placement
    }

    #[must_use]
    pub const fn source_owner(&self) -> Option<DxfHandle> {
        self.source_owner
    }

    #[must_use]
    pub const fn dialect_adaptations(&self) -> DxfPointCloneDialectAdaptations {
        self.adaptations
    }

    #[must_use]
    pub const fn color_name_transcode(&self) -> Option<DxfTextTranscodeReceipt> {
        self.color_name_transcode
    }

    #[must_use]
    pub const fn destination_id(&self) -> DxfSourceId {
        self.destination.source_id()
    }

    #[must_use]
    pub const fn destination(&self) -> &DxfEntityDraftRecordPlan {
        &self.destination
    }

    #[must_use]
    pub fn into_destination(self) -> DxfEntityDraftRecordPlan {
        self.destination
    }
}

impl std::fmt::Debug for DxfPointCloneDraftProjectionPlan {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DxfPointCloneDraftProjectionPlan")
            .field("source_id", &self.source_id)
            .field("source_key", &self.source_key)
            .field("source_version", &self.source_version)
            .field("source_placement", &self.source_placement)
            .field("has_source_owner", &self.source_owner.is_some())
            .field("adaptations", &self.adaptations)
            .field(
                "has_color_name_transcode",
                &self.color_name_transcode.is_some(),
            )
            .field("destination", &self.destination)
            .finish()
    }
}

impl DxfRawDocumentView<'_> {
    /// Projects one exact source POINT snapshot into this destination's
    /// canonical family record. The returned record is not inserted.
    pub fn project_point_clone_draft_from(
        self,
        source: DxfRawDocumentView<'_>,
        source_key: DxfEntityKey,
        destination_applicability: DxfEntityDraftApplicabilityPlan,
        bindings: DxfPointCloneDestinationBindings<'_>,
        profile: DxfResourceProfile,
        cancellation: &DxfCancellationToken,
    ) -> Result<Result<DxfPointCloneDraftProjectionPlan, DxfPointCloneDraftProjectionIssue>, DxfError>
    {
        ensure_not_cancelled(cancellation)?;
        ensure_source(self.source_id(), destination_applicability.source_id())?;
        if self.source_id() == source.source_id() {
            return Ok(Err(DxfPointCloneDraftProjectionIssue::SameDocument {
                source_id: source.source_id(),
            }));
        }
        let source_evidence = source.entity_field_evidence_directory(cancellation)?;
        let snapshot = match prepare_point_clone_snapshot(
            source,
            &source_evidence,
            source_key,
            None,
            bindings.includes_xdata_composition(),
            profile,
            cancellation,
        )? {
            Ok(snapshot) => snapshot,
            Err(issue) => return Ok(Err(DxfPointCloneDraftProjectionIssue::Source(issue))),
        };
        if let Some(issue) = validate_bindings(&snapshot, bindings) {
            return Ok(Err(issue));
        }
        let (lineweight, adaptations) =
            match destination_common_adaptations(&snapshot, destination_applicability.version()) {
                DestinationCommonAdaptation::Ready {
                    lineweight,
                    adaptations,
                } => (lineweight, adaptations),
                DestinationCommonAdaptation::NotRepresentable { field } => {
                    return Ok(Err(
                        DxfPointCloneDraftProjectionIssue::DestinationFieldNotRepresentable {
                            field,
                            source_version: snapshot.version(),
                            destination_version: destination_applicability.version(),
                        },
                    ));
                }
            };
        let color_name_transcode =
            match transcode_color_name(source, self, &snapshot, profile, cancellation)? {
                Ok(plan) => plan,
                Err(issue) => {
                    return Ok(Err(DxfPointCloneDraftProjectionIssue::TextTranscode {
                        field: DxfEntityField::COLOR_NAME,
                        issue,
                    }));
                }
            };
        let color_name = color_name_transcode
            .as_ref()
            .map(|plan| plan.encoded_bytes())
            .or_else(|| snapshot.color_name().map(|value| value.bytes()));
        let draft = snapshot.borrowed_for_destination(
            destination_applicability.identity().owner_handle(),
            lineweight,
            bindings,
            color_name,
        );
        let destination = match self.encode_entity_draft_record(
            destination_applicability,
            draft,
            profile,
            cancellation,
        )? {
            Ok(plan) => plan,
            Err(issue) => {
                return Ok(Err(DxfPointCloneDraftProjectionIssue::Destination(issue)));
            }
        };
        ensure_not_cancelled(cancellation)?;
        Ok(Ok(DxfPointCloneDraftProjectionPlan {
            source_id: snapshot.source_id(),
            source_key: snapshot.key(),
            source_version: snapshot.version(),
            source_placement: snapshot.placement(),
            source_owner: snapshot.owner(),
            adaptations,
            color_name_transcode: color_name_transcode
                .as_ref()
                .map(DxfTextTranscodePlan::receipt),
            destination,
        }))
    }
}

enum DestinationCommonAdaptation {
    Ready {
        lineweight: Option<DxfEntityLineweight>,
        adaptations: DxfPointCloneDialectAdaptations,
    },
    NotRepresentable {
        field: DxfEntityField,
    },
}

fn destination_common_adaptations(
    snapshot: &PointCloneSnapshot,
    destination_version: DxfAcadVersion,
) -> DestinationCommonAdaptation {
    if destination_version >= DxfAcadVersion::Ac1015 {
        return DestinationCommonAdaptation::Ready {
            lineweight: snapshot
                .lineweight()
                .or(Some(DxfEntityLineweight::BY_LAYER)),
            adaptations: DxfPointCloneDialectAdaptations(0),
        };
    }
    if snapshot.color_name().is_some() {
        return DestinationCommonAdaptation::NotRepresentable {
            field: DxfEntityField::COLOR_NAME,
        };
    }
    let mut adaptations = 0_u8;
    if snapshot.has_layout() {
        adaptations |= DxfPointCloneDialectAdaptations::LEGACY_PLACEMENT_LAYOUT;
    }
    match snapshot.lineweight() {
        None => {}
        Some(DxfEntityLineweight::BY_LAYER) => {
            adaptations |= DxfPointCloneDialectAdaptations::OMITTED_BY_LAYER_LINEWEIGHT;
        }
        Some(_) => {
            return DestinationCommonAdaptation::NotRepresentable {
                field: DxfEntityField::LINEWEIGHT,
            };
        }
    }
    DestinationCommonAdaptation::Ready {
        lineweight: None,
        adaptations: DxfPointCloneDialectAdaptations(adaptations),
    }
}

fn transcode_color_name(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
    snapshot: &PointCloneSnapshot,
    profile: DxfResourceProfile,
    cancellation: &DxfCancellationToken,
) -> Result<Result<Option<DxfTextTranscodePlan>, DxfTextTranscodeIssue>, DxfError> {
    let Some(color_name) = snapshot.color_name() else {
        return Ok(Ok(None));
    };
    let source_text = color_name.source();
    if source_text.source_id() != source.source_id()
        || source_text.encoding() != source.text_encoding_report().resolution()
    {
        return Err(invalid_internal_data());
    }
    if portable_text(source, destination, color_name.bytes()) {
        return Ok(Ok(None));
    }
    let plan = match source.transcode_text_span_to(
        source_text.value_span(),
        destination,
        profile,
        cancellation,
    )? {
        Ok(plan) => plan,
        Err(issue) => return Ok(Err(issue)),
    };
    let receipt = plan.receipt();
    if receipt.source_id() != source.source_id()
        || receipt.destination_id() != destination.source_id()
        || receipt.source_span() != source_text.value_span()
        || receipt.source_encoding() != source_text.encoding()
    {
        return Err(invalid_internal_data());
    }
    Ok(Ok(Some(plan)))
}

fn portable_text(
    source: DxfRawDocumentView<'_>,
    destination: DxfRawDocumentView<'_>,
    bytes: &[u8],
) -> bool {
    bytes.is_ascii()
        || source.text_encoding_report().resolution().decoder()
            == destination.text_encoding_report().resolution().decoder()
            && source
                .text_encoding_report()
                .resolution()
                .decoder()
                .is_some()
}

fn validate_bindings(
    snapshot: &PointCloneSnapshot,
    bindings: DxfPointCloneDestinationBindings<'_>,
) -> Option<DxfPointCloneDraftProjectionIssue> {
    if snapshot.has_linetype() != bindings.linetype().is_some() {
        return Some(if snapshot.has_linetype() {
            DxfPointCloneDraftProjectionIssue::DestinationBindingRequired {
                field: DxfEntityField::LINETYPE,
                source_handle: None,
            }
        } else {
            DxfPointCloneDraftProjectionIssue::UnexpectedDestinationBinding {
                field: DxfEntityField::LINETYPE,
            }
        });
    }
    for (field, source_handle, destination_handle) in [
        (
            DxfEntityField::MATERIAL,
            snapshot.material(),
            bindings.material(),
        ),
        (
            DxfEntityField::PLOT_STYLE,
            snapshot.plot_style(),
            bindings.plot_style(),
        ),
    ] {
        if source_handle.is_some() != destination_handle.is_some() {
            return Some(if source_handle.is_some() {
                DxfPointCloneDraftProjectionIssue::DestinationBindingRequired {
                    field,
                    source_handle,
                }
            } else {
                DxfPointCloneDraftProjectionIssue::UnexpectedDestinationBinding { field }
            });
        }
    }
    None
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
