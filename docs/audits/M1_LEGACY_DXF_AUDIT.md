# M1 legacy DXF asset audit

Status: complete; no legacy asset approved for direct transfer

Audit source ID: `legacy-cad-2026-07-23`

Legacy Git HEAD: `1f10da8ef39420c9290dc326b5bb1746d22ab280`

Manifest SHA-256:
`40dfd838eeb271823390ed83a31ff457e29e0d6ca70c0d0db281b376d817f674`

## Scope and method

M1 inspected the earlier DXF parser, lossless core, DXF adapters, tests,
fixtures, evidence, documents, oracle tools, and fuzz assets without modifying
the earlier workspace. Build outputs, scratch output, error reports, and
non-DXF format work were excluded. The deterministic audit script records a
relative path, byte length, SHA-256, Git state, provenance class, license state,
and disposition for every selected file. It never emits the physical source
path.

The old repository has no root `LICENSE` or `COPYING` file and no configured
remote. Its DXF adapter declares `acadrust 0.4.0`; the newer lossless crate is
local and untracked. These facts make direct code transfer inappropriate even
when an implementation appears independently written.

## Results

| Measure | Result |
| --- | ---: |
| Scoped assets | 907 |
| Exact source bytes accounted | 4,874,645 |
| Production-code files | 253 |
| Test files | 109 |
| Documentation/evidence/metadata files | 298 |
| Oracle/helper tools | 9 |
| DXF/DXB fixture files | 15 |
| Generated fuzz seeds | 223 |
| Git tracked clean / modified / untracked | 10 / 1 / 896 |
| Direct transfers approved | 0 |

The 15 CAD fixtures are all ASCII `.dxf`; M1 found no `.dxb` or Binary DXF
fixture in the selected legacy assets. Four fixtures have Git history under the
user's account. Eleven small, synthetic-looking fixtures are local and
untracked. Appearance is not proof of ownership, so all 13 non-private
fixtures remain quarantined until the user explicitly attests that SeaCad may
reuse their exact bytes.

The two `real_*` drawings are treated as private CAD data. Their hashes are in
the fixture receipt, but their bytes must remain outside the SeaCad source
repository.

## Binding dispositions

| Asset class | Count | M1 disposition |
| --- | ---: | --- |
| Production code | 253 | `rewrite_clean_room`: implement from public specifications and independently observed behavior; never translate line by line. |
| Tests | 109 | `reexpress_behavior`: write new SeaCad tests from requirements; do not copy test source. |
| Documents, evidence, metadata | 298 | `reference_only`: treat claims as leads and revalidate them against normative sources. |
| Oracle/helper tools | 9 | `rewrite_if_needed`: recreate the minimum tool at its owning milestone. |
| Small DXF fixtures | 13 | `quarantine_pending_attestation`: exact hash known, bytes not yet transferable. |
| Real DXF drawings | 2 | `private_corpus_only`: never commit to this repository. |
| Fuzz seeds | 223 | `quarantine_generated_seed`: mutation lineage and seed licenses are not documented. |

No old source code, test source, fixture bytes, corpus bytes, generated seed, or
external implementation was copied into SeaCad during M1.

## Knowledge allowed forward

The new implementation may carry forward only independently verifiable facts:

- raw bytes and complete record accounting are required;
- source spans, stable diagnostics, resource bounds, and byte-identical
  unchanged writes need explicit tests;
- normalized geometry and lossless framing are separate layers;
- AutoCAD/other readers may be isolated semantic oracles, never runtime parser
  dependencies;
- every support claim needs a new fixture, normative citation, test, and hash
  receipt in SeaCad.

Historical coverage numbers, semantic mappings, and prior-art notes are not
SeaCad support evidence until reproduced by the new core.

## Fixture decision required before M3

M2 does not require a DXF fixture, so work can proceed without weakening this
audit. Before M3 starts, the user should either:

1. attest ownership and approve exact reuse of selected small fixtures; or
2. keep all old fixtures quarantined and authorize new fixtures authored from
   the Autodesk DXF reference.

The second option is the conservative default.
