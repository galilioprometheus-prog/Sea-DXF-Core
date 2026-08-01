# M14.3w HELIX Embedded-SPLINE Readiness

## Scope

M14.3w composes the existing HELIX relation projection with the analytic
representation of the `AcDbSpline` data embedded in the same raw HELIX
record. It publishes fail-closed analytic readiness, not an evaluator or a
tessellated curve.

## Evidence boundary

Autodesk's HELIX DXF table places spline data before the `AcDbHelix` fields:

`https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-76DB3ABF-3C8C-47D1-8AFB-72942D9AE1FF.htm`

Autodesk's SPLINE table defines the embedded knot, control/fit point, weight,
tangent, normal, flag, degree, and count fields consumed by the existing
analytic-readiness projection:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

The ObjectARX class hierarchy documents `AcDbHelix` as deriving from
`AcDbSpline`, while its note says NURB functions on a HELIX have unknown
behavior and are not recommended:

`https://help.autodesk.com/view/OARX/2026/ENU/?guid=OARX-RefGuide-AcDb_Classes`

The managed HELIX reference likewise describes an embedded spline and warns
against inherited NURBS operations:

`https://help.autodesk.com/view/OARX/2025/ENU/?guid=OARX-ManagedRefGuide-Autodesk_AutoCAD_DatabaseServices_Helix`

Autodesk also describes a helix as a spline approximation whose exact length
may differ from calculated values. This reinforces the boundary between
stored analytic representation and evaluation:

`https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-MAC-Core/files/GUID-D887B1ED-2084-48B3-92AA-56E4A93CC766.htm`

One legacy test file was inspected read-only as a behavioral oracle. It
confirmed that legacy semantics treated HELIX as spline-derived; no source or
fixture was copied, translated, vendored, linked, or used at runtime.

| Read-only oracle | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\tests\semantic_spline_tests.rs` | `52dceb7f9f9e16581db68683bdfbf8bf1ff7052e432556c589932d00934df0ec` |

## Contract

- `DxfSplineDirectory` indexes exact `SPLINE` and `HELIX` markers in
  `BLOCKS` and `ENTITIES`, with an explicit record kind.
- Ordinary SPLINE records retain their previous field-scanning behavior.
  HELIX records admit spline values only after exact `AcDbSpline` and stop at
  the next subclass transition. Colliding `AcDbHelix` values remain outside
  the embedded curve.
- Each HELIX analytic entry joins the relation and embedded-spline entries by
  exact raw-record ordinal and verifies source identity and record kind.
- The subclass path must contain exactly one `AcDbSpline`, followed by exactly
  one `AcDbHelix`. Missing, duplicate, or reversed markers remain typed.
- Readiness requires an available embedded spline, an available nonzero axis
  with exact perpendicularity, nonnegative stored radius, positive turns, and
  finite compared height. Turns above 500 remain analytic-readable existing
  data while retaining their observational domain state; creation/update
  policy is deferred. Exact zero height remains valid for a flat helix.
- All failed requirements accumulate in one compact issue mask. No partial
  analytic value is published.
- Construction is cancellable, allocation-fallible, source-identity checked,
  and dependency-free. A public test guards the composed entry at 512 bytes.

## Nonclaims

M14.3w does not call inherited NURBS operations on HELIX, evaluate the spline,
calculate curve length, sample or tessellate geometry, infer a tolerance,
create or update HELIX records, write output, settle version applicability, or
advance HELIX to `Complete`.

## Verification

The focused HELIX analytic suite passes 3/3 tests and the workspace passes all
793 tests. Schema and release-evidence freshness checks,
`cargo deny --locked check`, format, workspace Clippy with warnings denied,
workspace tests, the production forbidden-macro scan, and `git diff --check`
all pass. The checkpoint adds 394 net physical production lines: 361 in the
HELIX analytic module, six module/export lines, and 27 lines extending shared
SPLINE evidence. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 125 | `dfaddf1364b2e0c41623446d20178cfbf099451e34d83933d34b164c496d7158` |
| `crates/seacad-dxf-core/src/lib.rs` | 933 | `433926a29af260ccb47cfa738c5ccf323bff1b60a643dbd321664f5dd3071233` |
| `crates/seacad-dxf-core/src/spline_evidence.rs` | 426 | `7e2e69ddb0f105edac82d9bcaae0da7d327a7cd632ce9c4c26d920a6756d977c` |
| `crates/seacad-dxf-core/src/helix_analytic.rs` | 361 | `108e74c3b13f58bf6d95a397f59daac8cf515c29819b7fd9f7299169362c5042` |
| `crates/seacad-dxf-core/tests/spline_evidence_tests.rs` | 432 | `ea738c777a6415191620904fc01d4fcd6a2e0650b599da7ba8e343506416b3ac` |
| `crates/seacad-dxf-core/tests/helix_analytic_tests.rs` | 421 | `0b929c3dc30d4c2b84ad9f106d3550295ec5f3ca1d464b7b42e0fca59aa5d829` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 535 | `2493c5be7184efe60d2ac464e34cb0a11143bd2f109de7f9571204d96a4d4e17` |
| `docs/IMPLEMENTATION_PLAN.md` | 1952 | `8c2bc7d745acc4a5ad97767452193c5fd50379ee6627dd687e364b4dc8eb1ad1` |
| `docs/SUPPORT_MATRIX.md` | 1598 | `48b69a3909d3dd9ccecb6bb55a9909cd2273268861f2624ae4e3958f1b991298` |
