# M14.3m SPLINE Weights and Vectors

## Scope

M14.3m closes the remaining public SPLINE defining-value evidence for group 41
and forms duplicate-safe structure for weights, start/end tangents, and the
normal vector. It does not yet validate numeric domains or create geometry.

## Normative evidence

Autodesk's 2024 SPLINE table documents group 41 as weight data omitted when
weights are 1, groups 12/22/32 and 13/23/33 as optional WCS start/end tangent
components, and groups 210/220/230 as the planar normal with optional Y/Z:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

The same table places these fields in `AcDbSpline`. M14.3m accepts legacy
pre-subclass data for compatibility and exact `AcDbSpline` data, while later
subclass codes remain raw but do not enter SPLINE evidence.

## Read-only behavioral evidence

The legacy source was inspected only to confirm observed array-by-ordinal
weight handling, duplicate scalar/vector reporting, and subclass separation.
No implementation or fixture was copied, translated, vendored, or linked.

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_spline_decode.rs` | `a8dd9117f21d44497eb9b620c6aa1ca539f7c3658de7bb3b94e599c63635c4fc` |
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_spline_validation.rs` | `f1d1e07660ed61dae2ed0170817a4947bd55b589397e59964ccb932183258104` |

## Contract

- `Weight` is the twenty-fifth fixed SPLINE evidence/card role and retains
  every group-41 numeric result and exact raw span.
- One weight entry per record is `ImplicitUnit` when group 41 is absent, with
  the observed group-10 control-point count. An explicit sequence retains its
  exact member slice and matched/mismatched count relation.
- Start tangent, end tangent, and normal each have one stable entry. Every
  component is `Absent`, `Unique(member)`, or `Multiple(count)`.
- Aggregate vector state is `Absent`, `Present(component mask)`, or
  `Ambiguous(duplicate-component mask)`. No duplicate is selected.
- Unique and sequence members resolve through the retained card directory;
  values and numeric failures are not copied.
- Construction uses fallible reservations, checked compact ordinals, bounded
  fixed vector expansion, and cancellation checks before/during/after scans.

## Nonclaims

M14.3m does not reject nonpositive or nonfinite weights; decide rational-flag
consistency; default missing vector components; require a planar normal; reject
zero vectors; reconcile tuple completeness; validate degree/count/knot
invariants; construct analytic NURBS data; process HELIX; migrate the older
SPLINE API; edit; write; or advance any entity to `Complete`.

## Verification

- `cargo test -p seacad-dxf-core --test spline_auxiliary_tests`: 3 passed.
- `cargo test -p seacad-dxf-core --test spline_evidence_tests`: 4 passed.
- `cargo test --workspace`: 759 passed.
- Generated schema `--check`: passed.
- Release evidence `--check`: passed.
- `cargo deny --locked check`: advisories, bans, licenses, and sources passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.
- Production additions are 515 physical lines: the new projection is 496 and
  the evidence/card/module integration adds 19, remaining near the checkpoint
  target without a production dependency.

## Artifact receipts

| Artifact | Physical lines | SHA-256 |
|---|---:|---|
| `README.md` | 97 | `5f9908b941fafe4cc748fb331e3fa4d9f886cf23a283800006ebd31b3d657e94` |
| `crates/seacad-dxf-core/src/lib.rs` | 873 | `02016c3f5651e71ce5e4f5215d9adbe7ca4c2c9c4d3c71b85041445db7572013` |
| `crates/seacad-dxf-core/src/spline_auxiliary.rs` | 496 | `24e907dbadf86e9bf7041f143ac966541d7c8a60671fa6baad0e27a6eabae188` |
| `crates/seacad-dxf-core/src/spline_card.rs` | 435 | `7bec70a31c7f07b2861b28898f1c314fa36d9ac6bb5bf2cc56284cd460317258` |
| `crates/seacad-dxf-core/src/spline_evidence.rs` | 399 | `c020d09b859d812e99d785a8f70abef27ff5a249a0beecf0f92846d4ae9b12fe` |
| `crates/seacad-dxf-core/tests/spline_auxiliary_tests.rs` | 464 | `65fb12b348750e5e536354e2b2a9f9de171a6d446a4f29217376a359fa356e1b` |
| `crates/seacad-dxf-core/tests/spline_evidence_tests.rs` | 430 | `79972437d55cc7c2ecadb4985a35e1b5f096d4b221e3666c8101ca27542f5ab6` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 418 | `060f3e834ed64834c4737e4cde3ffde73459bf84e2693a3d65f09de9f4d73b9e` |
| `docs/IMPLEMENTATION_PLAN.md` | 1813 | `937410124dd582575ec6b9a9a933d7fcc44f621d5defd887923657fc2c159417` |
| `docs/SUPPORT_MATRIX.md` | 1486 | `5d516179b44df64dbc74f7d116a65a0c069377449729785830a040f629f9d44b` |

This audit intentionally omits its own hash.
