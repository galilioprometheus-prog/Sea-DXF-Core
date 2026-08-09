# SeaCad

**English** | [Tiếng Việt](README.vi.md)

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

Development is complete through checkpoint **M14.4h**. The current core can:

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
- project one fail-closed semantic POINT snapshot across documents into a
  destination-bound canonical family draft, with explicit local symbol and
  handle bindings and no insertion yet;
- compose that exact POINT projection immediately with the encoded XDATA owned
  by the same source entity, while standalone projection still rejects XDATA;
- carry POINT provenance through atomic insertion, strict verification, and
  create-new write journals with cleanup and an executable exact inverse;
- adapt AC1032 POINT clones to AC1009 only for evidenced legacy placement and
  `BY_LAYER` lineweight equivalence, rejecting non-representable values;
- transcode one exact text span between independently parsed DXF documents
  with bounded, replacement-free decode/encode and byte-exact Unicode
  round-trip verification before exposing destination bytes;
- apply that verified conversion to group-1000 XDATA strings, retain compact
  per-occurrence receipts through application/entity grouping and POINT clone
  writing, and keep APPID/LAYER names destination-bound rather than translated;
- transcode a POINT color-book name in group 430 with exact source-field
  provenance and retain its receipt through family/XDATA draft, insertion,
  strict verification, and create-new write journals;
- expose an audited entity-completion ledger that places POINT at verified
  mutation (level 5 of 6), with private-corpus qualification and current-
  checkpoint six-native CI retained as explicit release blockers;
- evaluate analytically ready rational or non-rational SPLINE points on demand
  with bounded homogeneous De Boor arithmetic, exact parameter-domain checks,
  typed failures, and no implicit tessellation;
- evaluate the matching rational SPLINE first derivative with a bounded
  derivative control polygon and homogeneous quotient rule, retaining the
  evaluated point and tangent vector together;
- report audited curve-family completion explicitly: SPLINE at geometry level
  4 of 6, HELIX at typed-semantics level 3 of 6, and neither as complete;
- retain exact subclass-scoped raw HATCH and modern MESH field evidence,
  including duplicate subclass scopes and colliding nested group codes,
  without assigning roles, decoding topology, or implying applicability;
- decode the 25 globally unambiguous HATCH scalar/text roles into exact wire
  domains while leaving boundary, seed-point, and pattern-line collisions raw;
- expose 25 fixed cardinality cards per exact HATCH subclass with compact
  member references and independent absent, unique, or multiple states;
- select fail-closed singleton HATCH scalars with reviewed extrusion defaults,
  finite/domain checks, and source-anchored invalid evidence;
- assemble one exact, non-normalized HATCH extrusion tuple per subclass while
  retaining explicit/defaulted component provenance and typed unavailable or
  zero-vector outcomes;
- partition each HATCH subclass around unique ordered group-91/group-75 fences,
  keeping nested boundary payload opaque and fail-closed on missing, duplicate,
  or reversed anchors;
- select an exact required HATCH elevation tuple from the isolated header,
  preserving signed zero and raw provenance while rejecting nested decoys,
  unavailable components, non-finite values, and nonzero planar components;
- group exact HATCH boundary paths by group-92 anchors, retain pre-anchor orphan
  fields and opaque per-path payloads, and compare observed paths with the
  decoded group-91 declaration;
- inspect and validate entity XDATA, including exact source and independently
  parsed destination APPID/LAYER evidence, per-application symbol/structure and
  per-entity capacity/coordinate/handle/payload-envelope readiness, per-value
  logical destination projection, canonical destination group encoding, and
  exact per-application/per-entity encoded grouping and destination draft-record
  composition, atomic insertion planning, and strict post-image XDATA
  verification and create-new XDATA write journaling, plus create-new,
  strictly reparsed, same-
  dialect staged group-1005 handle-replacement writes;
- generate deterministic schemas, CycloneDX evidence, legal bundles, native
  packages, and six-platform release receipts.

The exact support boundary is intentionally narrower than the raw inventory.
Recognizing an entity name does not imply complete semantic or editing support.
See [the support matrix](docs/SUPPORT_MATRIX.md) and
[the DXF entity completion subplan](docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md) for normative
details.

## Not supported yet

- DXF Core 1.0 release qualification is not complete.
- Complete typed semantics and CRUD for every DXF entity are not claimed.
- DWG, DGN V7, and DGN V8 are not implemented.
- Rendering, constraint solving, and a native B-rep modeling kernel are not
  implemented.
- Source files are never overwritten in place.
- The Rust API is not yet stable.

The long-term roadmap stages native clean-room DGN V7, DGN V8, and DWG work
after DXF Core 1.0. The base plan does not use ODA, RealDWG, Bentley SDK, or
another proprietary format runtime; insufficient evidence remains explicit
opaque or read-only support rather than guessed parsing or writing.

## Workspace

| Crate | Responsibility |
|---|---|
| `seacad-dxf-core` | Bounded lossless DXF framing, reviewed semantics and geometry, reversible edit planning, and verified writers |
| `seacad-cli` | Human and JSON inspection/verification commands plus redacted corpus receipts |
| `seacad-schema-gen` | Deterministic schemas, dependency/legal evidence, native packages, and receipt verification |

Important project documents:

- [Master implementation plan](docs/IMPLEMENTATION_PLAN.md)
- [Support matrix](docs/SUPPORT_MATRIX.md)
- [DXF Core 1.0 implementation subplan](docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md)
- [DXF entity completion subplan](docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md)
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
