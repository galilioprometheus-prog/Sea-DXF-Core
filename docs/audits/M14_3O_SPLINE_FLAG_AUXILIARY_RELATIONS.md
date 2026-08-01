# M14.3o SPLINE Flag/Auxiliary Relations

## Scope

M14.3o composes the exact SPLINE group-70 state with M14.3n weight and normal
semantics. It reports relations and source contradictions without editing,
normalizing, rejecting, or selecting source evidence.

## Evidence boundary

Autodesk's 2024 SPLINE table defines bit 16 as linear with the planar bit also
set, says the group-210 normal is omitted for a nonplanar spline, and defines
bit 4 and group 41 as rational and weight data respectively:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

The read-only legacy validation reports linear-without-planar, missing planar
normal, and near-zero normal observations. M14.3o uses only the first two as
relation-oracle confirmation. Its public zero signal is deliberately exact;
the legacy epsilon is neither copied nor claimed as Autodesk behavior.

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_spline_validation.rs` | `f1d1e07660ed61dae2ed0170817a4947bd55b589397e59964ccb932183258104` |

No legacy code or fixture is copied, translated, vendored, or linked.

## Contract

- Every retained SPLINE record has one stable relation entry and retains its
  exact absent, duplicate, invalid, or explicit flag semantic.
- Missing, duplicate, and invalid flags make all dependent relations
  unavailable. No occurrence is chosen.
- Explicit flags distinguish not-linear, linear-planar, and the documented
  linear-without-planar contradiction while preserving unknown flag bits in
  the owned evidence chain.
- Rational state is paired with implicit-unit, explicit-matched, or explicit
  count-mismatched weights. Invalid and behavioral nonpositive member totals
  remain visible; no undocumented rational/weight equivalence is imposed.
- A planar normal is missing, explicit with an exact-zero signal, or
  unavailable with its component issue. A nonplanar normal is either omitted
  as documented or retained as an unexpected source observation.
- Construction uses compact checked ordinals, fallible reservation, bounded
  one-entry-per-record expansion, source-identity checks, and cancellation.

## Nonclaims

M14.3o does not reject negative or unknown flag bits, use an epsilon normal
threshold, validate closed/periodic topology, derive point semantics, reconcile
degree/knot equations, construct analytic NURBS data, process HELIX, edit,
write, or advance SPLINE to `Complete`.

## Verification

The focused relation suite passes 3/3 tests and the workspace passes all 765
tests. Schema and release-evidence checks, `cargo deny --locked check`, format,
workspace Clippy with warnings denied, workspace tests, the production
forbidden-macro scan, and `git diff --check` all pass. The checkpoint adds 332
physical production lines: 327 in the relation module and five module/export
lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 97 | `e539229b40a00d2c34d03abdab2341ee65287245ac2fcd41091ef8efceac702f` |
| `crates/seacad-dxf-core/src/lib.rs` | 884 | `6bddb296f5867a3bd7e95428d825ae831afda0f307403664b2f8421a4b5d6225` |
| `crates/seacad-dxf-core/src/spline_relation.rs` | 327 | `68380216452619afb19dd5092f9761ee54aac3eef296db167034bf3f6b985cb8` |
| `crates/seacad-dxf-core/tests/spline_relation_tests.rs` | 411 | `6ad2cbd753002a289550b952ce822126432a3f84b7056dd805d68d9ecc7f8dc0` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 438 | `f0f9b5b28795c06e8126ed74650db4566e060ee042126a9613fac2ccdb6a3a57` |
| `docs/IMPLEMENTATION_PLAN.md` | 1838 | `ba98e2e23bded10f7db7f61294afaf2782238e15ebd59177766dd21835227cbb` |
| `docs/SUPPORT_MATRIX.md` | 1506 | `ae7e25e97ba00c50ac905ca4f1811f5635e292adf5a53c642a2db1a15425c12a` |
