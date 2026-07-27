# M4.1 typed dialect discovery receipt

Status: completed 2026-07-27

This receipt records deterministic evidence for the AC1009-AC1032 registry and
source-anchored ASCII `$ACADVER` discovery. All vectors are synthetic and
contain no private CAD data.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/dialect.rs` | `ebee79f0c1767dfabee2860e7a457310f2c4941b61550299ee3f00fd58439aed` |
| `crates/seacad-dxf-core/src/ascii_document.rs` | `1c9f4bf340f216ea8b511f47a534a55a07c3bcebf4e4238ca600d6b161fa78d4` |
| `crates/seacad-dxf-core/src/diagnostic.rs` | `2bd7d57aad40e0cade57914a444c321aa1a00c4c136fe2006621b60fd0eba3a9` |
| `crates/seacad-dxf-core/src/lib.rs` | `b0b2858024d2d6291129abca6419fa70e34da4e63106880b03d0d5b58e90630c` |

M4.1 adds no dependency and changes no Cargo manifest or lockfile. No
legacy/external source or fixture is imported.

## Supported registry vectors

Each 54-byte vector is the exact LF-terminated sequence
`0/SECTION, 2/HEADER, 9/$ACADVER, 1/<code>, 0/ENDSEC, 0/EOF`.

| Code | SHA-256 |
| --- | --- |
| `AC1009` | `b386a65777af863e800b7c357ebea581666073349b8edb2f136806a549a9eefb` |
| `AC1012` | `8fd7fe9c9d445abd8c15369dfa6fc624df4a7ffbafb9ceb12a49f138151c4464` |
| `AC1014` | `db34498f3cf40054ac4304bc867f7fee61467a60b0148e56239f2e700ddf74fd` |
| `AC1015` | `d50cb68021a81be30365d7f043e462d52703d5e81ee984a12fd877fa0b06ef6d` |
| `AC1018` | `f40457a43127f5ad8c0611949f556a1cde7ccaf0e0a47e44d9cb430396ed4043` |
| `AC1021` | `21ff247e9c2ba2fc6e48a774eede9ee336dd6a463bc13f02b03889282d8ba432` |
| `AC1024` | `19a9604332471e938accee5259157ab5d13a863bfd17e1babf067f7aefbf9fa8` |
| `AC1027` | `bca259aaf597d985d6f03d69420b34ab774f7fb85fa487ac09820bd79042a50f` |
| `AC1032` | `5f25f87e0333fcb0922c7ec1ebd92beac8af31bef383e15249ef92e41c846055` |

Every vector verifies its typed version, `SourceId`, first occurrence, exact
variable/value spans, and raw span readback.

## Boundary vectors

| Case | Bytes | SHA-256 | Required result |
| --- | ---: | --- | --- |
| absent | 51 | `37fb7564b75aaf6548314e312542038cca36dd9b504a56c3e17a02ffcaa10720` | `Absent`, `DXF-E0401` |
| `AC1006` | 54 | `53fd2e924da25e1762cffe3bd61890f2fd96b7f78290d29ba4cdd7effdd8ed47` | `Unsupported`, exact raw retained |
| wrong value group | 53 | `d7a01322830c8ac4d458d43a2c7811a9a1da58317dde33841e26b6da64a33138` | `Invalid`, no guessing |
| missing value | 30 | `ecffc6481036d8b4f4c9f1064ed705df33eef2853ae5491da0364a59dc0a83ed` | `Invalid/MissingValue` |
| three occurrences | 94 | `cde93c943efb555e181591bc516e3639968a5724904094d388db67f09d9c77a0` | `Ambiguous`, count 3, fixed evidence |
| other section/non-exact names | 125 | `b803766ca8d599303535f06c6aa2590f91e398f12318db8aac193dbcef03b93a` | ignored, `Absent` |
| valid AC1032, missing EOF | 48 | `e533ea7912e761e2e17734ecc0894963acd2c0e819df24fb189432c8c30ee490` | Compatible `Recovered`, dialect supported |

The absent vector also proves semantic discovery errors remain separate from
strict raw framing diagnostics and conformance.

## Review size and gates

M4.1 adds 432 production lines and removes four, within the repository's
200-500-line micro-milestone target. Seven dialect tests plus existing
single-read, compact-metadata, malformed-input, verbatim, source, and
diagnostic tests exercise the integration.

The checkpoint requires Rustfmt, workspace Clippy with warnings denied, every
workspace test, strict UTF-8 validation, forbidden-production-pattern scan,
`git diff --check`, unchanged dependency manifests, and the three-platform
GitHub Actions matrix.
