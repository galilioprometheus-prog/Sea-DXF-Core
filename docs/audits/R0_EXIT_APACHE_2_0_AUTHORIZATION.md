# R0 Exit: Apache-2.0 Authorization

Date: 2026-08-08 (`Asia/Saigon`)

## Decision

The repository controller responded `Apache-2.0` to the explicit R0 exit
question asking for confirmation of ownership/relicensing authority and the
license to apply to the exact R0.3 public boundary. SeaCad therefore records
**Apache License 2.0 only** as the authorized future project license for that
boundary.

This decision rejects the earlier tentative `MIT OR Apache-2.0` target. Any
dual-license wording in R0.1-R0.3 historical design evidence records the option
that existed before this authorization and is superseded by this checkpoint.
No SeaCad-authored public export may offer an MIT alternative without another
explicit authorization checkpoint.

## Authorization scope

The authorization applies only to the future allowlist-derived
`seacad-formats` workspace defined by R0.2 and R0.3:

- the complete authorized `seacad-dxf-core` source, generated modules, Johab
  data, inline tests, and integration tests;
- all seven reviewed `schema/dxf/v1` inputs;
- the minimum DXF schema generator mapped from the current generator main;
- export-authored manifests, public CI, documentation allowlist, provenance
  summary, SBOM, legal manifest, and notices required by the R0.3 contract;
- a future real `seacad-format-foundation` only after its own reviewed
  extraction and authorization.

The authorization does not apply to:

- `seacad-cli`, corpus tooling, release packager/evidence/receipt binaries,
  private workflows, agent files, product roadmap, or internal audit history;
- future CAD model, geometry, command, renderer, GUI, automation, plugin, SDK,
  MCP, AI, DWG, DGN, or product layers;
- legacy code, tests, quarantined fixtures, fuzz seeds, vendor documentation,
  vendor software, private corpus data, or customer drawings;
- any third-party component beyond the rights already granted by its own
  license.

Anything not explicitly admitted by the R0.3 export manifest remains private
and proprietary unless a later named checkpoint says otherwise.

## Current-repository license state

The current mixed private repository remains under the SeaCad Proprietary
License. Its root `LICENSE`, workspace `license-file` inheritance, package
`publish = false` settings, source visibility, and distribution permissions do
not change at R0 exit.

The Apache-2.0 authorization becomes operational only in a separately generated
and verified export containing no private path. Changing the current root
license would incorrectly grant Apache-2.0 rights over private product and
operational material, so it is prohibited.

No public repository, crate, package, archive, installer, binary, tag, or
release is created by this checkpoint.

## Required Apache-2.0 treatment

When the authorized export is eventually implemented, it must:

1. include the complete unmodified Apache License, Version 2.0 text as
   `LICENSE` or `LICENSE-APACHE` according to the export contract;
2. declare `license = "Apache-2.0"` for SeaCad-authored public workspace
   packages and metadata;
3. retain every applicable third-party copyright, license, attribution,
   disclaimer, NOTICE, Unicode, WHATWG, BSD, MIT, and Unlicense artifact;
4. distinguish SeaCad's Apache-2.0 grant from third-party license grants;
5. retain source provenance and generated-data receipts without suggesting
   Autodesk, Microsoft, Unicode, WHATWG, or another vendor endorses SeaCad;
6. keep `publish = false` and version `0.0.0` until a separate release
   checkpoint authorizes publication and versioning;
7. produce the exact R0.3 export manifest, public SBOM, legal manifest, and
   reproducibility receipts before any distribution.

The Apache-2.0 patent grant, contribution terms, NOTICE handling, and
termination provisions apply as written in the license. This engineering
checkpoint does not reinterpret them or provide legal advice.

## Contribution provenance after export

The current history has one Git identity but no signature, DCO, CLA, assignment,
or `Signed-off-by` chain. The controller authorization resolves the engineering
decision for the reviewed existing boundary; it does not create evidence for
future contributors.

Before accepting an external contribution to the public workspace, its public
repository must adopt a contribution policy requiring the contributor to grant
rights compatible with Apache-2.0 and preserve authorship. DCO, CLA, or another
rights mechanism must be selected explicitly before the first external merge.
R0 exit does not silently choose one.

Assisted/generated contributions remain subject to the contributor's authority
and applicable service terms. They cannot bypass provenance review merely
because the output is committed under an authorized Git identity.

## Superseded and retained evidence

R0.1-R0.3 audit receipts remain immutable historical evidence. Their hashes and
checkpoint tags are not rewritten. This exit decision supersedes only their
tentative future license selection:

- R0.1 established a possible open-core target;
- R0.2 found the ownership/provenance gaps and exact candidate boundary;
- R0.3 designed a non-executing export and retained `publish = false`;
- this R0 exit records the controller's Apache-2.0-only authorization.

All clean-room, unknown-data preservation, dependency, resource-bound, corpus,
six-native, and support-claim gates remain unchanged.

## R0 exit result

R0 governance work is complete when this authorization passes mechanical
verification and receives its checkpoint commit/tag. The authorized engineering
state is then:

- exact open candidate boundary: defined;
- chosen future license: Apache-2.0 only;
- current private repository license: unchanged;
- export implementation/publication: not performed;
- unsupported or provenance-unclear files: excluded or still blocked;
- next product implementation program: resume DXF Core 1.0 at the first
  uncompleted checkpoint after M14.3dc.

The export itself remains a separate future implementation and release action.
R1 work may continue in the private repository without executing that export.

## Acceptance contract

The R0 exit checkpoint is acceptable only when verification proves:

1. the decision states Apache-2.0 only and rejects a new MIT grant;
2. the authorization is limited to the exact R0.3 allowlist-derived boundary;
3. the current root proprietary license, manifests, lockfile, and
   `publish = false` settings are unchanged;
4. no Apache/MIT project license file, public export, package, or archive is
   created in the current repository;
5. third-party licenses and notices remain authoritative and unchanged;
6. no Rust, schema, generated output, fixture, corpus, support matrix, release
   artifact, or workflow change belongs to this checkpoint;
7. concurrent named-symbol/XDATA paths remain byte-exact during verification;
8. master-plan live wording identifies Apache-2.0 as the authorized target and
   points to this decision;
9. repository quality and diff gates pass.

## Expected implementation effect

- Production behavior/API/support claims changed: none.
- Current source license changed: none.
- Future public-boundary license authorized: Apache-2.0 only.
- Export, publication, package, or release performed: none.
- Next checkpoint after R0 exit: R1 resumes the preserved DXF Core 1.0 plan.
