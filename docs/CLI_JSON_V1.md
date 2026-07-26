# SeaCad CLI and JSON schema v1

Status: M3.5 contract

M3.5 exposes the first operational SeaCad commands. They inspect lossless raw
ASCII DXF framing only; version, section, entity, and geometry semantics begin
in later milestones.

## Commands

```text
seacad inspect FILE [--mode strict|compatible] [--json] [--large] [--show-path]
seacad verify FILE [--mode strict|compatible] [--json] [--large] [--show-path]
```

`inspect` defaults to Compatible mode so explicitly permitted framing
recoveries remain visible. `verify` defaults to Strict mode. `--large` opts in
to the Large resource profile; Safe remains the default. `--json` selects the
stable machine-readable report. Human-readable output is English in M3.5;
Vietnamese localization is isolated to M3.6 and will not rename JSON keys,
codes, or status values.

Paths are redacted by default in both text and JSON. `--show-path` is the only
way to request the input path. Source IDs are SHA-256 identities of the bytes,
not filenames.

## Exit codes

| Exit | Meaning |
| ---: | --- |
| `0` | `inspect` opened the raw framing, or `verify` proved Strict framing |
| `1` | input/IO failure, unsupported format, invalid framing, or a recovered document that cannot be verified |
| `2` | command-line usage error reported by Clap |

Help and version display return `0`. A JSON report is written to stdout even
when the operation returns `1`; human-readable failures are written to stderr.

## Top-level JSON object

Every report contains every top-level key below. Optional values are encoded
as JSON `null`, not omitted.

| Key | Type | Contract |
| --- | --- | --- |
| `schema_version` | string | Exactly `v1` |
| `command` | string | `inspect` or `verify` |
| `status` | string | One of the stable status values below |
| `options` | object | Effective read mode, resource profile, and report language |
| `source` | object | SHA-256 source ID, byte count, and opt-in path |
| `format` | object | Physical probe result |
| `document` | object or null | Raw ASCII framing summary when opening succeeded |
| `diagnostics` | array | Stable core diagnostic codes, severity, and optional byte span |
| `error` | object or null | Stable code and English message for a failed outcome |

`options` contains `read_mode` (`strict` or `compatible`),
`resource_profile` (`safe` or `large`), and `language` (`en` in M3.5).
`source.id` is a lowercase 64-hex SHA-256 string when the complete source could
be scanned, `source.bytes` is an unsigned byte count when the file opened, and
`source.path` is `null` unless `--show-path` was explicitly supplied.

`format.physical` is `ascii_candidate`, `binary`, or `unknown`.
AsciiCandidate means only that the bounded physical probe permits ASCII
framing; it is not a validity or semantic-support claim.

When present, `document` contains:

| Key | Type | Meaning |
| --- | --- | --- |
| `conformance` | string | `strict` or `recovered` |
| `groups` | integer | Number of framed group-code/value pairs before any opaque tail |
| `eof_occurrence` | integer or null | Zero-based occurrence of the recognized EOF group |
| `trailing_bytes` | integer | Opaque bytes preserved after recognized EOF |
| `diagnostics_truncated` | boolean | Whether the profile diagnostic cap was reached |

Each diagnostic has a stable `code`, a `severity` of `info`, `warning`, or
`error`, and a `span`. A span is either `null` or an object with half-open byte
offsets `start` and `end`.

## Status and CLI codes

| Status | Meaning |
| --- | --- |
| `ok` | Inspect opened Strict-conforming framing |
| `recovered` | Inspect opened Compatible framing with one or more recoveries |
| `verified` | Verify opened Strict-conforming framing |
| `not_verified` | Compatible framing opened, but verification deliberately failed |
| `invalid` | Source or framing is invalid |
| `unsupported` | Physical format was identified but is not implemented yet |

| Code | Meaning |
| --- | --- |
| `CLI-E0001` | Internal CLI state or representation failure |
| `CLI-E0002` | Identified DXF physical format is not supported in this milestone |
| `CLI-E0003` | Source is not a recognized ASCII or Binary DXF candidate |
| `CLI-E0004` | Recovered framing is inspect/verbatim-only and cannot be verified |
| `CLI-E0005` | CLI output could not be written |

Core failures retain their stable `DXF-E...` codes and recoveries retain their
`DXF-W...` diagnostic codes. English `message` text is descriptive and is not
a machine contract; automation must branch on codes and status values.

## Current boundary

Binary DXF is positively identified and returns `unsupported` with
`CLI-E0002`; Binary parsing arrives at M5. Unknown/empty input returns
`invalid` with `CLI-E0003`. A Compatible malformed document may be inspected,
but it is never promoted to verified, semantic-editable, or canonical-writable
state.
