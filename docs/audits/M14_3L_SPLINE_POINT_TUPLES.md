# M14.3l SPLINE Point Tuples

## Scope

M14.3l groups the already retained SPLINE control-point and fit-point
component sequences. It adds structural evidence only; it does not construct
semantic WCS points or curve geometry.

## Normative evidence

Autodesk's 2024 SPLINE group-code table identifies group 10 with 20/30 as one
WCS control-point sequence and group 11 with 21/31 as one WCS fit-point
sequence, with one entry per point:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

Field order remains non-semantic. M14.3l therefore pairs the nth occurrence
within each component role rather than requiring 10/20/30 or 11/21/31 groups
to be physically adjacent.

## Read-only behavioral evidence

The legacy fixture below places all control-point X groups before all Y groups
and expects the coordinate arrays to associate by ordinal. It was inspected as
a behavioral oracle only; no parser implementation, fixture, or source text
was copied into SeaCad.

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\tests\semantic_spline_tests.rs` | `52dceb7f9f9e16581db68683bdfbf8bf1ff7052e432556c589932d00934df0ec` |
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_spline_validation.rs` | `f1d1e07660ed61dae2ed0170817a4947bd55b589397e59964ccb932183258104` |

## Contract

- Each retained SPLINE record receives fixed `ControlPoint` and `FitPoint`
  entries, including empty sequences.
- Component counts are exact and the tuple count is their maximum.
- Tuple `n` holds optional references to component-role member `n`; it does
  not copy or choose a decoded value.
- `Complete` means X, Y, and Z evidence all exist. Every other nonempty shape
  is a typed `Partial` state with an explicit component-presence mask.
- Numeric failures remain on the underlying `DxfSplineValue` reached through
  the card member. Missing components remain absent.
- Construction is linear in retained component evidence, uses fallible
  reservations and checked compact ordinals, and checks cancellation before,
  during, and after sequence construction.

## Nonclaims

M14.3l does not decide whether X, Y, or Z is required; apply a zero default;
validate declared counts; associate group-41 weights; project tangents or the
normal vector; validate degree, knot, rational, planar, or periodic relations;
construct analytic NURBS data; process HELIX; migrate the older SPLINE API;
edit; write; or advance any entity to `Complete`.

## Verification

- `cargo test -p seacad-dxf-core --test spline_point_tuple_tests`: 3 passed.
- `cargo test --workspace`: 756 passed.
- Generated schema `--check`: passed.
- Release evidence `--check`: passed.
- `cargo deny --locked check`: advisories, bans, licenses, and sources passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.
- New production module: 486 physical lines, within the checkpoint target;
  `lib.rs` contains only registration and public exports for this checkpoint.

## Artifact receipts

| Artifact | Physical lines | SHA-256 |
|---|---:|---|
| `README.md` | 97 | `2628437dcc43d1cf9494e96cfd3dbc8248c3eb94ae73c93a511185d98c5630e4` |
| `crates/seacad-dxf-core/src/lib.rs` | 867 | `842a4a07e9bf1e1c6bd297560cb45b3864f1643798cec01d39aa7ff6a29f0a6c` |
| `crates/seacad-dxf-core/src/spline_point_tuple.rs` | 486 | `58b01c051d343f76bc6b6fcc3b7a69d65983a60749578f3e8a83c46a8515dbdd` |
| `crates/seacad-dxf-core/tests/spline_point_tuple_tests.rs` | 403 | `e6efd91693a6ef012223a2c88a26ee7fc7a538d2df2c79088e0e3b537132ccdc` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 407 | `a7970088f4468c54b4c2ce5a237be89dfbb656954434d12101301b60727de108` |
| `docs/IMPLEMENTATION_PLAN.md` | 1800 | `f8814f5abcee108a8087a8a7ccc581930a4cf4183caca4154df5403300914663` |
| `docs/SUPPORT_MATRIX.md` | 1475 | `5d45815c1e11b0864c094a93a2ddd91afaf8eed1fa82dd387e427947eacfb6f4` |

This audit intentionally omits its own hash.
