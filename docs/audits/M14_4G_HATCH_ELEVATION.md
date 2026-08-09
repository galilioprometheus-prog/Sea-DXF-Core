# M14.4g HATCH Elevation Tuple

## Scope

M14.4g selects one exact required elevation point from each available M14.4f
HATCH header partition. It excludes all boundary-path and seed-point collisions.

## Normative evidence

Autodesk defines the HATCH elevation point in OCS as groups 10/20/30, requires
X and Y to equal zero, and uses Z as the elevation:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm`

The boundary-path table documents the nested group 10/20 meanings excluded by
the M14.4f header fence:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

No external or legacy implementation, fixture, data, or dependency was copied,
translated, vendored, linked, or consulted.

## Contract

- Every exact HATCH subclass receives one independent elevation entry.
- An available header requires exactly one group 10, 20, and 30. Each selected
  component preserves exact `DxfDouble` bits and its raw group occurrence/span.
- X and Y must compare equal to zero; positive and negative zero are both valid
  and their original bits remain unchanged. Z may be any finite value.
- Missing, duplicate, malformed ASCII, non-finite Binary, or nonzero planar
  components retain distinct typed issues. The tuple reports their exact mask.
- A failed M14.4f partition remains a typed partition issue and exposes no
  guessed components. Boundary, seed, and trailing decoys never participate.
- Construction is cancellation-aware, fallibly allocated, source-identity
  checked, and bounded by the exact header inventory.

## Nonclaims

M14.4g does not default elevation components, transform OCS to WCS, validate
boundary counts or topology, partition pattern/seed/gradient payloads, derive
HATCH geometry, establish applicability, add CRUD/write behavior, qualify a
corpus, or claim `Complete` support.

## Verification

The focused HATCH elevation suite passes 4/4 tests and the workspace passes
1,117/1,117 tests. Dependency policy, formatting, schema, release evidence,
workspace Clippy with warnings denied, safety/link/protected-surface scans, and
whitespace checks pass. No dependency, schema, license, release, corpus, or CI
configuration changes. The checkpoint adds 450 physical production lines: 443
in the elevation module and seven module/export lines. This audit intentionally
omits its own self-referential hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 207 | `0af2c97b4905b643f5962caf0ff7f0db575bf330ffbb0bedd852e934648aed5b` |
| `README.vi.md` | 205 | `2988293e841ea2203c57d9590f68b2b96dac23100a17c5cf209df0b16b685349` |
| `crates/seacad-dxf-core/src/hatch_elevation.rs` | 443 | `2543b5459525495f3af6a16b47fe0fcf78d64abdb09084d14de4edf46caaaf7d` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,307 | `3dab8d71350e5f6268182fc49b0ac735c74ca99a3418b503d0f3ae1d5ee656ad` |
| `crates/seacad-dxf-core/tests/hatch_elevation_tests.rs` | 404 | `cc9216d83d712a4028828f8492ad804b24c7d9e156b962dfac95b5f78430c052` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `1651d8faeafaf1a5c553e59395169824ba51ab3746cbe516a73f1945fd5950c3` |
| `docs/SUPPORT_MATRIX.md` | 3,080 | `aa7ff741425e61eb1e4976e9c0508120b3e77d91f9826ed5131ac73963ff035c` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,000 | `45a9bdab019c92bafdc8139b9b2838e75ef6fb33602910c6eedd22ff2d47124a` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,422 | `893c7d54421def3185da89b2e8eaa7d6b9a7eb2cae51f0df13e907f6799c76b1` |
