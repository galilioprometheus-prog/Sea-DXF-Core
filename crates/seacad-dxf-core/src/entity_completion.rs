use crate::DxfEntityTopic;

/// Ordered evidence levels used by the DXF entity-completion subplan.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DxfEntityCompletionLevel {
    ExactEvidence = 1,
    Cardinality = 2,
    TypedSemantics = 3,
    Geometry = 4,
    VerifiedMutation = 5,
    ReleaseQualified = 6,
}

/// Release evidence that prevents an audited entity from reaching level 6.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum DxfEntityCompletionBlocker {
    PrivateCorpusQualification,
    CurrentCheckpointSixNativeCi,
}

/// One explicitly audited entity-family completion assessment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DxfEntityCompletionAssessment {
    topic: DxfEntityTopic,
    achieved_level: DxfEntityCompletionLevel,
    blockers: &'static [DxfEntityCompletionBlocker],
    audited_checkpoint: &'static str,
}

impl DxfEntityCompletionAssessment {
    const fn new(
        topic: DxfEntityTopic,
        achieved_level: DxfEntityCompletionLevel,
        blockers: &'static [DxfEntityCompletionBlocker],
        audited_checkpoint: &'static str,
    ) -> Self {
        Self {
            topic,
            achieved_level,
            blockers,
            audited_checkpoint,
        }
    }

    #[must_use]
    pub const fn topic(self) -> DxfEntityTopic {
        self.topic
    }

    #[must_use]
    pub const fn achieved_level(self) -> DxfEntityCompletionLevel {
        self.achieved_level
    }

    #[must_use]
    pub const fn blockers(self) -> &'static [DxfEntityCompletionBlocker] {
        self.blockers
    }

    #[must_use]
    pub const fn audited_checkpoint(self) -> &'static str {
        self.audited_checkpoint
    }

    #[must_use]
    pub const fn satisfies(self, required: DxfEntityCompletionLevel) -> bool {
        self.achieved_level as u8 >= required as u8
    }

    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.satisfies(DxfEntityCompletionLevel::ReleaseQualified) && self.blockers.is_empty()
    }
}

const POINT_BLOCKERS: &[DxfEntityCompletionBlocker] = &[
    DxfEntityCompletionBlocker::PrivateCorpusQualification,
    DxfEntityCompletionBlocker::CurrentCheckpointSixNativeCi,
];

/// Audited assessments only. A missing topic is unaudited, not unsupported.
pub static DXF_ENTITY_COMPLETION_ASSESSMENTS: &[DxfEntityCompletionAssessment] =
    &[DxfEntityCompletionAssessment::new(
        DxfEntityTopic::POINT,
        DxfEntityCompletionLevel::VerifiedMutation,
        POINT_BLOCKERS,
        "m14.3dz-point-completion-ledger",
    )];

/// Returns the explicit assessment for `topic`, or `None` when it is unaudited.
#[must_use]
pub fn dxf_entity_completion_assessment(
    topic: DxfEntityTopic,
) -> Option<&'static DxfEntityCompletionAssessment> {
    DXF_ENTITY_COMPLETION_ASSESSMENTS
        .iter()
        .find(|assessment| assessment.topic() == topic)
}
