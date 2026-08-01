# M14.3am Common Transparency

## Scope

M14.3am reviews common entity group 440 as a typed transparency domain and
composes it with existing raw semantics and verified common-field edits.

## Sources

- Autodesk common entity codes reserve group 440 for the AcDbEntity
  transparency value with no omission default:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk's group-code reference defines 440–447 as Int32 transparency values:
  `https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm`
- ObjectARX defines method values `kByLayer = 0`, `kByBlock = 1`,
  `kByAlpha = 2`, and documents alpha 0 as fully transparent and 255 as fully
  opaque:
  `https://help.autodesk.com/cloudhelp/2019/ENU/OARX-RefGuide/files/OREF-AcCmTransparency__transparencyMethod.html`
  and
  `https://help.autodesk.com/cloudhelp/2019/ENU/OARX-RefGuide/files/OREF-__MEMBERTYPE_Methods_AcCmTransparency.html`

The user-authorized legacy repository was searched read-only and supplied no
usable group-440 typed behavior. No legacy or external code, fixture, data, or
dependency was copied, translated, vendored, or linked.

## Native receipt

On 2026-08-02, AutoCAD Core Console 2027 version 26.0.60.0.0 opened a temporary
AC1032 document outside this repository. Native `CHPROP` created three LINE
values and `entget` reported:

| CHPROP value | group 440 observation |
|---|---:|
| ByLayer | omitted |
| ByBlock | `16777216` (`0x01000000`) |
| 50 percent | `33554559` (`0x0200007F`) |

This receipt establishes the high method byte and low alpha byte. It is a
behavioral receipt, not a vendored fixture or runtime dependency.

## Contract

- `DxfEntityTransparency` exposes only `ByLayer`, `ByBlock`, and
  `ByAlpha { alpha }`, with exact Int32 round-trip methods.
- Method bytes 0 and 1 require a zero payload. Method byte 2 admits all 256
  low-byte alpha values and requires bits 8–23 to be zero.
- Unknown method bytes and reserved payload bits retain distinct typed issues.
- The ninth reviewed common scalar domain preserves generic
  explicit/defaulted/absent/invalid provenance and duplicate behavior.
- Valid edits use existing wire validation, strict reparse, semantic
  verification, and byte-identical inverse restoration. AC1009 cannot encode
  group 440 in Binary and continues to fail before planning.

## Nonclaims

This checkpoint does not compute effective ByLayer/ByBlock transparency,
convert UI percentages, render or plot alpha, establish dialect applicability,
or advance any entity topic to `Complete`.

## Verification

Focused typed-domain and existing-document suites cover all modes, alpha
boundaries, reserved payloads, unsupported methods, wrong value kinds, exact
provenance, ASCII/Binary parity across AC1009–AC1032, verified edit output,
inverse restoration, cancellation, source identity, and public bounds. Final
gate counts, production diff, and artifact hashes are recorded below after the
release gate. The focused suites passed 8/8 tests and the full workspace passed
850/850 tests. Generated schema and release-evidence checks, `cargo deny
--locked check`, formatting, workspace Clippy with warnings denied, workspace
tests, and `git diff --check` all passed. The production diff is 183 added and
1 removed lines: 151 in the transparency type, 28 added and 1 removed in common
domain integration, and 4 module/exports. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 198 | `4db7a90843cdd3cb8d2557dd760740b12f37ef772a211a198e8858c095bed585` |
| `crates/seacad-dxf-core/src/entity_common_field_domain.rs` | 406 | `4834838b6d851f1bcac035c76cc8a5cd4e536300fbe833d6eb053e1fb56fe5b4` |
| `crates/seacad-dxf-core/src/entity_transparency.rs` | 151 | `7cd76cf1b591330bc93b89280536daa446b19e0366248304f85cdd024f651379` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,005 | `65eec321922c61a647a99c0aab2846e31e802a30025e161de78ebbeb30fc0fe1` |
| `crates/seacad-dxf-core/tests/entity_common_field_domain_tests.rs` | 574 | `423bc7ab7e15f9f0655831b1e35fa98a12dc9302dacd02890988bcfbb6427862` |
| `crates/seacad-dxf-core/tests/entity_common_field_domain_semantic_tests.rs` | 470 | `0a8774ebd87597c5de41921ad53b614eb7e6eae6d9ba4eba077692f0b25aa9a1` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 726 | `0141afc7291c883fca3d4c8cf667c9d85e873fb17930af88f5cffcb5d1356d16` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,155 | `953a87464fb24598117a6fde66350a4cb3e476eaee4e890d5612672d9de1dd5a` |
| `docs/SUPPORT_MATRIX.md` | 1,816 | `4399e75641b0a34b59010cca5b7e3087a4f346d00d3454eb4ce422c67ab3a8af` |
