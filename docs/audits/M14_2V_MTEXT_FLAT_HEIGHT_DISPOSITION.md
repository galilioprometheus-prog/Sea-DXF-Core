# M14.2v MTEXT Flat Height Disposition

## Scope

M14.2v closes the direct MTEXT group-50 investigation with a typed,
fail-closed height-framing disposition. It does not assign ambiguous values to
rotation, shared height, or individual column heights.

## Evidence and decision

Autodesk's MTEXT DXF reference assigns group 50 both to rotation angle and to
repeated column heights. The reference enumerates direct column groups but
does not publish a discriminator that remains valid for arbitrary preserved
source order.

The read-only legacy SeaCad workspace independently retained every group-50
value and emitted `AmbiguousMTextGroup50Role` whenever direct column fields
coexisted. It explicitly rejected inference from incidental record order. No
legacy source or fixture bytes were copied.

An isolated AutoCAD 2027 Core Console probe was attempted with synthetic
scratch-only fixtures. The executable failed to complete even for the
no-group-50 control after profile initialization, so the run was rejected as
oracle evidence. The scratch material is not part of this checkpoint.

The only evidence-backed result is therefore a conservative typed boundary:

- Embedded and XDATA envelopes have unambiguous height framing.
- Flat sources without group 50 report no height evidence.
- Flat sources with group 50 report `AmbiguousDirectGroup50` with exact
  occurrence count and first raw provenance.
- Static and dynamic-manual flat modes report a dedicated
  `UnsupportedAmbiguousDirectGroup50Height` relation issue.
- Dynamic-automatic mode remains usable because it consumes no height value.

## Coverage and size

All nine supported dialects retain ASCII/Binary parity for the direct source,
including the two-value ambiguous disposition and usable dynamic-automatic
mode. Separate static and dynamic-manual records verify the dedicated typed
failure and raw provenance. Changed behavioral production modules remain
between 369 and 407 lines; the existing 741-line `lib.rs` only receives the
public re-export.

## Explicit nonclaims

M14.2v does not disambiguate direct group 50, derive column geometry, validate
linked-column graph membership, edit MTEXT, or write column objects.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 681 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The focused
flat-column suite passed 4/4 tests. No dependency manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `4e976f491bf92b06eb3277fa67db13c00decd66aabf0804f5d834cd934b65034` |
| `crates/seacad-dxf-core/src/lib.rs` | 741 | `0028dfd8b88f607d890dd64bd5a3cd7c29c552120efe80b0338f6ccc3213f58c` |
| `crates/seacad-dxf-core/src/mtext_column_relation.rs` | 369 | `df107fd5a1cc201f534ae2ca0091362455c9ecc201a83088458bbdef77d37f41` |
| `crates/seacad-dxf-core/src/mtext_column_semantic.rs` | 407 | `1d64f83db6115056b84cc9f0ada6163eba492dfd5d974da8541a9d57f20fb001` |
| `crates/seacad-dxf-core/src/mtext_column_semantic_project.rs` | 371 | `6269f0b5939f9f18153b3f398f51a8f3209db4ca4e9b315bf0d3141efedb0b0f` |
| `crates/seacad-dxf-core/tests/mtext_flat_column_evidence_tests.rs` | 328 | `75a185e078609cbe19d2b8020f0d0296892c3e41d66f70d913890b012af46524` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 226 | `3930284261508d7a6d9dfff2d4850fa684f6835c100f0e0152c9ca89722b0a30` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,574 | `7ff6366d400e0bc3215d78a68e50535be94e6e378a24f03bec732054feca1290` |
| `docs/SUPPORT_MATRIX.md` | 1,268 | `1ade3e66678e981906d105fb22d9b87e320a47edef9b04b24b0f9c454c7150bf` |
