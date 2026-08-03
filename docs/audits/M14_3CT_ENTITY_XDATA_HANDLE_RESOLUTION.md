# M14.3ct Entity XDATA Handle Resolution

Retrieved: 2026-08-04

## Normative basis

Autodesk assigns group 1005 to database handles of entities in the drawing
database and lists it among the generic XDATA value families:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

The core already classifies group 1005 as a soft-pointer handle, retains exact
record identities, and resolves reference values document-locally. This
checkpoint composes those reviewed primitives only after M14.3cl/co establishes
that the same group is genuinely retained generic entity XDATA.

## Contract

`DxfEntityXDataHandleResolutionDirectory` emits one compact source-bound entry
for every retained typed group-1005 occurrence, including application values
and orphans. Each entry retains its typed ordinal, generic resolution state,
and an optional exact `DxfHandleIdentityMatch`. Invalid, null, missing,
ambiguous, and unique states are preserved; only a unique result carries a
target record and identity candidate. The typed and generic parse states are
cross-checked rather than trusted independently.

The directory deliberately excludes group-1005-shaped payload inside group-102
application controls because M14.3cl excludes that scope from generic XDATA.
No missing or ambiguous target is guessed, and no cross-document identity is
accepted by lookup.

## Verification boundary

The focused suite pairs ASCII and Binary fixtures for AC1009, AC1012, AC1014,
AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032. It covers invalid, null,
missing, unique, and case-insensitive ambiguous handles; application values;
orphans in two entities; group-102 exclusion; exact target identity/record;
generic-resolution linkage; source identity; cancellation; lookup bounds;
metadata bounds; and non-disclosing debug output.

This checkpoint does not remap handles across documents, interpret an
application-specific payload, write group-1005 values, clone XDATA, or advance
any entity to `Complete`.

## Gate receipts

The focused handle-resolution suite passed 3/3 tests and the full workspace
passed all 995 listed tests across 188 targets. Generated schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. No manifest, lockfile, dependency, generated
schema, locale source, committed fixture, or external corpus changed.

Production adds one bounded 269-line module plus four root registration/export
lines; the focused target adds 276 lines. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 465 | `662e83a20b42485bc03667b85ef21837f0c6f7eabc43c36a7dcf31b1eeccfb93` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_resolution.rs` | 269 | `d071c43ea13a5634ac11b17ae31e431681bcb76450b18e40b84d666dbc7a5d88` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,122 | `cf529e44e0d9de39e52759e2ba68920428289ee574d6ad54c3e024556ce7079c` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_resolution_tests.rs` | 276 | `dab97a3b991ef12b2105482ff74317bcee6cfbdc8ad8031b49c1db8c4826ecc6` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,446 | `e33b708d0f402b913dce08fc1b10a082a066bc2da8ade8664a250dd95ccf29ae` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,863 | `57d44b5f81d72fee5c00a35613010efa02d7f0898e223743ada2fdf3eaba1d57` |
| `docs/SUPPORT_MATRIX.md` | 2,502 | `c047af5e42bfa2208dba6ba802ad2164443ad6d7781cf8810e1ced527fd28420` |
