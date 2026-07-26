# M3.4 verified Verbatim writer receipt

Status: completed 2026-07-27

This receipt records deterministic evidence for the first SeaCad writer. It
contains only synthetic byte vectors and no private CAD data. Verbatim output
is proved by SeaCad's own source/output SHA-256 and byte-count checks; AutoCAD
save output is deliberately not used as a byte-preservation oracle.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/verbatim.rs` | `094ae36865e672ac2f4e5ca6bc37ccc4acdf9fc4a29f3aa994c48645f7f086ab` |
| `crates/seacad-dxf-core/src/error.rs` | `8144d916eeff388f1e154f63a1a343b9f5aab78ab8986eaf59c1077edae2b073` |

No dependency, legacy source, external parser, or fixture was added. The
writer reuses the M2.3-reviewed `sha2` dependency and Rust standard-library
file APIs.

## Deterministic byte vectors

Each vector is opened as a raw ASCII document, written to a unique absent path,
closed, reopened, and independently hashed. Tests require the output bytes and
receipt IDs to equal the input exactly.

| Vector ID | Bytes | Input and output SHA-256 | Coverage |
| --- | ---: | --- | --- |
| `strict_mixed_endings_no_final_newline` | 37 | `d01aad6e2f1f3ecad12fd450747c531ebf0e3d44d70a66e9d993f1931213d068` | Strict, CRLF/LF/CR mixture, EOF without terminator |
| `compatible_bom_padded_eof` | 11 | `0d62665023c48c0b14c2300a0548cd5a1fd8ae756fceb8150df3390838a34771` | BOM and EOF whitespace remain unchanged |
| `compatible_missing_eof` | 19 | `09fd60cb2c2df5b80b6c206cb16ba3f81c515c5182116ef8b2dc615bc92ff787` | Recovered missing EOF remains unchanged |
| `compatible_tail_after_eof` | 17 | `d68c017bc2a740319ade1a905d41e3e9e10ba10efadc6b7ee41fb39f1f75f02d` | Group bytes and SUB after EOF remain opaque/exact |
| `compatible_empty` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Empty recovered source creates verified empty output |

A separate synthetic value crosses multiple 64 KiB chunks during both copy
and verification. Its progress events are required to start at zero, increase
strictly, and finish at `2 * source_len`. File-backed and memory-backed sources
are both covered.

## Failure and safety evidence

| Case | Required result |
| --- | --- |
| Destination already exists | `Create/AlreadyExists`; existing bytes unchanged |
| Source changes after document open | `DXF-E0301`; created output removed |
| Observer cancels after copying begins | `DXF-E0002`; partial output removed |
| Output length differs on reopen | `DXF-E0302` |
| Output hash differs on reopen | `DXF-E0303` |
| Empty source | New zero-byte file with identical empty SHA-256 receipt |

The public API has no overwrite switch. It uses `create_new`, never writes the
input path, never logs a destination path, and returns only byte counts and
SHA-256 identities. Cleanup is best-effort but observable: if deletion of an
incomplete file fails, the returned path-redacted `Remove` IO error indicates
that a partial file may remain.

## Focused verification

The pre-checkpoint focused run executed 68 core tests with no failures,
including nine writer-specific tests covering multi-chunk, file-backed, and
direct output-verifier cases. The repository checkpoint additionally requires
formatting, workspace Clippy with warnings denied, all workspace tests, diff whitespace,
and the three-platform GitHub Actions matrix.
