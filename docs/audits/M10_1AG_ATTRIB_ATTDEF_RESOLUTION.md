# M10.1ag ATTRIB-to-ATTDEF Resolution Audit

## Outcome

M10.1ag composes the existing INSERT block-name resolution, classic ATTRIB text
semantics, and block-local ATTDEF tag index. Each retained ATTRIB is resolved
only when its INSERT has one exact BLOCK target. Exact same-document source
spans avoid allocating a tag-sized query buffer.

## Contract

- Every retained ATTRIB receives one source-order resolution entry.
- Non-unique INSERT targets remain `TargetUnavailable` with their exact M10.1j
  resolution state.
- A unique target with an unusable ATTRIB tag remains `TagUnavailable`.
- An exact zero-match result is `Missing` only when the target BLOCK has no
  unusable ATTDEF tags; otherwise it is `Indeterminate`.
- One exact match is `Unique`; duplicates remain `Ambiguous` in
  definition-local order.
- Entries retain the unique target and compact ranges into one shared match
  array.
- Construction is cancellation-aware and source-identity checked.

## Evidence

- ASCII/Binary parity covers all nine supported AC1009-AC1032 dialects.
- Integration coverage distinguishes unique, ambiguous, missing,
  indeterminate, unusable-tag, missing-target, and ambiguous-target states.
- Bounds, cancellation, and public `Copy`/`Send`/`Sync` traits are covered.

## Non-claims

M10.1ag does not normalize or decode tags, validate tag syntax, choose among
duplicate BLOCK or ATTDEF records, require a closed attribute sequence or
BLOCK definition, validate ownership, compare flags/default values, transform
attribute placement, edit, write, or render.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (560 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/insert_attribute_definition_resolution.rs` | 329 | `0e0c5d69c5f599efd4a086f6e0b2cbc621f8eb83af8201627677325c1440f6a0` |
| `crates/seacad-dxf-core/src/lib.rs` | 606 | `db379243af086ede46951179d6ad4e0fce794c008680afb6ba8c63ed17e09a9d` |
| `crates/seacad-dxf-core/tests/insert_attribute_definition_resolution_tests.rs` | 226 | `edf27493a9085b92dabe3fbc43ae26aa074c1582ea22a6799f09e3cb070bc0bd` |
| `docs/IMPLEMENTATION_PLAN.md` | 1128 | `6a367db09fa68301e2019c77c30b459bd2ed7e29b65e2922c20c0397d93aae38` |
| `docs/SUPPORT_MATRIX.md` | 933 | `96b87d7806385154d6596b8485c08ddf965dba4e1cab65a941b5d736489980a6` |
