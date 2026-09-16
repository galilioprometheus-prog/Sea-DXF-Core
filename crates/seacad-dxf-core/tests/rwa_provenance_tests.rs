use seacad_dxf_core::{compute_rwa_from_evidence, RwaComputationError, RwaEvidence, RwaModellability, RwaPolicy};

fn evidence(factor_id: &str, exposure_units: u64) -> RwaEvidence {
    RwaEvidence {
        factor_id: factor_id.to_owned(),
        exposure_units,
        risk_weight_bps: 5000,
        horizon_multiplier_bps: 10_000,
        modellability: RwaModellability::Modellable,
        source_work_id: "work-14".to_owned(),
        source_work_version: "1".to_owned(),
        evidence_version: "e1".to_owned(),
    }
}

fn policy() -> RwaPolicy {
    RwaPolicy {
        policy_version: "p1".to_owned(),
        modellability_policy_version: "m1".to_owned(),
        horizon_policy_version: "h1".to_owned(),
        capital_ratio_bps: 800,
    }
}

#[test]
fn t18_identical_replay_preserves_provenance_fingerprint() {
    let evidence = vec![evidence("beta", 200), evidence("alpha", 100)];
    let first = compute_rwa_from_evidence(&evidence, &policy()).unwrap();
    let replay = compute_rwa_from_evidence(&evidence, &policy()).unwrap();
    assert_eq!(first, replay);
}

#[test]
fn t19_policy_version_change_changes_fingerprint_but_not_capital_state() {
    let evidence = vec![evidence("alpha", 100)];
    let baseline = compute_rwa_from_evidence(&evidence, &policy()).unwrap();
    let changed = RwaPolicy {
        policy_version: "p2".to_owned(),
        ..policy()
    };
    let replay = compute_rwa_from_evidence(&evidence, &changed).unwrap();
    assert_ne!(baseline.provenance.fingerprint, replay.provenance.fingerprint);
    assert_eq!(baseline.capital_state, replay.capital_state);
}

#[test]
fn t20_evidence_ordering_is_deterministic() {
    let a = evidence("alpha", 100);
    let b = evidence("beta", 200);
    let forward = compute_rwa_from_evidence(&[a.clone(), b.clone()], &policy()).unwrap();
    let reverse = compute_rwa_from_evidence(&[b, a], &policy()).unwrap();
    assert_eq!(forward, reverse);
}

#[test]
fn t21_empty_evidence_is_rejected() {
    let error = compute_rwa_from_evidence(&[], &policy()).unwrap_err();
    assert_eq!(error, RwaComputationError::EmptyEvidence);
}

#[test]
fn t22_mixed_source_provenance_is_rejected() {
    let mut second = evidence("beta", 200);
    second.source_work_id = "work-15".to_owned();
    let error = compute_rwa_from_evidence(&[evidence("alpha", 100), second], &policy()).unwrap_err();
    assert_eq!(error, RwaComputationError::MixedSourceWork);
}

#[test]
fn t23_invalid_policy_basis_points_are_rejected() {
    let invalid = RwaPolicy {
        capital_ratio_bps: 100_001,
        ..policy()
    };
    let error = compute_rwa_from_evidence(&[evidence("alpha", 100)], &invalid).unwrap_err();
    assert_eq!(error, RwaComputationError::InvalidCapitalRatio);
}

#[test]
fn t24_mixed_evidence_versions_are_rejected() {
    let mut second = evidence("beta", 200);
    second.evidence_version = "e2".to_owned();
    let error = compute_rwa_from_evidence(&[evidence("alpha", 100), second], &policy()).unwrap_err();
    assert_eq!(error, RwaComputationError::MixedEvidenceVersion);
}

#[test]
fn t25_invalid_risk_weight_is_rejected() {
    let invalid = RwaEvidence {
        risk_weight_bps: 100_001,
        ..evidence("alpha", 100)
    };
    let error = compute_rwa_from_evidence(&[invalid], &policy()).unwrap_err();
    assert_eq!(error, RwaComputationError::InvalidRiskWeight);
}

#[test]
fn t26_invalid_horizon_multiplier_is_rejected() {
    let invalid = RwaEvidence {
        horizon_multiplier_bps: 100_001,
        ..evidence("alpha", 100)
    };
    let error = compute_rwa_from_evidence(&[invalid], &policy()).unwrap_err();
    assert_eq!(error, RwaComputationError::InvalidHorizonMultiplier);
}
