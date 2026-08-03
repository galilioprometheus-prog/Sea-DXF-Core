# M14.3cn Entity XDATA Structure

Retrieved: 2026-08-03

## Normative basis

Autodesk's *About Extended Data (DXF)* limits group-1001 application names to
31 bytes. It defines group 1002 control strings as exact left or right braces,
allows nested lists, and requires braces to balance:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

Autodesk's `regapp` reference additionally requires registered application
names to be valid symbol-table names. This checkpoint records that broader
character-policy validation as a nonclaim rather than guessing `$EXTNAMES`
behavior:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-AutoLISP-Reference/files/GUID-D331A88A-9B6E-49C9-B745-D0A063F38FD2.htm>

## Contract

- `DxfEntityXDataStructureDirectory` owns the generic XDATA directory and
  publishes one source-order entry for every application.
- A group-1001 payload longer than 31 exact source bytes emits
  `ApplicationNameTooLong` with its observed byte count.
- Group 1002 accepts only exact one-byte `{` and `}` values. Other bytes emit
  `InvalidControlString` with the exact group.
- Nested opens are counted per application. A close at depth zero emits
  `UnexpectedListEnd`; nonzero final depth emits `UnclosedLists`.
- A normal entity group ending an open XDATA application emits `Interrupted`.
  This remains distinct from brace imbalance, so both issues may coexist.
- Issues remain in deterministic validation order. Construction and span
  comparisons are cancellation-aware, allocation is fallible, foreign-source
  lookup fails typed, and public ordinals/ranges are bounded.

## Verification boundary

The focused three-test suite covers all nine supported versions in paired
ASCII and Binary. It proves balanced nested lists, the exact 31/32-byte
boundary, invalid control bytes, premature close, remaining open depth,
interruption plus unclosed depth, and a new application after interruption.
Separate assertions cover source identity, foreign-source rejection,
cancellation, public lookup bounds, metadata size, and non-disclosing debug.

## Nonclaims

This checkpoint does not implement the complete version-sensitive symbol-table
name character policy, typed XDATA value domains, 1000/1004 payload limits,
the separate per-entity 16-KiB limit, application payload meaning, group-1005
handle resolution/remap, XDATA clone/write, or POINT/entity `Complete` support.

## Verification

The focused structure suite passed 3/3 tests. The first cold quiet workspace
run reached its 180-second wrapper after only passing output; its cache-hot
retry passed all 977 listed tests in 45.9 seconds. Generated schema and
release-evidence checks, `cargo deny --locked check`, formatting, workspace
Clippy with warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. Production changes are 354 insertions; the
focused test target adds 250 lines. No manifest, lockfile, dependency,
generated schema, locale source, committed fixture, or external corpus
changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 417 | `265823820b4f9526e12e48a707b9bae9ae8ea295748d41df386f02a4415be29e` |
| `crates/seacad-dxf-core/src/entity_xdata_structure.rs` | 348 | `374f05d5e82f1bc26bd4cd6ba32d41965b965b9bbe937476244b2fbb0c3816c4` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,084 | `735d7e7da341d44aca0bd7b6095344c77ff113389cafb8ef8d85aabaf73adb18` |
| `crates/seacad-dxf-core/tests/entity_xdata_structure_tests.rs` | 250 | `056d3249b54a88c41374b1637bfff9dec8c9f788c1096297ed02bad43272a53d` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,361 | `5bbc3d0605a8fa51762cb60cefbe6fcf323198218b3cda0bc35456b681d92b20` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,783 | `d235f200a0b8c112e5500e63de4792401ba372cc5f8bfa75eb6588390aa13f29` |
| `docs/SUPPORT_MATRIX.md` | 2,428 | `104559be35e342477e47972ff08e8cf825f0dc0659926b1d0fe66951ff7b2575` |
