# R0.2 Provenance, Ownership, and Boundary Inventory

Date: 2026-08-08 (`Asia/Saigon`)

## Decision

R0.2 inventories the current repository ownership evidence, contribution
history, generated and derived data, fixtures, dependency licenses, and the
exact proposed public/private split. The inventory is complete enough for R0.3
workspace-export design, but it does **not** authorize relicensing or source
publication.

The repository remains governed by the current SeaCad Proprietary License.
`MIT OR Apache-2.0` remains only the proposed future license for the reviewed
format layer. No file changes license at this checkpoint.

## Baseline and method

- Baseline commit: `a84d1985958d58949326f2c857d4e3e8e4e51210`.
- Baseline Git tree: `907` tracked paths.
- Baseline history: `363` commits reachable from `HEAD`.
- Inventory sources: committed Git objects, workspace manifests and lockfile,
  schema/source registries, generated-file headers, M1 provenance receipts,
  dependency/legal receipts, corpus policies, and the committed legal bundle.
- Uncommitted named-symbol/XDATA work is concurrent DXF work outside R0.2 and
  is excluded from ownership and boundary counts.
- Legacy trees, private corpus roots, registry sources, and vendor parser source
  were not read or copied for this checkpoint.

This is an engineering provenance inventory, not legal advice. A Git author,
committer, copyright string, or repository license is not treated as proof of
beneficial ownership or authority to relicense.

## Repository identity and contribution evidence

| Evidence | Observed baseline | Interpretation |
| --- | --- | --- |
| Git authors | 1 unique identity | `SeaCad <209146803+seaflower205@users.noreply.github.com>` |
| Git committers | 1 unique identity | Same identity as author |
| Reachable commits | 363 | Attribution is internally consistent |
| Git signature state | 363 `N` | No reachable commit has a verifiable Git signature |
| `Signed-off-by` trailers | 0 | No DCO chain is recorded |
| CLA or assignment artifact | none found | No contributor/relicensing authority record exists |
| Workspace author | `SeaCad` | A package metadata label, not a legal-person identity |
| Current project license | proprietary | Remains authoritative |

The single-identity history reduces the number of people requiring review but
does not answer who owns the work, whether it was created as employment or
contract work, whether rights were assigned to a legal entity, or whether the
identity has authority to publish under a new license. R0 cannot infer those
facts.

Before any open license is activated, the owner must provide a dated
attestation that identifies the legal copyright holder, confirms authority to
relicense the reviewed public boundary, and accounts for assisted or generated
contributions under the applicable service terms. If another contributor or
rightsholder exists, a compatible assignment, license grant, or exclusion is
required.

## Current license and dependency surface

The root `LICENSE` grants no public use, copy, modification, or distribution
permission. All three workspace crates inherit that exact license file and are
`publish = false`:

| Current crate | Intended R0 disposition |
| --- | --- |
| `seacad-dxf-core` | Candidate public format core after R0 approval |
| `seacad-cli` | Private product/verification tooling |
| `seacad-schema-gen` | Mixed; split required before public export |

The locked graph has 29 packages: three proprietary workspace packages and 26
reviewed crates.io packages. `release/sbom.cdx.json` records all 29 components.
`release/legal/manifest.json` records all 26 third-party packages and their
legal artifacts. The deterministic legal bundle contains the required project
and package license/notice files.

The runtime dependency families used by the format core are compatible with
the proposed target subject to their existing conditions:

- `sha2` and its locked transitive graph: `MIT OR Apache-2.0` compatible;
- `encoding_rs`: `(Apache-2.0 OR MIT) AND BSD-3-Clause`, with WHATWG notice;
- `unicode-ident` and the frozen Johab data retain their Unicode notices;
- CLI-only Clap, Serde, and JSON dependencies do not enter the minimal public
  format-core dependency surface unless R0.3 explicitly chooses otherwise.

Compatibility of dependencies does not relicense SeaCad-authored files.
Third-party license texts and notices must remain exact in every source or
binary distribution that includes their covered code or data.

