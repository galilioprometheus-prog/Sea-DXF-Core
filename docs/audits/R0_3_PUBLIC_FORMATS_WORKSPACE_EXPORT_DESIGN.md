# R0.3 Public `seacad-formats` Workspace Export Design

Date: 2026-08-08 (`Asia/Saigon`)

## Decision

R0.3 defines a reproducible, allowlist-only export design for a future public
`seacad-formats` workspace. It does not create that workspace, move or copy
source, change generated output, activate an open license, publish a crate, or
change any current support claim.

The initial export has exactly two workspace members:

1. the current `seacad-dxf-core` source and tests; and
2. the minimum deterministic DXF schema generator, separated from private
   release tooling as `seacad-dxf-schema-gen`.

The current SeaCad Proprietary License remains authoritative. Export generation
is prohibited until the R0 legal/provenance exit explicitly authorizes the
project license and exact source boundary.

## Baseline

- Baseline commit: `a0e5ec66b856d22dc278187da5937b17f8e083ba`.
- R0.2 boundary decision: `r0.2-provenance-ownership-boundary`.
- Current public-candidate core: 433 tracked files, 6,240,044 bytes.
- Reviewed DXF schema: seven tracked files, 103,938 bytes.
- Current mixed generator crate: five tracked files, 207,705 bytes.
- Minimum schema-generator source: `src/main.rs`, 3,006 lines, 118,048 bytes,
  SHA-256
  `4c6f5074fd5f7654d9ca74fe3f56d1386c3f4ffd75519e6ff52026f70c96fdb2`.
- Concurrent named-symbol/XDATA work remains outside R0.3 and is not copied,
  edited, staged, or counted by this design checkpoint.

## Export invariant

The export is derived from one exact private-repository commit. It is not a
second independently edited source tree. Every exported path has:

- one source path or an explicitly identified generated/export-authored path;
- one disposition and applicable license class;
- byte length and SHA-256 receipt;
- no collision after slash and case normalization;
- no symlink, submodule, absolute path, `..`, device name, alternate stream,
  executable payload, or unlisted file;
- deterministic lexical ordering in manifests and archives.

The exporter fails closed on a missing allowlisted path, an unexpected output,
hash drift, license ambiguity, generated-output drift, dependency-closure drift,
or a private-path match. There is no copy-everything-and-delete blacklist mode.

## Exact future workspace layout

