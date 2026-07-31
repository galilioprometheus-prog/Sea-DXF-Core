# M6.5o-r7 CLI i18n Catalog and Repository Size Audit

## Verdict

SeaCad already provides explicit English and Vietnamese human output, but 35
call sites embedded parallel language strings across command construction,
execution, and rendering. M6.5o-r7 moves that reusable text into compile-time
per-locale catalogs, so another language can be added without expanding those
business modules. JSON v1, stable codes, DXF behavior, and dependencies do not
change.

After separating inline tests, integration tests, and generated source, no
handwritten production file exceeds 1,000 lines. Line count alone therefore
does not justify a broad source split.

## System and dependency evidence

The runtime path is:

`argv -> CliLanguage -> CliCatalog -> Clap/CliOptions -> DXF core -> CliReport
-> JSON or localized text`

Localization remains in `seacad-cli`; `seacad-dxf-core` exposes stable
technical values and has no dependency on presentation language. JSON keys,
status values, codes, and English technical messages remain the machine
contract. `cargo metadata --no-deps` and `cargo tree --workspace --depth 1`
show the existing three-crate workspace and no new dependency edge.

## Ranked findings

### P1 — bilingual strings were coupled to three business modules

Evidence: prior to this checkpoint, `CliLanguage::pick` appeared 35 times in
`crates/seacad-cli/src/main.rs`, `command.rs`, and `output.rs`.

Impact: every additional language would multiply edits inside command and
rendering logic, making omissions and inconsistent labels more likely.

Change: `locale/catalog.rs` defines the complete reusable catalog shape;
`locale/en.rs` and `locale/vi.rs` own language data; business modules select
one catalog. Construction of a catalog is compile-time complete, and a direct
test rejects empty entries for every supported language.

Verification: English/Vietnamese help, usage, human output, redaction, error
mapping, and JSON invariance tests must remain byte-contract compatible.

Uncertainty: Clap-generated English usage errors and the existing Vietnamese
error-kind translation remain intentionally specialized in `locale.rs`; a
third language will need its own usage-error renderer.

### P2 — one integration test file mixes many HEADER numeric domains

Evidence:
`crates/seacad-dxf-core/tests/header_numeric_tests.rs` is 2,336 handwritten
test lines. Its tests cover parity, malformed cardinality, DIMENSION values,
coordinates, ASCII grammar, binary boundaries, time values, cancellation, and
large shared fixtures.

Impact: future HEADER work has a broad review surface and fixture helpers
couple otherwise distinct test domains.

Recommended cut: when HEADER numeric behavior is next changed, split the file
into parity/cardinality, domain/boundary, and fixture-support modules while
preserving exact fixtures and assertions. Do not split merely to meet a line
threshold.

Verification: the existing test binary's cases must be preserved one-for-one,
then workspace tests and fixture hashes must remain unchanged.

Uncertainty: no active HEADER change pressure exists in the current DXF entity
milestone, so this checkpoint does not perform that unrelated move.

### P3 — release evidence has the largest cohesive production surface

Evidence:
`crates/seacad-schema-gen/src/bin/seacad-release-evidence.rs` has 957
handwritten production lines plus 135 inline test lines and owns both
CycloneDX generation and legal-bundle verification.

Impact: a future change spanning either policy has a larger review surface,
although both responsibilities share Cargo metadata, package identity, and
hashing policy.

Recommended cut: only when this binary changes again, extract `sbom` and
`legal_bundle` private modules behind the existing command boundary. Keep
canonicalization and package identity shared; do not add a framework or public
abstraction.

Verification: generated SBOM/legal files must remain byte-identical and the
release-evidence drift gate must pass.

Uncertainty: Git history shows five focused M13 changes, but no current defect
or dependency-direction violation; immediate movement has lower value than
continuing DXF entity semantics.

## Minimal refactor sequence

1. Complete the compile-time CLI catalog and prove current output unchanged.
2. Continue DXF M14.2; do not interrupt core progress for line-count-only
   changes.
3. Split the HEADER numeric integration test only when that test domain changes.
4. Split release SBOM/legal internals only at their next behavioral milestone.

## Size inventory

Handwritten production files over 1,000 lines: none.

Test/generated files over 1,000 lines:

- `crates/seacad-dxf-core/tests/header_numeric_tests.rs`: 2,336 test lines.
- `crates/seacad-dxf-core/src/generated/header_schema.rs`: 2,392 generated
  lines; its source of truth is the reviewed schema manifest.

The largest handwritten production files after separating inline tests are:

- `seacad-release-evidence.rs`: 957 lines.
- `header_numeric.rs`: 834 lines.
- `seacad-schema-gen/src/main.rs`: 712 lines.
- `seacad-release-receipts.rs`: 699 lines.

The implemented catalog adds one locale ownership boundary and removes 35
parallel-language selections from business modules. It adds no package,
runtime lookup, serialization concept, or core dependency.

## Commands and evidence gaps

Read-only evidence used `rg --files`, `rg`, `cargo metadata --no-deps`,
`cargo tree --workspace --depth 1`, line counts separated at `#[cfg(test)]`,
and focused Git history. The first pass inspected the ten largest apparent
production files and the single oversized integration test.

Internal release executables, including `seacad-corpus-receipt`, remain
English-only operational tooling and were not reclassified as end-user product
UI. No GUI or plugin UI exists in this workspace, so their future localization
surface cannot yet be audited.

Repository modified: yes, only the CLI catalog boundary, its contract test, and
documentation. Final gates and artifact hashes are recorded below after the
checkpoint passes.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 657 workspace tests with zero failures/ignored tests, prohibited
production-macro scanning, removal of all `CliLanguage::pick` call sites, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `897d516eb06260014f6a1b0baf5573c9df57840feed8cca1204da652cab57176` |
| `crates/seacad-cli/src/command.rs` | 251 | `0bca9d29026d7d0235ca0aad866a5e882f848e115d149fa4eaa372a902880909` |
| `crates/seacad-cli/src/locale.rs` | 186 | `30168021c0ddac1867100c3562630fe912ffbddda291f2639e8a2c778f6f1cff` |
| `crates/seacad-cli/src/locale/catalog.rs` | 91 | `b992a1ad44766ca64e17297f80ef68be5f91268445e33d1f55733050a660d2a1` |
| `crates/seacad-cli/src/locale/en.rs` | 43 | `7538fcdac92069efa3618930ba9d2f93ec1e82c8600b464cb0898b2d593c2b21` |
| `crates/seacad-cli/src/locale/vi.rs` | 43 | `4ad6f83333f04c7aeb0e7997ed0a31c916eed6795a63f164aafe48c6c53d08ff` |
| `crates/seacad-cli/src/main.rs` | 758 | `00bf1e0f8329c0169b2ee7d4b4668b1e36986bde32164c09f8f3ace8f3823850` |
| `crates/seacad-cli/src/output.rs` | 367 | `8783732f18dc66a22aa180bebe677cc9ea5d8d1163dde2931af2e96a96de0cfc` |
| `docs/CLI_JSON_V1.md` | 131 | `ec442fd1d1808a3e22875f50b142894866aa802f7177ee030509c9d1fe7edfc2` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,494 | `3c01aab71c024920ad4f8ba22b3d52ab16236a6799b44112340600b950b8cfe7` |
