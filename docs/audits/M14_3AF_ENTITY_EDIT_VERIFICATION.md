# M14.3af Entity Edit Verification

## Scope

M14.3af adds a verifiable finish path for `DxfEntityEditSession`. The resulting
`DxfEntityEditPlan` owns one immutable transaction plus the semantic
postconditions of every queued common-field singleton. It verifies an
independently opened post-image and returns an executable inverse journal only
after both semantic and exact raw-byte checks pass.

## Evidence boundary

Field cardinality, defaults, wire domains, and writer order remain sourced from
the generated common-field registry and Autodesk's common entity code table:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`

M14.3af adds no new DXF field meaning. It composes the M14.3k semantic
directory, M14.3ae session, and M11.1b exact post-image/inverse contracts. The
user-authorized legacy tree remained read-only; its earlier plan contained no
semantic edit verifier to copy. No external or legacy implementation, code,
fixture, or data was copied, translated, vendored, linked, or added at runtime.

## Contract

- `finish()` continues to return a plain `DxfTransactionPlan`.
  `finish_verifiable()` additionally moves every queued edit into one
  `DxfEntityEditPlan` semantic expectation list.
- An explicit expectation owns exact raw text or retains the typed finite
  double, handle, Int16, or Int32 requested by the caller. Proxy group-310
  chunks cannot enter this singleton path. An implicit expectation represents
  an accepted optional reset. `Debug` exposes counts and identities only.
- Expectations use raw record ordinal rather than the source-bound pre-image
  key because a successfully edited post-image has a different `DxfSourceId`.
  Raw record ordinals remain stable because this checkpoint edits fields but
  does not insert or delete entity records.
- Verification first checks cancellation, the source transaction precondition,
  and the post-image projected length/physical format.
- A fresh common-field semantic directory must contain the same record ordinal
  and field. Explicit edits require `Unique` plus `Explicit`; resets require
  `AbsentOptional` plus the generated `Defaulted` or `Absent` state.
- Exact text is reread from its post-image value span and compared byte-for-
  byte. Numeric and handle values compare their typed lossless projections.
- A semantic mismatch returns a typed, payload-free issue. Only after all
  expectations pass does M11.1b compare the complete raw post-image and
  materialize the inverse. Unrelated raw mutations therefore remain fatal raw
  transaction errors rather than semantic-field issues.

## Nonclaims

M14.3af does not write a destination; connect semantic verification to M12.1b
create-new cleanup; verify already-implicit no-op receipts; validate common-
property domains or references; add family patch variants; edit sequences or
nested structures; insert entities; allocate handles/owners; clone/delete
closed sets; settle applicability; or advance an entity topic to `Complete`.

## Verification

Focused tests cover strict ASCII/Binary verification for AC1009 through AC1032,
explicit replacement/insertion, implicit reset, exact text, handle, finite
double, Int16 and Int32 comparison, semantic value mismatch, unrelated raw-byte
mismatch, source mismatch, post-image length mismatch, cancellation, source
immutability, byte-identical inverse restoration, consuming accessors, public
traits, and debug redaction. The focused suite passed 4/4 tests and the full
workspace passed 825/825 tests. Schema generation and release-evidence checks,
`cargo deny --locked check`, formatting, workspace Clippy with warnings denied,
workspace tests, and `git diff --check` all passed. The production diff is 465
added and 5 removed lines: 425 in the verification module, 34 added and 5
removed in the edit session, and 6 exports. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 165 | `a0d93b3e1ba0e1950da5db18ccada732562d4e12a938b6c85e80c52dc55aedad` |
| `crates/seacad-dxf-core/src/lib.rs` | 972 | `8450deea8755a6617dbec8f2b419c600c8d16b34d1cc46b2aa8c15a08d72a8a1` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 495 | `20d84fb7f61077449f2649f3557ae460aa8b692af5e71aa949cdaa77a10a7fe7` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 425 | `cbe9c98f3c51eaa25cde1d0ab0db4a2b82686be0cf8ebfca83963cc9204d40a0` |
| `crates/seacad-dxf-core/tests/entity_edit_verification_tests.rs` | 463 | `335756f55485fa07075347aae3830f2b22e130329b3292be35905a951f496332` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 640 | `5d6b627ee10a3a36d3240481ffd64b3ba079e5ac752d9a47ebd220d5ceb6434c` |
| `docs/IMPLEMENTATION_PLAN.md` | 2071 | `956c32283f50298f60239a977627eb7d8c509fd1f57bd02776a232a068f055f7` |
| `docs/SUPPORT_MATRIX.md` | 1721 | `8f8afdee02d6ab37e5ab7175b4057ffd1b642ff3874d22cabbc51564a414ffc6` |
