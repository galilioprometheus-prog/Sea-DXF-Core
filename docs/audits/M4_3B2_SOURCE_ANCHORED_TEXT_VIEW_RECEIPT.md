# M4.3b2 source-anchored text view receipt

Status: completed 2026-07-27

This receipt records deterministic implementation, resolution, provenance, and
resource-bound evidence. All vectors are synthetic and contain no private CAD
data.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/diagnostic.rs` | `da713ac20be1cd4b6d2bbc9eb8ded48409ff502458b185202643b849a515c1bd` |
| `crates/seacad-dxf-core/src/encoding.rs` | `30e7542b316e1b305e852884f6b3137cfa8746151872ff647da52ba8c6c44c01` |
| `crates/seacad-dxf-core/src/lib.rs` | `66b0512a21fcff97f1ec66349f4ca94f8ed4e4278038fba947a001dc2716cfd0` |
| `crates/seacad-dxf-core/src/source.rs` | `385e944e07b7e39d8f147cb8ea45f84a294bf933d1a82b8c811bbd398b141cb7` |
| `crates/seacad-dxf-core/src/text_decoder.rs` | `35359f65e0487137f65debf277641a56c42fb94e4efe78802db462b6a98a8f79` |
| `crates/seacad-dxf-core/src/text_view.rs` | `23ae1b209403ddfbc40201333afc743597bf53ecd542330f383bdc9c8bf6774f` |

M4.3b2 changes no Cargo manifest, lockfile, feature, dependency, or external
source. It continues to use only the reviewed M4.3b1 `encoding_rs` surface.

## Synthetic vector receipts

The fixture shape is one AC1009-or-later HEADER and one group-code 1 value in
ENTITIES, with LF framing and exact EOF.

| Case | Bytes | SHA-256 | Required result |
| --- | ---: | --- | --- |
| UTF-8 scalar split at the 4 KiB source boundary | 4,202 | `255e5cf3a1ef0a0c6bd56584088f71a699bb02840779cd4aa2d15c04980ba193` | `Complete`; exact 4,097-byte output and provenance |
| `ANSI_932`, source `82 A0`, exact three-byte destination | 115 | `5e926a1de6f390a30331da6928aa17280c7c78f668a7ab2a528579a627318c38` | `U+3042`; `Complete` without padding |
| AC1018 `ANSI_1361` | 118 | `1dde0d0464f6a8408122d44e12097f0b6248c307933dc58d787b7c8fc847bb31` | `UnsupportedLegacy`, `DXF-E0423`, no value read |
| AC1018 without declaration | 109 | `a469ad5469d955ffc4635819873e5d61cbfe86af88b0f310420071bc39dc1b12` | `Indeterminate`, no value read |
| malformed modern UTF-8 `C3 28` | 107 | `6731cfc487cf37658eca03c5beabb04c377cb1a55aa816af4e3d8e5e2f8f3df9` | `Malformed`, no U+FFFD |
| 8 KiB ASCII with four-byte destination | 8,297 | `056c418ce285dcbc974462db6b7cafd33072d9777ad6631115e5af5dda95643e` | `OutputFull`; at most one 4 KiB source chunk read |

The version matrix test covers every supported AC1009-AC1032 value. Legacy
versions with `ANSI_1252` resolve to the exact legacy decoder; AC1021 and later
resolve to UTF-8. An AC1021 file with stale `ANSI_1361` remains UTF-8 and does
not receive the legacy-only diagnostic.

## Provenance and negative evidence

Every successful receipt is checked against the document `SourceId`, selected
group occurrence, and exact raw value span. A recording source proves legacy
decoder selection does not reread the declaration and that only the chosen
value span is read.

Unsupported and indeterminate resolutions return no decode result, do not
touch the destination, and perform no value I/O. An invalid occurrence returns
a path-redacted invalid-data error without output mutation. Output exhaustion
is retried from the original occurrence, and malformed input never contains a
replacement scalar in its defined output prefix.

## Review size and gates

M4.3b2 adds 363 physical Rust production lines:

- 239 in the new source-view module;
- 83 in parse-time resolution/reporting;
- 29 in the internal streaming decoder session;
- 12 across diagnostics, exports, and source-contract clarification.

It adds 390 physical test lines. The checkpoint requires Rustfmt, workspace
Clippy with warnings denied, all 112 workspace tests, stable diagnostic-code
coverage, forbidden-production-pattern scan, `git diff --check`, artifact-hash
verification, and the Windows x64/macOS ARM64/Linux x64 CI matrix.
