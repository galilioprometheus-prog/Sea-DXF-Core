# M3.5 inspect and verify CLI receipt

Status: completed 2026-07-27

This receipt records deterministic evidence for the first operational SeaCad
CLI. It uses synthetic byte vectors only and contains no private CAD data.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-cli/src/main.rs` | `d7fa3e30177189a2c828c36d35ff27b3754321cbd6611426b20ca6705dde542a` |
| `crates/seacad-cli/Cargo.toml` | `a8b84e48b48a02b85fc011d7795a97a84b57e0301ba318078addc96c0c61408a` |
| `Cargo.lock` | `8ed9311ef18db7a29a573f1730bf59aab7411e8c9e687a938be3b4ddeec1c26f` |
| `docs/CLI_JSON_V1.md` | `2c2d616e6c1cb0c59304eb398d4054eb4c711ef4a6f2caf79fb55aa72ef44a2b` |

The dependency decision, checksums, feature tree, build-script inspection, and
RustSec snapshot are recorded in `M3_5_CLI_DEPENDENCY_REVIEW.md`.

## Deterministic smoke vectors

| Vector ID | Bytes | SHA-256 | Command and required result |
| --- | ---: | --- | --- |
| `strict-ascii` | 36 | `dc919c8f27c3229e9c4a240278f07819b2ed960642d983ceefc6604750b621f2` | `inspect --json`: exit 0, `ok`, 4 groups; `verify --json`: exit 0, `verified` |
| `recovered-missing-eof` | 19 | `09fd60cb2c2df5b80b6c206cb16ba3f81c515c5182116ef8b2dc615bc92ff787` | `verify --mode compatible --json`: exit 1, `not_verified`, `CLI-E0004`, `DXF-W0203`, 2 groups |

All three reports used schema `v1`, identified `ascii_candidate`, and returned
`source.path: null`. The smoke run executed the built native Windows binary,
not only the in-process unit-test harness.

## Unit-test coverage

Six CLI tests cover:

- help, version, and usage exit codes;
- inspect defaults, JSON v1 keys, group count, and path redaction;
- explicit `--show-path` disclosure;
- Compatible recovery versus verification refusal;
- Strict verification, malformed group-pair errors, and path-redacted open failures; and
- Binary identification, unsupported status, unknown input, and source IDs.

JSON failures are written to stdout for automation; successful text is written
to stdout and failed text to stderr. Tests assert that private fixture paths do
not appear unless `--show-path` is explicit.

## Support boundary

M3.5 exposes M3 raw framing; it does not add version, section, entity, or
geometry semantics. Binary DXF is detected but deliberately returns
`CLI-E0002` until M5. English human output is complete for this checkpoint;
Vietnamese localization is the separate M3.6 checkpoint so it cannot alter
the JSON v1 machine contract accidentally.

## Checkpoint gates

The checkpoint requires Rustfmt, workspace Clippy with warnings denied, every
workspace test, forbidden-production-pattern scan, `git diff --check`, and the
three-platform GitHub Actions matrix. Final local counts and CI identity are
recorded in the checkpoint commit and tag history.
