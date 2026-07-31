# M14.2u MTEXT Flat Column Unification

## Scope

M14.2u projects the six unambiguous direct MTEXT column fields through the
same scalar and mode implementation used by Embedded and R2007 XDATA storage.

## Design

- `DxfMTextColumnSourceEntry::Flat` retains the physical source identity.
- The unified directory owns flat, Embedded, and XDATA evidence and orders
  their entries by source marker occurrence.
- The generic value adapter maps type, count, flow, auto-height, width, and
  gutter to existing roles.
- `RotationOrColumnHeight` deliberately maps to no scalar role.
- Flat dynamic-auto columns can satisfy the shared mode relationship without
  inventing a height; static/manual modes remain fail-closed when height
  evidence is unavailable.

## Coverage and size

All nine dialects have ASCII/Binary parity for evidence, scalar projection,
and usable dynamic-auto mode. Tests prove coexisting group-50 values create no
shared or individual height. The unified directory, generic projector, and
integration test remain independently below 500 lines.

## Explicit nonclaims

M14.2u does not disambiguate group 50, make flat static/manual height modes
usable, derive layout geometry, or edit/write MTEXT columns.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 680 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The three
flat-column tests additionally cover scalar/mode parity. No dependency
manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `8f9808bb1337ac6d45f79566b0eaca2539b01bc03bf68890e81f9098192ca71c` |
| `crates/seacad-dxf-core/src/mtext_column_semantic.rs` | 359 | `91fec323275dd01213cb0bdb3e5f18f13ef830967a9d7474aad3f39fef52d417` |
| `crates/seacad-dxf-core/src/mtext_column_semantic_project.rs` | 369 | `316847ac2cbfbc67380863ffdc9a0b1f3b2be9711927451dd3f7a469961a8146` |
| `crates/seacad-dxf-core/tests/mtext_flat_column_evidence_tests.rs` | 284 | `72efba13f46406f9f108dab0a45abb9a89c0462c8d184505830245cdd9500ccb` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 219 | `188fbc285455ed612f10d684a230ffca943c3c467865da87c48a597c0ec1c83e` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,565 | `063096a84fe009f03db584ae504b4a33619885cfcaa33f1a534ed82c6e2dcd45` |
| `docs/SUPPORT_MATRIX.md` | 1,260 | `c50122b1117bee88143462a7276286e5f6ad08dff92557056199243ade0f2378` |
