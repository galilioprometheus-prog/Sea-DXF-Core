# M6.4d schema-wide HEADER directory

M6.4d adds one explicit lazy resolution pass from the complete raw HEADER
variable index to every field in the generated HEADER schema. It adds no new
field, semantic-support claim, dependency, parser recovery, cache, or writer.

## Contract

`DxfRawDocumentView::resolve_header_schema` returns an immutable
`DxfHeaderSchemaDirectory` tied to the source identity. The operation accepts
a shared cancellation token and checks it before allocation, periodically
during the raw-variable scan, and before returning. The directory has one
ordered entry per generated field and each entry exposes:

- stable schema ordinal, field ID, and canonical DXF name;
- `Absent`, `Unique`, or `Ambiguous`;
- the total exact occurrence count;
- the first and second exact raw `DxfHeaderVariable` evidence in source order.

Unknown and custom variables do not become schema entries, but remain
unchanged and fully accessible in `DxfHeaderVariableIndex`. No name is decoded,
trimmed, normalized, case-folded, copied, or inferred.

## Complexity and collision safety

For `F` generated fields and `V` raw HEADER variables, resolution:

1. computes the document-keyed fingerprint of each static schema name;
2. sorts `F` compact candidates by fingerprint;
3. scans the `V` raw variable records once and binary-searches candidate
   fingerprints;
4. compares every candidate against authoritative source bytes in bounded
   chunks before recording evidence.

The resulting bound is `O(F log F + V log F + candidate bytes)`, replacing a
future `O(F * V)` sequence of single-field lookups. Temporary candidate memory
and the returned directory are both `O(F)` and allocation failures are typed.
The field count is schema-controlled, while raw variable count remains bounded
by the selected document record profile.

A keyed fingerprint is never semantic evidence. A forced-collision test
proves that a same-length unknown name cannot be claimed as a schema field.
Source-identity mismatch fails closed.

## Verification

Tests cover all nine AC1009-AC1032 versions in ASCII and Binary, ordered schema
parity, absent/unique/ambiguous states, duplicate evidence, unknown retention,
256 unknown variables without extra name reads, an artificial fingerprint
collision, mismatched source identity, redacted Debug output, and bounded
`Copy + Send + Sync` metadata. A pre-cancelled operation performs no source
reads and returns a typed cancellation failure.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `995c99dfd6cc5546adf971e1c1f7d364c1dea8be287661fe528ef26f5812dc52` |
| `crates/seacad-dxf-core/src/raw_document.rs` | `fc245848c0cb2993d0d6848e89d0e6dd88dde577663d5b93fad9d7c2aa749f5f` |
| `crates/seacad-dxf-core/src/lib.rs` | `ebb784a0afd92848343557d8bfeb0bf2c200db70fa3aa790b1f2c164ad51132d` |
