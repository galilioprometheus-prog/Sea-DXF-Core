use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::mem::size_of;

use seacad_dxf_core::{
    DXF_ENTITY_COMPLETION_ASSESSMENTS, DXF_ENTITY_TOPICS, DxfEntityCompletionAssessment,
    DxfEntityCompletionBlocker, DxfEntityCompletionLevel, DxfEntityTopic,
    dxf_entity_completion_assessment,
};

fn assert_stable_traits<T: Copy + Debug + Eq + Hash + Send + Sync>() {}

fn assessed(topic: DxfEntityTopic) -> DxfEntityCompletionAssessment {
    let assessment = dxf_entity_completion_assessment(topic);
    assert!(assessment.is_some());
    assessment
        .copied()
        .unwrap_or(DXF_ENTITY_COMPLETION_ASSESSMENTS[0])
}

#[test]
fn point_is_audited_at_verified_mutation_only() {
    let assessment = dxf_entity_completion_assessment(DxfEntityTopic::POINT);
    assert!(assessment.is_some());
    let assessment = assessment.copied();

    assert_eq!(
        assessment.map(DxfEntityCompletionAssessment::achieved_level),
        Some(DxfEntityCompletionLevel::VerifiedMutation)
    );
    assert_eq!(
        assessment.map(DxfEntityCompletionAssessment::audited_checkpoint),
        Some("m14.3dz-point-completion-ledger")
    );
    assert_eq!(
        assessment.map(DxfEntityCompletionAssessment::is_complete),
        Some(false)
    );
}

#[test]
fn point_satisfies_exactly_levels_one_through_five() {
    let assessment = assessed(DxfEntityTopic::POINT);

    for level in [
        DxfEntityCompletionLevel::ExactEvidence,
        DxfEntityCompletionLevel::Cardinality,
        DxfEntityCompletionLevel::TypedSemantics,
        DxfEntityCompletionLevel::Geometry,
        DxfEntityCompletionLevel::VerifiedMutation,
    ] {
        assert!(assessment.satisfies(level));
    }
    assert!(!assessment.satisfies(DxfEntityCompletionLevel::ReleaseQualified));
}

#[test]
fn point_retains_the_exact_release_evidence_blockers() {
    let assessment = assessed(DxfEntityTopic::POINT);

    assert_eq!(
        assessment.blockers(),
        &[
            DxfEntityCompletionBlocker::PrivateCorpusQualification,
            DxfEntityCompletionBlocker::CurrentCheckpointSixNativeCi,
        ]
    );
}

#[test]
fn spline_is_geometry_ready_but_not_verified_for_mutation() {
    let assessment = assessed(DxfEntityTopic::SPLINE);

    assert_eq!(
        assessment.achieved_level(),
        DxfEntityCompletionLevel::Geometry
    );
    assert!(assessment.satisfies(DxfEntityCompletionLevel::Geometry));
    assert!(!assessment.satisfies(DxfEntityCompletionLevel::VerifiedMutation));
    assert_eq!(
        assessment.blockers(),
        &[
            DxfEntityCompletionBlocker::VerifiedMutation,
            DxfEntityCompletionBlocker::PrivateCorpusQualification,
            DxfEntityCompletionBlocker::CurrentCheckpointSixNativeCi,
        ]
    );
    assert!(!assessment.is_complete());
}

#[test]
fn helix_stops_at_typed_semantics_without_public_geometry_qualification() {
    let assessment = assessed(DxfEntityTopic::HELIX);

    assert_eq!(
        assessment.achieved_level(),
        DxfEntityCompletionLevel::TypedSemantics
    );
    assert!(!assessment.satisfies(DxfEntityCompletionLevel::Geometry));
    assert_eq!(
        assessment.blockers(),
        &[
            DxfEntityCompletionBlocker::PublicGeometryQualification,
            DxfEntityCompletionBlocker::VerifiedMutation,
            DxfEntityCompletionBlocker::PrivateCorpusQualification,
            DxfEntityCompletionBlocker::CurrentCheckpointSixNativeCi,
        ]
    );
    assert!(!assessment.is_complete());
}

#[test]
fn every_unaudited_public_topic_remains_explicit() {
    let unaudited_count = DXF_ENTITY_TOPICS
        .iter()
        .filter(|descriptor| dxf_entity_completion_assessment(descriptor.topic()).is_none())
        .count();

    assert_eq!(DXF_ENTITY_TOPICS.len(), 45);
    assert_eq!(unaudited_count, 42);
}

#[test]
fn audited_topic_lookup_is_unique_and_deterministic() {
    let mut topics = HashSet::new();
    for assessment in DXF_ENTITY_COMPLETION_ASSESSMENTS {
        assert!(topics.insert(assessment.topic()));
        assert_eq!(
            dxf_entity_completion_assessment(assessment.topic()),
            Some(assessment)
        );
    }

    assert_eq!(DXF_ENTITY_COMPLETION_ASSESSMENTS.len(), 3);
    assert!(
        DXF_ENTITY_COMPLETION_ASSESSMENTS
            .windows(2)
            .all(|pair| pair[0].topic().ordinal() < pair[1].topic().ordinal())
    );
}

#[test]
fn completion_metadata_has_stable_traits_and_compact_storage() {
    assert_stable_traits::<DxfEntityCompletionLevel>();
    assert_stable_traits::<DxfEntityCompletionBlocker>();
    assert_stable_traits::<DxfEntityCompletionAssessment>();

    assert_eq!(size_of::<DxfEntityCompletionLevel>(), 1);
    assert_eq!(size_of::<DxfEntityCompletionBlocker>(), 1);
    assert!(size_of::<DxfEntityCompletionAssessment>() <= 48);
}
