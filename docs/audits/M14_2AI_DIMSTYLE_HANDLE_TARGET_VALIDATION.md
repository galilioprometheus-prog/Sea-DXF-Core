# M14.2ai DIMSTYLE Handle Target Validation

## Scope

M14.2ai validates each uniquely resolved M14.2ah DIMSTYLE handle target against
exact membership in the expected named symbol table. Text-style group 340
expects `STYLE`; arrow-block groups 341 through 344 expect `BLOCK_RECORD`.

The checkpoint extracts the existing DIMSTYLE table-envelope logic into one
shared immutable directory for `DIMSTYLE`, `STYLE`, and `BLOCK_RECORD`. This
removes duplicated scanning logic while retaining the earlier DIMSTYLE API.

## Contract

- Only exact uppercase matching table and record markers are recognized.
- A table contributes entries only after a matching `ENDTAB`; interrupted and
  unclosed envelopes contribute nothing.
- Exactly one group 2 outside group-102 application content names a record.
- A generically unique handle target is classified as the expected named-table
  kind, another reviewed named-table kind, or another raw record.
- Absent, multiple-value, invalid, null, missing, and ambiguous lower-layer
  states pass through unchanged and expose no selected target.
- Source identity, cancellation, raw/role/ordinal lookup, duplicate identity
  preservation, and ASCII/Binary behavior remain explicit.

## Coverage and size

The focused 7-test run covers all nine dialects in ASCII and Binary, correct
STYLE/BLOCK_RECORD targets, wrong named-symbol kinds, other raw records,
missing/null/invalid/ambiguous targets, duplicate fields, application-group
decoys, wrong-case table names, mismatched record markers, duplicate names,
interrupted/unclosed tables, cancellation, lookup misses, and public traits.

Production modules are 267, 137, and 283 lines; the integration test is 447
lines. Every changed production/test module remains below 500 lines. No
user-facing string, i18n catalog, dependency manifest, or lockfile changes.

## Nonclaims

This checkpoint does not resolve a target name, validate STYLE or BLOCK_RECORD
record contents, interpret tolerance strings, apply DIMSTYLE defaults, build
glyph geometry, edit, or write a DIMSTYLE record.

## Verification

All required local gates passed on 2026-07-31: dependency policy,
generated-schema drift, release-evidence drift, formatting, workspace Clippy
with warnings denied, 718 workspace tests with zero failures or ignored tests,
production forbidden-macro scanning, and `git diff --check`.

GitHub Actions is externally blocked before every first step because the
account's Actions payment/budget limit is exhausted. This is not a source or
test failure. No cloud or six-platform success is claimed until billing is
available and a deliberately requested run succeeds.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `a335617c499f9544d3255255836464c01bf3e865c03a2602b734650db4d891f7` |
| `crates/seacad-dxf-core/src/lib.rs` | 820 | `469622530a8d4d5a9ce065f34a1072cff90a7f09ea24a6183cb2bde17bdbf148` |
| `crates/seacad-dxf-core/src/named_symbol_table.rs` | 267 | `134194544ff5992e56cb9abb43970bf9fdab2a33ab05e58855fc5ba243bf7d4b` |
| `crates/seacad-dxf-core/src/dimstyle_table.rs` | 137 | `143f4d4b09f2c89008d6655593dc04f9d137a8de2f1b15cd8aa782a7776419f9` |
| `crates/seacad-dxf-core/src/dimstyle_handle_target_validation.rs` | 283 | `dbab86cc04046d2c4b6433ed37c7d13fbbc5a34db51326b4c51281af707d9787` |
| `crates/seacad-dxf-core/tests/dimstyle_handle_resolution_tests.rs` | 447 | `bbce1a70d8225f147fe92512ca43eaa252ecad57190c239178ea24d63cbf2ab2` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 310 | `bbf0ad8fe89827f1629910267b4aa17950b9d1330454495b184d8e8016230566` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,672 | `0d30a90cd2e7b2124fa4b71eb505945266404ef6ff8ddfe865bed98c16eb4f0a` |
| `docs/SUPPORT_MATRIX.md` | 1,368 | `5493eb997db6adce719c64d64a1ca6447cdb4b5ec99491a647cf0904c8a464c7` |
