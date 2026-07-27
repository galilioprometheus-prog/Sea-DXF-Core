# M4.2 ASCII structure index receipt

Status: completed 2026-07-27

This receipt records deterministic evidence for one-pass section accounting
and the compact group-code 0 index. Every fixture is synthetic and contains no
private CAD data.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/ascii_index.rs` | `f1e4aaab561ecfd12f10a497bed0d19ea354f0a96054d458e536e8a32edaaf80` |
| `crates/seacad-dxf-core/src/ascii_document.rs` | `e9d77d77d395ce36aca56e9cf93890484d305d468d3d9b262d24715f5d79bee0` |
| `crates/seacad-dxf-core/src/diagnostic.rs` | `c845b92ef13a0bab100cf53222098a680df6a0972462cf9c876f100a5c7025e1` |
| `crates/seacad-dxf-core/src/lib.rs` | `c21f3ef3cb76f98f47daf39d555b3f8edbeb287025b17f2b5fb2f35403934ead` |

M4.2 changes no Cargo manifest or lockfile and imports no legacy/external code
or fixture.

## Synthetic fixture evidence

| Case | Bytes | SHA-256 | Required result |
| --- | ---: | --- | --- |
| seven known + `ACDSDATA` unknown | 363 | `dfc619ad3dc2d3e2cc70de17494de91997c345477ef4e39bb4322f15b746675b` | 39 total, 38 inside, 1 outside, 8 closed non-overlapping sections, 25 group-zero entries |
| orphan/interrupted/unclosed | 95 | `7d6e5571dc4efcf6c4e3d4a833821d8efb56c67503817e63107351075aa3b241` | all four malformed states source-anchored; ranges remain disjoint |
| group-code 9 name candidate | 34 | `fb4d656aa76902a64f436d74eb942c001843e4f4e620b6ec55daf1cc0a678264` | invalid name, no HEADER guess |
| lowercase/padded markers + recovered EOF | 51 | `41e2c6fef44dc45144ba769d2c58c5afaaadebd6b74930e71d156c555510d1b5` | only exact section semantics; Compatible EOF still bounds accounting |
| 10,005 orphan ENDSEC markers | 90,051 | `93b7c98dc29311b05d25504a252e738213dc970439b5cd3752ca10a5c8cee593` | 10,000 retained Safe diagnostics, `DXF-W0001`, no unbounded growth |

The standard fixture reads the exact unknown name back through its span and
proves both accounting equations. The malformed fixture proves the equations
again across interrupted ranges. The existing counting source test proves
dialect discovery, structure indexing, framing, and SHA-256 still consume each
source byte exactly once.

## Review size and tests

M4.2 adds 486 production lines and removes four, within the repository's
200-500-line micro-milestone target. It adds six focused structure tests plus
accounting assertions to the existing single-read test and stable-code checks
for all four new diagnostics. Section metadata is capped by test at 72 bytes;
each group-zero entry is one `u32`.

The checkpoint requires Rustfmt, workspace Clippy with warnings denied, every
workspace test, strict UTF-8 validation, forbidden-production-pattern scan,
`git diff --check`, unchanged dependency manifests, and the three-platform
GitHub Actions matrix.
