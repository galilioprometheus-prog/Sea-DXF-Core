# M4.3a text-encoding policy receipt

Status: completed 2026-07-27

This receipt records deterministic evidence for `$DWGCODEPAGE` discovery and
version-backed text-storage policy. All DXF vectors are synthetic and contain
no private CAD data.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/encoding.rs` | `09c66d58453b038a41bb33a7fbf74a7dec236ee62a359e868d1671e94ef1357e` |
| `crates/seacad-dxf-core/src/ascii_document.rs` | `edaa4ffb3e8d237b6f46b4d7ea9a1bb32af6ba443ffe7ee71d6570e0f5723582` |
| `crates/seacad-dxf-core/src/diagnostic.rs` | `fa843e46e36d9dce9aa9ff5206b161c1949102cbbae6d6398fd9354c4a4a5a95` |
| `crates/seacad-dxf-core/src/dialect.rs` | `1ba52f8a196e1c5654152fe962923b36df8b2538a5d9ec5eb9764312d8b4b175` |
| `crates/seacad-dxf-core/src/lib.rs` | `d859ac8be1b587100a267cffbdf59eb9866abfbcaa2642b3e62abccd0cc784a9` |

M4.3a changes no Cargo manifest or lockfile and imports no external code or
fixture. The reviewed `encoding_rs` archive remains only in Cargo's cache and
is not linked into SeaCad.

## AC1009-AC1032 policy vectors

Each 81-byte vector contains one exact `ANSI_1252` declaration. AC1009-AC1018
must select `LegacyDeclared`; AC1021-AC1032 must select UTF-8 by version.

| Version | SHA-256 |
| --- | --- |
| `AC1009` | `a76420224fd4b3885a5ff57117734b52fbce39f6e5f2974706d0272a5cecef2b` |
| `AC1012` | `64015b86d12e2e66b546c1db697c18c6fe6d6428f82eac4cf04f777702c4ecff` |
| `AC1014` | `12a14d57e6ba95fafeeaa23644abce216bc89d114d4d178371a9e73498b3ce2d` |
| `AC1015` | `57dbefa53b8330fa85f43337b859f58a26673e1071798a1242140ad4f1894f69` |
| `AC1018` | `6e3d6bb3faf19808b7d0a77861f8a562af0064f69946ac87d7839914b50b9fa1` |
| `AC1021` | `4e4d5e8f79eb9f9dbfa825450be2f56d1a030c4a1b3675c9d394fe357b716fda` |
| `AC1024` | `a264a6bcd581341f6e6c6459e10924e86b14a84c434b7b2d02e358ae2a32078d` |
| `AC1027` | `21733c99e20fcb25f5fc3bf7bee63ffbb80abae246d3647834eb6a4879652b0f` |
| `AC1032` | `6021d62178199e0c090344b2c29e4644d3039c8b7e5c4843f7f12be23a1a635b` |

Every vector verifies source identity, exact variable/value occurrences and
spans, raw token readback, declaration state, effective policy, and absence of
diagnostics.

## Boundary vectors

| Case | Bytes | SHA-256 | Required result |
| --- | ---: | --- | --- |
| AC1018 without declaration | 54 | `f40457a43127f5ad8c0611949f556a1cde7ccaf0e0a47e44d9cb430396ed4043` | `Indeterminate`, `DXF-E0420`, HEADER span |
| AC1021 without declaration | 54 | `21ff247e9c2ba2fc6e48a774eede9ee336dd6a463bc13f02b03889282d8ba432` | UTF-8, no codepage diagnostic |
| empty group-3 value | 72 | `7811b42c7875d33cec556763d201ed5f08ce5fb1ba7e6ce461c6959f7627a1e7` | invalid, no fallback |
| wrong group 1 | 81 | `8b3e4b2649c41e34aa1e957d4274ca81f6af1dea9d15310052062667a8dc3030` | invalid; modern version still UTF-8 when used there |
| missing value at input end | 54 | `d65baa987697213b49e22db934f059972db98274427a1f0663f43ecc36bd8646` | `MissingValue`, Compatible raw open |
| three declarations | 133 | `00708f14040ed31255f0c3cee3862dfa067f09d7d70608ed0d613854adc4cb4e` | ambiguous, count 3, fixed evidence |
| wrong section/non-exact names | 164 | `4c071b8a382dbe6ed6dc6ebfeb71a4bfb2833f2ec377acd08d7df3a5d8c8ad74` | ignored exactly |
| ambiguous `$ACADVER` | 101 | `124c6942f9cbcdf74ed64602850e31de2b7387ee1fa37f0ad9770cce38747474` | valid declaration but encoding remains indeterminate |

## Review size and gates

M4.3a adds 375 production lines and removes two, within the repository's
200-500-line micro-milestone target. Seven encoding tests cover the policy
matrix and boundary cases; existing single-read integration proves the added
tracker performs no extra source I/O. Stable-code tests cover all three new
diagnostics.

The checkpoint requires Rustfmt, workspace Clippy with warnings denied, every
workspace test, strict UTF-8 validation, forbidden-production-pattern scan,
`git diff --check`, unchanged dependency manifests, artifact-hash
verification, and the three-platform GitHub Actions matrix.