## Legacy and external-source boundary

M1 accounted 907 legacy DXF assets and approved zero direct transfers. Its
binding dispositions remain unchanged:

- 253 legacy production-code files remain clean-room rewrite references only;
- 109 legacy tests may inform re-expressed requirements, never source copying;
- 13 small DXF fixtures remain quarantined pending ownership attestation;
- two real drawings remain private-corpus-only and must never be committed;
- 223 fuzz seeds remain quarantined because mutation lineage and licenses are
  undocumented;
- legacy documents and oracle notes are leads only until independently
  reproduced from normative sources.

The committed M1 CSV contains hashes and dispositions, not reusable fixture
authority. None of its legacy code, tests, fixture bytes, or private drawings
is admitted into the proposed public export.

Autodesk documentation is normative evidence. The repository stores reviewed
facts, stable topic identifiers, source identifiers, and normalized fact
hashes; it does not vendor Autodesk documentation or parser source. R0 legal
review must still consider the protectability and selection/arrangement of the
schema datasets before publication. Oracle observations remain factual test
evidence and do not grant rights to vendor software or data.

## Generated and derived artifacts

| Artifact family | Input/provenance | R0.2 disposition |
| --- | --- | --- |
| `schema/dxf/v1/*.json` | SeaCad-reviewed normalized facts with Autodesk source identifiers and hashes | Candidate public input; legal review required |
| `generated/header_schema.rs` | Deterministic `seacad-schema-gen` output | Candidate public generated output with its generator/input |
| `generated/entity_schema.rs` | Deterministic `seacad-schema-gen` output | Candidate public generated output with its generator/input |
| `johab_decode_le.bin` | Derived from Unicode-hosted Microsoft CP1361 decode records | Public candidate under Unicode License v3 notice obligations |
| `release/sbom.cdx.json` | Deterministic locked-workspace inventory | Regenerate for each exported workspace; current file remains private-release evidence |
| `release/legal/**` | Exact upstream package legal files | Redistribute only for packages included by the export |

The Johab table is 131,072 bytes with SHA-256
`d04a1a13d5f4706df6fa46394cda98a817570e774d601acd042e0fa57249f7ea`.
Its source, canonical map, Windows comparison, generated output, and Unicode
License v3 notice are already recorded. The input data file and vendor oracle
artifacts remain outside the repository.

Generated Rust cannot be published without the corresponding reviewed schema,
the minimum reproducible generator, and all applicable notices. Generated
headers are evidence of reproducibility, not a separate ownership grant.

## Fixture and corpus inventory

- No tracked path at the baseline ends in `.dxf`, `.dxb`, `.dwg`, or `.dgn`.
- DXF test inputs are newly authored inline builders and temporary files inside
  SeaCad Rust tests; they are covered by the same unresolved SeaCad ownership
  attestation as the surrounding source.
- `corpus/offline-manifest.json` and `corpus/release-manifest.json` are
  aggregate-only policies, not corpus content or evidence that the release
  threshold was achieved.
- No customer name, path, file hash, source identity, or private CAD byte is
  admitted into the public candidate boundary.
- The M13 private 1,000-file/10-GiB threshold and 20 consecutive six-native
  receipts remain release gates, not public-source inputs.

No standalone redistributable CAD fixture set currently exists. R0.3 must not
claim one. Future fixtures enter the public export only with explicit author,
license, provenance, and SHA-256 receipts.

## Exact proposed public/private boundary

The boundary below is normative for R0.3 design. `Public candidate` means
eligible for a future export after R0 legal approval; it does not change the
current license.

