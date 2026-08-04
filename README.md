# SeaCad

SeaCad is a clean-room CAD kernel written in Rust. The first release target is
a lossless, resource-bounded core for ASCII and Binary DXF from AC1009 through
AC1032.

SeaCad is pre-1.0 software. Its source is currently visible under a proprietary
license; source availability does not grant permission to use, copy, modify,
or redistribute the project.

## Why SeaCad

CAD files combine long-lived engineering data with version-specific syntax,
extension payloads, and references that must not be guessed. SeaCad therefore
uses four rules throughout the core:

- preserve original bytes and provenance;
- keep unknown and proprietary payloads opaque instead of discarding them;
- reject ambiguous or invalid edits instead of silently repairing input;
- write to a new destination and independently re-open the result.

The project is a native implementation. Autodesk and other CAD products may be
used as isolated behavioral oracles, never as runtime parser dependencies or
sources of copied implementation code.

## Current status

Development is complete through checkpoint **M14.3da**. The current core can:

- frame and open bounded ASCII and Binary DXF AC1009-AC1032;
- retain immutable raw source identity and exact record provenance;
- expose reviewed HEADER, table, block, object, entity, handle, text, and
  geometry semantics without promoting unreviewed fields;
- preserve unknown, custom, proxy, and application-defined data as raw
  evidence;
- plan immutable edits with conflict checks and byte-identical inverse plans;
- write preserve-patch results to a new path, strictly reparse them, and verify
  typed postconditions;
- exercise the first verified entity CRUD path for canonical POINT records;
- inspect and validate entity XDATA, including verified same-dialect group-1005
  handle-replacement transactions;
- generate deterministic schemas, CycloneDX evidence, legal bundles, native
  packages, and six-platform release receipts.

The exact support boundary is intentionally narrower than the raw inventory.
Recognizing an entity name does not imply complete semantic or editing support.
See [the support matrix](docs/SUPPORT_MATRIX.md) and
[the entity completion plan](docs/DXF_ENTITY_COMPLETION_PLAN.md) for normative
details.

## Not supported yet

- DXF Core 1.0 release qualification is not complete.
- Complete typed semantics and CRUD for every DXF entity are not claimed.
- DWG, DGN V7, and DGN V8 are not implemented.
- Rendering, constraint solving, and a native B-rep modeling kernel are not
  implemented.
- Source files are never overwritten in place.
- The Rust API is not yet stable.

Future DWG and DGN work is expected to combine native parsing, exact opaque
preservation, and optional vendor bridges for features that cannot be matched
reliably by a clean-room implementation alone.

## Workspace

| Crate | Responsibility |
|---|---|
| `seacad-dxf-core` | Bounded lossless DXF framing, reviewed semantics and geometry, reversible edit planning, and verified writers |
| `seacad-cli` | Human and JSON inspection/verification commands plus redacted corpus receipts |
| `seacad-schema-gen` | Deterministic schemas, dependency/legal evidence, native packages, and receipt verification |

Important project documents:

- [Implementation plan](docs/IMPLEMENTATION_PLAN.md)
- [Support matrix](docs/SUPPORT_MATRIX.md)
- [DXF entity completion plan](docs/DXF_ENTITY_COMPLETION_PLAN.md)
- [Dependency policy](docs/DEPENDENCY_POLICY.md)
- [Toolchain and oracle policy](docs/TOOLCHAIN.md)
- [CLI JSON v1 contract](docs/CLI_JSON_V1.md)

Private CAD corpus files are never committed. Public fixtures may enter the
repository only after ownership, license, provenance, and hashes are reviewed.

## Build and verify

SeaCad pins Rust 1.97.1. From the repository root:

```console
cargo +1.97.1 build --locked --workspace
cargo +1.97.1 run --locked -p seacad-cli -- inspect drawing.dxf
cargo +1.97.1 run --locked -p seacad-cli -- verify drawing.dxf
```

Human output defaults to English. Vietnamese output is available with
`--lang vi`; JSON field names and status codes remain stable English
identifiers.

```console
cargo +1.97.1 run --locked -p seacad-cli -- inspect drawing.dxf --lang vi
cargo +1.97.1 run --locked -p seacad-cli -- verify drawing.dxf --json
```

Required local gates:

```console
cargo deny --locked check
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings
cargo +1.97.1 test --locked --workspace
git diff --check
```

GitHub Actions is intentionally manual and budget-aware. Ordinary development
is verified locally; the six-native release workflow is dispatched only at
reviewed checkpoints.

## Security and data handling

All CAD input is untrusted. Production Rust forbids unsafe code and disallows
panic-oriented shortcuts. Parsers enforce explicit resource limits, reports
redact paths by default, and writers create a new destination rather than
overwriting the source.

Please do not attach confidential or third-party CAD drawings to public issues.
Use a minimal synthetic reproducer whose redistribution rights are clear.

## Contributing

The repository is not accepting unrestricted code reuse or redistribution at
this stage. Before contributing code, fixtures, format observations, or derived
tables, open a discussion describing their origin and license. Contributions
must preserve the clean-room boundary and may not contain copied proprietary
code, SDK material, customer drawings, or data from unofficial software
distributions.

## Sponsorship and licensing

Copyright (c) 2026 SeaCad. All rights reserved. See [LICENSE](LICENSE).

Evaluation of the visible repository does not grant production, modification,
or redistribution rights. Anyone wishing to use SeaCad must first obtain a
separate written agreement from the project owner. The project intends to keep
small-scale access affordable and may grant an appropriate license in exchange
for a small project sponsorship agreed case by case.

A sponsorship payment by itself does not grant rights unless the accompanying
written agreement explicitly does so. Commercial embedding, hosted services,
redistribution, and vendor-SDK bridges may require different terms.
