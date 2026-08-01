# M14.3ad Entity Field Insertion

## Scope

M14.3ad encodes and plans one explicit insertion for an absent common-entity
singleton. It composes the reviewed field descriptor, M14.3y encoder, M14.3ac
anchor, and M11 transaction/inverse kernel. The original document stays
immutable and no destination is written.

## Evidence boundary

Common-field wire types and canonical writer order remain backed by Autodesk's
common entity group-code table:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`

M14.3ad adds no new DXF semantic interpretation. Anchor structure comes from
M14.3ac, encoding from M14.3y, and insertion/inverse behavior from M11.1a/b.
Readers remain order-independent and all groups outside the empty insertion
span remain exact.

The user-authorized legacy trees named in the task remained read-only. Earlier
fixture inspection corroborated the same common-property boundaries and line-
ending diversity. No external or legacy implementation, code, fixture, or data
was copied, translated, vendored, linked, or added at runtime.

## Contract

- Planning is source-bound to the raw document, evidence directory,
  `DxfEntityKey`, field, value, resource profile, and cancellation token.
- M14.3ac runs first. Any existing/duplicate singleton, sequence, nested
  extension dictionary, subclass/application-group failure, wrong section,
  unavailable dialect, or source mismatch returns an `Anchor` issue without
  encoding or creating a transaction.
- The M14.3y encoder then enforces the descriptor wire type, finite numeric and
  exact-text constraints, chunk bound, dialect group-code availability,
  resource limits, and cancellation. Failures return an `Encoding` issue.
- A successful insertion is one M11 patch with `[offset, offset)` source span,
  one complete encoded group, and an empty captured inverse fragment.
- Binary bytes are used exactly as encoded for the declared dialect.
- ASCII encoded groups contain exactly two separators. Each is replaced with
  the complete preceding raw group's exact LF, CR, or CRLF ending before the
  patch is frozen. Payloads cannot contain these bytes under the encoder
  contract.
- The plan exposes its source identity, entity key, field, anchor, transaction,
  and consuming transaction conversion. Debug output never includes value
  bytes.
- Strictly reparsed post-images must expose the requested common-field semantic;
  the executable inverse must restore the complete source byte-for-byte.

## Nonclaims

M14.3ad does not insert group-310 sequences; create or edit an
`ACAD_XDICTIONARY` envelope; combine multiple fields; validate color, material,
owner, handle, plot-style, or other property domains/references; allocate a
handle or owner; clone/delete graphs; run a create-new verified destination
write; settle applicability; or advance any entity to `Complete`.

## Verification

Focused tests cover all nine dialects with paired strict ASCII/Binary insertion,
semantic re-projection, executable inverse restoration, source immutability,
LF/CR/CRLF preservation, all singleton wire domains, empty insertion spans,
empty inverse capture, existing/duplicate/sequence/nested anchor failures,
wire mismatch, AC1009-unavailable group code, source mismatch,
pre-cancellation, public traits, and debug redaction. Final workspace counts and
artifact receipts are fixed below. The focused suite passes 4/4 and all 817
workspace tests pass. Schema and release-evidence freshness,
`cargo deny --locked check`, formatting, workspace Clippy with warnings denied,
the production forbidden-macro scan, and `git diff --check` all pass. The
production diff is 277 added lines: 273 in the insertion planner and four
module/export lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 156 | `5521fd8f9d97eecba516ed462d2a3545ba2881d40a4eb04ed2d6361adedca043` |
| `crates/seacad-dxf-core/src/lib.rs` | 961 | `c59d17157b4947456f27c90bbaf9495ad33310c20cb9abdc8e93d35cf1773745` |
| `crates/seacad-dxf-core/src/entity_field_insertion.rs` | 273 | `8e49251cae5bd92982dbcf37353dafab8bed86654f452bcf470a068c906754eb` |
| `crates/seacad-dxf-core/tests/entity_field_insertion_tests.rs` | 612 | `2000353f331d0ed00f14a3db8d0552795b14d12496d579536fc5ad0c97de0c14` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 614 | `a0f337feb7f5abc347590d3cae9ee37c7bf32cbf7ebc64f28a6626a072170896` |
| `docs/IMPLEMENTATION_PLAN.md` | 2044 | `fad1a584e89901eb0877b43f566b6e39ac4feb7232d14ed5c0661e9c203044d2` |
| `docs/SUPPORT_MATRIX.md` | 1689 | `f4fc9aa5afa327ae91811e8f83c56a07f8b09310dfed39768089984cc8d9786a` |