| Current path/component | Classification | Required R0.3 treatment |
| --- | --- | --- |
| `crates/seacad-dxf-core/**` | Public candidate | Export complete source, inline tests, generated modules, and Johab data |
| `schema/dxf/v1/**` | Public candidate | Export all seven files with source registry and deterministic hashes |
| Schema-generation logic in `crates/seacad-schema-gen/src/main.rs` | Public candidate | Extract the minimum DXF schema generator into the public workspace |
| `crates/seacad-schema-gen/Cargo.toml` | Mixed | Replace with a public generator manifest that excludes release-only bins |
| `crates/seacad-schema-gen/src/bin/seacad-release-*.rs` | Private | Keep in product/release tooling |
| `crates/seacad-cli/**` | Private | Keep out of the minimal format workspace |
| `corpus/**` and corpus receipt tooling | Private release qualification | Export no private corpus or current product harness by default |
| `release/**` | Private release evidence | Regenerate only the dependency legal subset needed by the public workspace |
| `.github/**` | Private repository operations | Design separate public CI in R0.3; do not copy secrets or runner policy |
| `.agents/**`, `AGENTS.md` | Private development process | Exclude from distributable format source |
| `audits/m1/**` | Private provenance evidence | Exclude legacy/private drawing hashes from public export |
| DXF contracts and selected DXF audit receipts under `docs/**` | Mixed public evidence | R0.3 must name an allowlist; exclude product roadmap and private operations |
| `docs/IMPLEMENTATION_PLAN.md`, product architecture and future product plans | Private product planning | Exclude from public format workspace |
| Root Cargo/toolchain/dependency-policy files | Mixed | Generate minimal public equivalents from the exact exported member set |
| Root legal files | Mixed | Replace project license only after authorization; retain applicable notices |
| Future CAD model, GUI, renderer, commands, scripts, plugins, SDKs, MCP, AI | Private product | Must never become dependencies of public format crates |

The public candidate dependency direction is strictly:

```text
minimal format foundation (when extracted)
  <- seacad-dxf-core
  <- public DXF schema inputs and minimum generator
```

No private product crate may be required to build, test, document, or regenerate
the public candidate. R0.3 must prove that split without changing generated DXF
output.

## Relicensing gaps and stop conditions

R0.2 closes inventory only. The following block any license activation:

1. No legal copyright-holder identity or authority attestation is recorded.
2. No DCO, CLA, assignment, or equivalent contributor-rights chain exists.
3. Assisted/generated contribution rights have not been expressly accounted.
4. Autodesk-derived normalized schemas need legal review before publication.
5. The mixed schema-generator crate has not been physically split.
6. Public documentation and audit allowlists have not been designed.
7. No standalone redistributable fixture set is approved.
8. The public workspace does not yet have an independent manifest, lockfile,
   CI, SBOM, legal bundle, or reproducibility receipt.

If legal review cannot clear any candidate file, R0.3 must exclude or clean-room
replace it. No compatibility, schedule, or test-coverage benefit overrides a
provenance failure.

## R0.2 acceptance contract

R0.2 is acceptable when mechanical verification proves:

1. the baseline commit, contributor counts, signature/DCO counts, and tracked
   path count above match Git evidence;
2. the current proprietary license and all three `publish = false` manifests
   remain unchanged;
3. M1 still approves zero direct transfers and no CAD fixture bytes are tracked;
4. the Johab artifact and schema/generated hashes match their committed
   receipts and generator checks;
5. SBOM/legal package counts remain 29/26 and dependency gates pass;
6. the public/private/mixed table names every current workspace crate and every
   repository root that could enter an export;
7. no support matrix, Rust source, schema, fixture, dependency, lockfile, legal
   artifact, release artifact, or support claim changes in R0.2;
8. concurrent named-symbol/XDATA work is unchanged during verification;
9. repository quality and Markdown/diff gates pass.

## Expected implementation effect

- Runtime behavior and public APIs changed: none.
- Support claims changed: none.
- Licenses activated or permissions granted: none.
- Dependencies, schemas, fixtures, corpus data, and release artifacts changed:
  none.
- Next authorized roadmap work after acceptance: R0.3 public-workspace export
  design, followed by explicit legal/provenance authorization at the R0 exit.
