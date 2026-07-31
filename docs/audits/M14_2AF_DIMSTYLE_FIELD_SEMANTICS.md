# M14.2af DIMSTYLE Field Semantics

## Scope

M14.2af publishes one reviewed registry for all 68 DIMSTYLE-specific fields
documented by Autodesk. Exact named records admitted by M14.2ae now expose
source-order text, binary64, signed-16-bit, and handle evidence plus one stable
absent/unique/multiple card for every registry field.

This is a lossless typed projection. It does not choose among duplicates,
invent defaults for absent values, normalize text or handles, or infer a value
domain beyond the documented wire family.

## Normative basis

Autodesk's DIMSTYLE table-entry reference enumerates the record fields and
their group codes:
<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-F2FAD36F-0CE3-4943-9DAD-A9BCD2AE81DA.htm>.

Autodesk's symbol-table reference defines the surrounding exact `TABLE`,
group-2 table name, record, and `ENDTAB` structure:
<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-5AB9300F-F0AC-4ADE-89EA-A9D1D152D8B8.htm>.

The registry covers DIMSTYLE-specific groups `3..=7`, `40..=48`, `70..=79`,
`140..=148`, `170..=179`, `270..=289`, and `340..=344`, with only the exact
documented members of those ranges. The existing M14.2ae table directory owns
the required group-2 name, while the common record-handle layer already owns
DIMSTYLE group 105; neither is duplicated as a style-property card.

## Contract

- `dxf_dimstyle_fields()` is sorted by group code and freezes 5 text, 18
  binary64, 40 signed-16-bit, and 5 handle fields.
- Values remain tied to their exact raw group and authoritative source span.
  Source order and every duplicate occurrence are preserved.
- ASCII numbers and handles decode strictly into exact values or typed lexical
  issues. Binary floating-point payloads retain their exact IEEE-754 bits.
- Content nested inside group-102 application groups cannot impersonate a
  DIMSTYLE field. Unknown and common record groups remain available through
  the retained lower raw/table directories.
- Every admitted named DIMSTYLE record receives 68 cards. A card reports
  `Absent`, `Unique`, or `Multiple { occurrence_count }` and points back to all
  retained value occurrences without selecting one.
- Source identity, cancellation, record/value lookup, and public `Copy` or
  `Send + Sync` bounds remain explicit.

## Dialect boundary

All nine supported dialects have ASCII/Binary parity for every physically
representable fixture field. AC1009 Binary uses a one-byte group-code header,
so codes above 255 are absent from both sides of its parity fixture; this is a
wire limitation, not a claim that all such fields are semantically forbidden
in every AC1009 source. AC1012 through AC1032 additionally verify handle-field
parity.

## Size and localization review

The registry replaces per-field branches with sorted immutable data. The
behavior modules are 139, 301, and 299 lines; the focused integration test is
354 lines. All new production and test modules stay below the repository's
500-line review target. The 801-line `lib.rs` change is twelve declarative
module/re-export lines only; it adds no behavior. The prior repository-wide
size findings and next-touch split policy remain recorded in
`M6_5O_R7_CLI_I18N_AND_SIZE_AUDIT.md`.

No user-facing string was added. English/Vietnamese CLI output continues to
come from the compile-time i18n catalogs, and the 17 CLI localization/contract
tests remain green. No dependency manifest or lockfile changed.

## Coverage

The focused 4-test suite freezes all 68 registry codes and wire-family counts,
checks ASCII/Binary evidence and cardinality parity across all nine dialects,
preserves duplicate doubles, proves absent cards and application-group decoy
exclusion, retains malformed ASCII number/handle issues, and covers
cancellation, source identity, lookup misses, and public traits.

## Nonclaims

This checkpoint does not select a DIMSTYLE field occurrence, apply Autodesk or
application defaults, validate enum/range/flag combinations, resolve text
style or arrow-block handles, interpret tolerance strings, construct glyph
geometry, edit, or write DIMSTYLE records.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 715 workspace tests with zero failures or ignored tests, production
forbidden-macro scanning, and `git diff --check`. The focused suite passed 4/4
tests. No dependency manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `7d8a11713351b0d93226ab4475f43af94e3a32f78c8efea6dfa16a59f74fb2f7` |
| `crates/seacad-dxf-core/src/lib.rs` | 801 | `b4ca22e1ca240e44586c4cfff862e8fbc55873b6979715c10da1f33fb8565c4b` |
| `crates/seacad-dxf-core/src/dimstyle_field.rs` | 139 | `b69598d8186092f399b64a0a5398c0bfccd0bb6dcd88a603c0232610a3f2c689` |
| `crates/seacad-dxf-core/src/dimstyle_field_evidence.rs` | 301 | `edcb7c7726abdfadac4694ede0a3f534fc91055eb8198f691aabb9fb76aa6794` |
| `crates/seacad-dxf-core/src/dimstyle_field_card.rs` | 299 | `bf5802a7d9849fb545af03914949200f4b027b04a42d45bb9273bfb261863994` |
| `crates/seacad-dxf-core/tests/dimstyle_field_semantic_tests.rs` | 354 | `54f3f7627ac8cc96305fc4cd4f463cc53cea11952b88fffa9d112c7decaaa965` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 288 | `50839c11df2828f5bc4555e817f57f22a86d97b11816c2d06d1ded673363afde` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,648 | `876055b00c684ad771de38f3b0f730d5337d9f2d9e84e21f2e02010d22a95f00` |
| `docs/SUPPORT_MATRIX.md` | 1,342 | `41ac4c38e3c5e03b2182ae42be72391b7dd578ff75817f6c51be35f1c6f5ddb6` |