```text
seacad-formats/
|-- Cargo.toml
|-- Cargo.lock
|-- rust-toolchain.toml
|-- rustfmt.toml
|-- deny.toml
|-- README.md
|-- LICENSE-APACHE
|-- LICENSE-MIT
|-- NOTICE
|-- THIRD_PARTY_NOTICES.md
|-- crates/
|   |-- seacad-dxf-core/
|   |   |-- Cargo.toml
|   |   |-- src/
|   |   `-- tests/
|   `-- seacad-dxf-schema-gen/
|       |-- Cargo.toml
|       `-- src/main.rs
|-- schema/dxf/v1/
|   |-- manifest.json
|   |-- sources.json
|   |-- header.bootstrap.json
|   |-- entity_topics.json
|   |-- entity_aliases.json
|   |-- entity_applicability.json
|   `-- entity_common_fields.json
|-- docs/
|   |-- SUPPORT_MATRIX.md
|   |-- DEPENDENCY_POLICY.md
|   |-- PROVENANCE.md
|   `-- contracts/
|-- legal/
|   |-- manifest.json
|   `-- packages/
|-- sbom.cdx.json
`-- export-manifest.json
```

No empty `seacad-format-foundation` crate is invented. That crate enters the
public workspace only after a later reviewed extraction has real bounded-source
contracts, preserves DXF behavior, and proves dependency direction. Until then,
`seacad-dxf-core` remains self-contained.

## Workspace manifest contract

The future root manifest contains exactly:

```toml
[workspace]
members = [
    "crates/seacad-dxf-core",
    "crates/seacad-dxf-schema-gen",
]
resolver = "3"
```

Shared package metadata pins Rust `1.97.1`, edition `2024`, the authorized
repository URL, and the authorized dual-license expression. `publish = false`
remains on both members until a separate release checkpoint assigns a stable
version and explicitly approves registry publication.

Workspace lints retain `unsafe_code = "forbid"` and deny production `expect`,
`panic`, `todo`, `unimplemented`, and `unwrap`. The root lockfile is generated
with `--locked`-compatible exact versions and committed. It is derived only
after the member set and licenses are authorized; the private repository's
three-member `Cargo.lock` is not copied as the public lockfile.

## `seacad-dxf-core` export

The source mapping is one-to-one:

```text
crates/seacad-dxf-core/**
  -> crates/seacad-dxf-core/**
```

The complete crate is exported, including inline/integration tests, generated
Rust modules, and `johab_decode_le.bin`. No private crate is added as a normal,
development, build, optional, target, feature, example, benchmark, or test
dependency.

The member keeps only exact reviewed dependencies:

- `encoding_rs = 0.8.35`, default features disabled, `alloc` only;
- `sha2 = 0.11.0`, default features disabled.

Generated source remains committed so consumers do not need the generator to
build the core. Regeneration remains a required contributor and CI check.

## Schema-generator split

The current `seacad-schema-gen` crate is mixed because one manifest exposes the
DXF generator plus three private release binaries. R0.3 maps only:

```text
crates/seacad-schema-gen/src/main.rs
  -> crates/seacad-dxf-schema-gen/src/main.rs
```

The three excluded private binaries are:

- `seacad-release-evidence`;
- `seacad-release-packager`;
- `seacad-release-receipts`.

The public generator manifest declares only `serde 1.0.229` with `derive,std`,
`serde_json 1.0.151` with `std`, and `sha2 0.11.0`, all with default features
disabled. It has one binary target and no build script.

The generator already resolves its root as two parents above
`CARGO_MANIFEST_DIR`. Preserving `crates/<generator>` plus `schema/dxf/v1` and
`crates/seacad-dxf-core/src/generated` therefore preserves all current path
constants without changing generator logic. R0.3 does not rename identifiers
inside `main.rs`; any cosmetic rename is deferred until byte-identical output
has been proven in the exported layout.

## Generated-output identity

The public generator must reproduce these exact current artifacts:

| Output | Required SHA-256 |
| --- | --- |
| `generated/header_schema.rs` | `34675e171859d3075f70344e80c96217ed7994c7dd4e38b6bf09de33b6699b7c` |
| `generated/entity_schema.rs` | `0872b9e0bb85361a549fc77ee5265d05d4420cc8796ddb75ffd2ce6c83eaa610` |

Verification runs once from the private workspace and once from the proposed
export tree, then compares both generated byte streams and normalized input
hash comments. A match inside only one tree is insufficient. `--check` must be
read-only; intentional regeneration uses an explicit reviewed write mode.

## Public dependency closure

The current exact closure for the two proposed members is 23 packages: two
workspace packages plus 21 registry packages. It includes the core's encoding
and hashing graph and the generator's Serde/JSON graph.

The public closure excludes exactly these current CLI-only packages:

- `seacad-cli`;
- `clap`, `clap_builder`, and `clap_lex`;
- `anstyle`;
- `strsim`.

Closure is recomputed from public `cargo metadata --locked`, not filtered from
the private SBOM by package name. Any new package, duplicate version, Git
source, unknown registry, wildcard requirement, feature drift, build script, or
interpreted artifact requires its own dependency review before export.

## SBOM and legal bundle

The current private SBOM has 29 components and the private legal manifest has
26 third-party packages. Neither file is copied into the public workspace.

Public artifacts are regenerated from the two-member closure and must contain:

- exactly 23 SBOM components at the R0.3 baseline;
- exactly 21 third-party legal-package entries;
- the authorized project license files and project notice;
- exact upstream root legal files for every included registry package;
- the WHATWG, Unicode, BSD-3-Clause, MIT, Apache-2.0, and Unlicense material
  required by the selected graph;
- the Johab Unicode License v3 attribution and disclaimer.

The legal generator rejects missing, unexpected, modified, or symlinked files.
Project notices must not mention private agent skills, private runners, private
corpus operations, or excluded CLI-only packages.

## Documentation allowlist

R0.3 does not export the private `docs/` tree wholesale. The public documentation
set is limited to:

- one new export-specific `README.md`;
- `docs/SUPPORT_MATRIX.md`;
- `docs/DEPENDENCY_POLICY.md`, filtered only if private-tool sections can be
  removed without weakening dependency requirements;
- one new `docs/PROVENANCE.md` containing the authorized, non-sensitive portion
  of R0.2 evidence;
- these format contracts under `docs/contracts/`:
  - `M2_SOURCE_CONTRACT.md`;
  - `M3_ASCII_FRAMING_CONTRACT.md`;
  - `M4_1_DIALECT_CONTRACT.md`;
  - `M4_2_STRUCTURE_INDEX_CONTRACT.md`;
  - `M4_3A_ENCODING_POLICY_CONTRACT.md`;
  - `M4_3B1_CODEPAGE_DECODER_CONTRACT.md`;
  - `M4_3B2_SOURCE_ANCHORED_TEXT_VIEW_CONTRACT.md`;
  - `M4_3C1_CIF_MIF_ESCAPE_CONTRACT.md`;
  - `M4_3C2A_MIF_MAPPING_CONTRACT.md`;
  - `M4_3C2B_JOHAB_DECODER_CONTRACT.md`;
  - `M4_3C3_TEXT_CONTROL_TOKENIZER_CONTRACT.md`;
  - `M5_1A_BINARY_WIRE_REGISTRY_CONTRACT.md`;
  - `M5_1B_BINARY_GROUP_CURSOR_CONTRACT.md`;
  - `M5_2A_BINARY_RAW_DOCUMENT_CONTRACT.md`;
  - `M5_2B_BINARY_ENVELOPE_INDEX_CONTRACT.md`.

The CLI replay contract, corpus contracts, product master plan, product
architecture, active implementation plans, Antigravity batches, private
toolchain/runbook documents, and all 339 current audit receipts are excluded by
default. A future receipt may be added only through a named allowlist review
that removes private paths, customer hashes, runner identities, oracle binaries,
and product/release operations without rewriting historical evidence.

## Export manifest

`export-manifest.json` is a deterministic receipt with schema
`seacad-formats-export/v1`. It records:

- source repository commit and tree identity;
- export policy version;
- each source path, destination path, disposition, byte length, and SHA-256;
- every export-authored path and its generating policy;
- public Cargo member/package closure and lockfile SHA-256;
- schema-input and generated-output hashes;
- SBOM and legal-manifest hashes;
- toolchain and CI-policy hashes;
- total path and byte counts.

It contains no private source path, username, hostname, local drive, timestamp,
customer identifier, private corpus commitment, or credential. Its path set is
sorted and duplicate-free. The export archive hash is separate from the
manifest so archive framing cannot redefine source identity.

## Public CI design

Public CI uses GitHub-hosted or equivalently isolated public runners and no
private self-hosted label, path, cache root, environment secret, or execution
policy. The required matrix is:

- Linux x64 and ARM64;
- Windows x64 and ARM64;
- macOS x64 and ARM64.

Every target checks the authorized toolchain, dependency policy, format,
generator drift, Clippy, tests, and diff cleanliness. At least one job rebuilds
the public SBOM/legal bundle and export manifest from scratch. The current
private six-native release receipt sequence remains a product release gate and
is not replaced by source-workspace CI.

Network access is limited to normal pinned Rust/tool/action acquisition.
Workflows pin actions by full commit, disable credential persistence, grant
read-only contents by default, and define explicit concurrency and timeouts.
No publishing token exists until a separate release milestone.

## API, version, and publishing policy

The initial export remains version `0.0.0`, `publish = false`, and explicitly
pre-stable. R0.3 does not promise SemVer stability for the detailed evidence
APIs. A later DXF Core 1.0 gate may define a small stable facade while leaving
evidence-heavy APIs unstable or feature-scoped.

Repository visibility, source availability, crate publication, package
versioning, signing, release archives, and a license change are separate
decisions. Passing an export reproduction test grants none of them.

## Reproduction and equivalence gates

The future export implementation is acceptable only when one clean command
produces an isolated tree and proves:

1. every output path is allowlisted and every allowlisted source path exists;
2. the core source/test/data bytes equal the authorized source commit;
3. both schema-generated Rust files are byte-identical across private and
   export workspaces;
4. public metadata has exactly two members and a 23-package closure;
5. excluded CLI/release/corpus/private-operation names do not enter source,
   metadata, SBOM, legal, documentation, or CI surfaces;
6. all required third-party notices and legal files are present;
7. the public six-target CI definition has no private runner or secret;
8. dependency, format, generator, Clippy, test, documentation-link, archive,
   and diff gates pass from the isolated tree;
9. no current private-repository file is modified by export;
10. a second export from the same commit produces byte-identical manifest and
    archive bytes.

## R0.3 non-claims and stop conditions

R0.3 is design-only. It does not:

- authorize `MIT OR Apache-2.0`;
- prove copyright ownership or assignment;
- create or publish a repository, crate, package, SBOM, legal bundle, or
  archive;
- split the current generator crate;
- change `seacad-dxf-core`, schema, dependencies, or generated output;
- export audit history or fixture/corpus bytes;
- satisfy DXF Core 1.0, private corpus, or 20-nightly release gates.

Implementation stops if legal authorization is absent, a candidate path cannot
be cleared, generated bytes differ, dependency closure expands unexpectedly,
or a private identifier reaches the export. The remedy is exclusion or a
separately reviewed clean-room replacement, never weakening the gate.

## R0.3 acceptance contract

This design checkpoint is acceptable when verification proves:

1. the baseline and R0.2 boundary references are exact;
2. the two-member layout and generator split name every included/excluded
   current generator target;
3. the 23-package inclusion and six-package exclusion counts match metadata;
4. generated-output hashes match current committed artifacts;
5. the documentation allowlist excludes all 339 current audit receipts;
6. six public CI target pairs are explicit and private runner state is absent;
7. license activation, publication, source copying, core changes, and support
   changes are explicitly denied;
8. no current Rust, schema, manifest, lockfile, license, fixture, corpus,
   support matrix, release artifact, or workflow changes belong to R0.3;
9. concurrent named-symbol/XDATA paths are unchanged during verification;
10. repository quality and diff gates pass.

## Expected implementation effect

- Production/source files changed: none.
- Generated files changed: none.
- Dependencies or manifests changed: none.
- Licenses or publishing state changed: none.
- Support claims changed: none.
- Next R0 action after acceptance: obtain explicit legal/provenance
  authorization for the exact boundary, or keep the repository proprietary and
  do not execute the export.
