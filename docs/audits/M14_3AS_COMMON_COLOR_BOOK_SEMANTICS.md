# M14.3as Common Color-Book Semantics

## Scope

M14.3as parses common entity group 430 as a source-backed color-book envelope
and composes it with the same entity's reviewed indexed and true-color scalar
semantics without modifying source bytes.

## Sources

- Autodesk common entity codes define group 62 as the color number, group 420
  as a 24-bit color value, and group 430 as the color name:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk `acad_truecolordlg` documents color-book group 430 in
  `colorbook$colorname` format and returns the related groups 62, 420, and 430;
  its public example is `RAL CLASSIC$RAL 1003` with indexed color 40 and true
  color 16235019:
  `https://help.autodesk.com/cloudhelp/2018/DEU/AutoCAD-AutoLISP-Reference/files/GUID-E6FF435F-9E66-4F37-8770-2E3FB87B8E0B.htm`
- ObjectARX `AcCmColor` exposes separate color and book names and identifies
  the DXF color-name representation:
  `https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-__MEMBERTYPE_Methods_AcCmColor.html`

The user-authorized legacy repository was inspected read-only and contained no
usable color-book contract. No legacy or external code, fixture, data,
dependency, or color-book payload was copied, translated, vendored, or linked.

## Contract

- An explicit group 430 is structured only when its exact encoded value has
  one `$` byte and both sides are nonempty. Missing, empty, and multiple
  separators are distinct typed issues; no occurrence is guessed.
- The book and color names remain exact half-open spans in the immutable source
  and retain the original text encoding receipt. Payload bytes are not copied
  into the semantic directory or debug output.
- A structured color-book value also requires usable group-420 true color and
  group-62 indexed color semantics from the same entity. Their absence,
  cardinality, numeric, and domain failures remain embedded typed relation
  evidence. Group order has no semantic effect.
- Parsing is source-bound, cancellation-aware, allocation-free over the name
  payload, and scans in fixed 4-KiB chunks under the existing source limits.

## Nonclaims

This checkpoint does not open `.acb` files, prove that a book or color exists,
verify the external name-to-RGB/ACI mapping, normalize names, establish case or
character policy, admit color-book edits, review dialect applicability, add
family graph CRUD, or advance any entity topic to `Complete`.

## Verification

Paired ASCII/Binary tests cover all nine dialects, exact AC1009 absence,
Autodesk's public tuple, out-of-order 430/62/420 groups, every separator
failure, missing and invalid true color, duplicate indexed color, raw
provenance, cancellation, source identity, public bounds, and payload-redacted
debug output. Final gate counts, production diff, and artifact hashes are
recorded after the release gate. This audit intentionally omits its own hash.

The focused common-text suite passed 6/6 tests and the full workspace passed
868/868 tests. Generated schema and release-evidence checks, `cargo deny
--locked check`, formatting, workspace Clippy with warnings denied, workspace
tests, and `git diff --check` all passed. The production diff is 209 added and
23 removed lines: 204 added and 20 removed in the common-text projection plus
5 added and 3 removed in public exports. No production dependency changed.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 225 | `62f1208d0349ecb92095a1b70e669404d08de81078dee5dd11d44d6cd9cea0a7` |
| `crates/seacad-dxf-core/src/entity_common_text_semantic.rs` | 744 | `09c152c0daa43b6de244a733a9781185c39d7b7011b84969266e057075c4779c` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,031 | `06eafd675b49de3211040e2358f79c7db9baed35333e0c3116dd2504b4b747a8` |
| `crates/seacad-dxf-core/tests/entity_common_text_semantic_tests.rs` | 672 | `b6aa472ef211209e4f4c9541c7b280abc9d4a5ca7fbd1d55284046a446cb75eb` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 805 | `77f2cee0ca3f295f625182399cb8d208ec635ef4c5a9f14c6d03cb6aed911848` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,236 | `a2be23690e7016286f23b32ee17bb119c00666fe68c327551160217f1110d5da` |
| `docs/SUPPORT_MATRIX.md` | 1,896 | `430a0fbf376226bef936cfd10f0510e37879eaba849595f95ad677a6c9a0525f` |
