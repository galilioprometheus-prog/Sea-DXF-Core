//! Evidence-driven risk-weighted-asset computation with replay-bound provenance.
//!
//! This module intentionally uses integer basis-point arithmetic so identical
//! evidence, policy versions, and ordering produce identical results without
//! floating-point replay drift.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RwaModellability {
    Modellable,
    NonModellable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RwaEvidence {
    pub factor_id: String,
    pub exposure_units: u64,
    pub risk_weight_bps: u32,
    pub horizon_multiplier_bps: u32,
    pub modellability: RwaModellability,
    pub source_work_id: String,
    pub source_work_version: String,
    pub evidence_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RwaPolicy {
    pub policy_version: String,
    pub modellability_policy_version: String,
    pub horizon_policy_version: String,
    pub capital_ratio_bps: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RwaCapitalState {
    pub exposure_units: u64,
    pub risk_weighted_assets: u64,
    pub capital_requirement_units: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RwaProvenance {
    pub source_work_id: String,
    pub source_work_version: String,
    pub evidence_version: String,
    pub modellability_policy_version: String,
    pub horizon_policy_version: String,
    pub factor_count: usize,
    pub evidence_count: usize,
    pub policy_count: usize,
    pub capital_state: RwaCapitalState,
    pub fingerprint: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RwaComputation {
    pub capital_state: RwaCapitalState,
    pub provenance: RwaProvenance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RwaComputationError {
    EmptyEvidence,
    MixedSourceWork,
    MixedSourceVersion,
    MixedEvidenceVersion,
    InvalidRiskWeight,
    InvalidHorizonMultiplier,
    InvalidCapitalRatio,
    ArithmeticOverflow,
}

#[must_use]
pub fn compute_rwa_from_evidence(
    evidence: &[RwaEvidence],
    policy: &RwaPolicy,
) -> Result<RwaComputation, RwaComputationError> {
    if evidence.is_empty() {
        return Err(RwaComputationError::EmptyEvidence);
    }
    if policy.capital_ratio_bps > 100_000 {
        return Err(RwaComputationError::InvalidCapitalRatio);
    }

    let first = &evidence[0];
    if first.risk_weight_bps > 100_000 {
        return Err(RwaComputationError::InvalidRiskWeight);
    }
    if first.horizon_multiplier_bps > 100_000 {
        return Err(RwaComputationError::InvalidHorizonMultiplier);
    }

    let mut canonical = evidence.to_vec();
    canonical.sort_by(|a, b| {
        a.factor_id
            .cmp(&b.factor_id)
            .then_with(|| a.exposure_units.cmp(&b.exposure_units))
            .then_with(|| a.risk_weight_bps.cmp(&b.risk_weight_bps))
            .then_with(|| a.horizon_multiplier_bps.cmp(&b.horizon_multiplier_bps))
    });

    for item in &canonical {
        if item.source_work_id != first.source_work_id {
            return Err(RwaComputationError::MixedSourceWork);
        }
        if item.source_work_version != first.source_work_version {
            return Err(RwaComputationError::MixedSourceVersion);
        }
        if item.evidence_version != first.evidence_version {
            return Err(RwaComputationError::MixedEvidenceVersion);
        }
        if item.risk_weight_bps > 100_000 {
            return Err(RwaComputationError::InvalidRiskWeight);
        }
        if item.horizon_multiplier_bps > 100_000 {
            return Err(RwaComputationError::InvalidHorizonMultiplier);
        }
    }

    let mut exposure_units = 0_u64;
    let mut risk_weighted_assets = 0_u64;
    let mut factor_count = 0_usize;

    for item in &canonical {
        exposure_units = exposure_units
            .checked_add(item.exposure_units)
            .ok_or(RwaComputationError::ArithmeticOverflow)?;

        let weighted = (item.exposure_units as u128)
            .checked_mul(item.risk_weight_bps as u128)
            .ok_or(RwaComputationError::ArithmeticOverflow)?
            .checked_mul(item.horizon_multiplier_bps as u128)
            .ok_or(RwaComputationError::ArithmeticOverflow)?
            / 10_000_u128
            / 10_000_u128;
        let weighted = u64::try_from(weighted)
            .map_err(|_| RwaComputationError::ArithmeticOverflow)?;
        risk_weighted_assets = risk_weighted_assets
            .checked_add(weighted)
            .ok_or(RwaComputationError::ArithmeticOverflow)?;
        factor_count += 1;
    }

    let capital_requirement = (risk_weighted_assets as u128)
        .checked_mul(policy.capital_ratio_bps as u128)
        .ok_or(RwaComputationError::ArithmeticOverflow)?
        / 10_000_u128;
    let capital_requirement_units = u64::try_from(capital_requirement)
        .map_err(|_| RwaComputationError::ArithmeticOverflow)?;

    let capital_state = RwaCapitalState {
        exposure_units,
        risk_weighted_assets,
        capital_requirement_units,
    };

    let fingerprint = fingerprint(&canonical, policy, capital_state);
    let provenance = RwaProvenance {
        source_work_id: first.source_work_id.clone(),
        source_work_version: first.source_work_version.clone(),
        evidence_version: first.evidence_version.clone(),
        modellability_policy_version: policy.modellability_policy_version.clone(),
        horizon_policy_version: policy.horizon_policy_version.clone(),
        factor_count,
        evidence_count: canonical.len(),
        policy_count: 1,
        capital_state,
        fingerprint,
    };

    Ok(RwaComputation {
        capital_state,
        provenance,
    })
}

fn fingerprint(
    evidence: &[RwaEvidence],
    policy: &RwaPolicy,
    capital_state: RwaCapitalState,
) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    let mut feed = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3_u64);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3_u64);
    };

    feed(policy.policy_version.as_bytes());
    feed(policy.modellability_policy_version.as_bytes());
    feed(policy.horizon_policy_version.as_bytes());
    feed(&policy.capital_ratio_bps.to_le_bytes());

    for item in evidence {
        feed(item.factor_id.as_bytes());
        feed(&item.exposure_units.to_le_bytes());
        feed(&item.risk_weight_bps.to_le_bytes());
        feed(&item.horizon_multiplier_bps.to_le_bytes());
        feed(&[match item.modellability {
            RwaModellability::Modellable => 1,
            RwaModellability::NonModellable => 0,
        }]);
        feed(item.source_work_id.as_bytes());
        feed(item.source_work_version.as_bytes());
        feed(item.evidence_version.as_bytes());
    }

    feed(&capital_state.exposure_units.to_le_bytes());
    feed(&capital_state.risk_weighted_assets.to_le_bytes());
    feed(&capital_state.capital_requirement_units.to_le_bytes());
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn t18_identical_replay_is_byte_equivalent_in_result_and_provenance() {
        let evidence = vec![evidence("beta", 200), evidence("alpha", 100)];
        let first = compute_rwa_from_evidence(&evidence, &policy()).unwrap();
        let replay = compute_rwa_from_evidence(&evidence, &policy()).unwrap();
        assert_eq!(first, replay);
        assert_eq!(first.provenance.fingerprint, replay.provenance.fingerprint);
    }

    #[test]
    fn t19_policy_change_invalidates_replay_fingerprint() {
        let evidence = vec![evidence("alpha", 100)];
        let baseline = compute_rwa_from_evidence(&evidence, &policy()).unwrap();
        let changed = RwaPolicy {
            policy_version: "p2".to_owned(),
            ..policy()
        };
        let replay = compute_rwa_from_evidence(&evidence, &changed).unwrap();
        assert_ne!(baseline.provenance.fingerprint, replay.provenance.fingerprint);
        assert_ne!(baseline.capital_state, replay.capital_state);
    }

    #[test]
    fn t20_evidence_ordering_is_deterministic() {
        let a = evidence("alpha", 100);
        let b = evidence("beta", 200);
        let forward = compute_rwa_from_evidence(&[a.clone(), b.clone()], &policy()).unwrap();
        let reverse = compute_rwa_from_evidence(&[b, a], &policy()).unwrap();
        assert_eq!(forward, reverse);
    }
}
