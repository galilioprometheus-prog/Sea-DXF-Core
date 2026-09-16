# M14.3 — RWA provenance and replay integration

## Scope

This M14 increment adds an evidence-driven RWA computation module with explicit provenance and deterministic replay behavior.

## Public computation contract (module scope)

`compute_rwa_from_evidence` consumes a bounded evidence slice and an explicit policy. The result carries capital state plus `RwaProvenance`, including:

- source work identity and version;
- evidence version;
- modellability and horizon policy versions;
- factor, evidence, and policy counts;
- computed capital state;
- deterministic 64-bit fingerprint.

The calculation uses integer basis-point arithmetic and rejects invalid basis-point inputs before calculation.

## Replay and rejection coverage

The dedicated RWA test file covers:

- **T18** — identical replay preserves the complete computation result and fingerprint;
- **T19** — policy-version changes alter provenance fingerprint while preserving the capital state for the same numeric policy inputs;
- **T20** — evidence ordering does not change the result;
- **T21** — empty evidence is rejected;
- **T22** — mixed source-work provenance is rejected;
- **T23** — invalid capital ratio is rejected;
- **T24** — mixed evidence versions are rejected;
- **T25** — invalid risk weight is rejected;
- **T26** — invalid horizon multiplier is rejected.

## Integration status

The implementation currently remains isolated from the crate root public export surface. This is intentional: the repository's large `lib.rs` was restored byte-for-byte after an unsafe whole-file replacement attempt, and no speculative rewrite is being used to obtain root-level wiring.

The next integration step is therefore narrowly defined:

1. add the `rwa` module declaration to the crate root;
2. add explicit public re-exports for the RWA contract;
3. convert the replay test from a path-included module to a true crate-level integration test;
4. verify the resulting diff and CI before treating M14 as complete.

Until those steps are verified, this document treats the RWA implementation as **module-complete but crate-root integration-pending**.
