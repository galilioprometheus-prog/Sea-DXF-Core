# M4.3a Autodesk text-storage reference audit

Status: completed 2026-07-27

This audit freezes the public evidence used for `$DWGCODEPAGE` discovery and
SeaCad's version-backed text-storage policy. No external implementation or
fixture was copied, translated, or ported.

## Normative public references

- Autodesk, [HEADER Section Group Codes](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm):
  `$DWGCODEPAGE` uses group code 3. Autodesk says it is set to the system code
  page when a drawing is created but is not otherwise maintained.
- Autodesk, [Group Code Value Types Reference](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm):
  DXF string values may use plain ASCII, UTF-8, CIF, and MIF. AutoCAD 2007 DXF
  and later write UTF-8; AutoCAD 2004 DXF and earlier write plain ASCII and
  CIF. Unicode control sequences are also documented.
- Autodesk's same HEADER reference maps AutoCAD 2007 to `AC1021`, establishing
  the exact supported version boundary used by SeaCad.

References were reviewed on 2026-07-27.

## Clean-room policy derived from the references

- `AC1021`, `AC1024`, `AC1027`, and `AC1032` select UTF-8 string storage by
  file version. A stale, absent, or malformed `$DWGCODEPAGE` cannot override
  that documented format rule.
- `AC1009` through `AC1018` require exactly one non-empty group-code 3
  declaration before SeaCad can select a legacy-declared policy.
- Because Autodesk says `$DWGCODEPAGE` is not maintained, its raw token remains
  provenance metadata rather than an unquestioned global truth.
- SeaCad never substitutes the host operating-system locale or a default
  Windows codepage. Missing, invalid, duplicate, or version-ambiguous evidence
  yields `Indeterminate`.
- Matching HEADER and `$DWGCODEPAGE` is byte-exact and case-sensitive. The raw
  declaration is not trimmed, decoded, normalized, or mapped in M4.3a.

This checkpoint does not claim to decode CIF, MIF, `\U+xxxx`, MTEXT formatting,
or any legacy codepage. Those require separate evidence and replacement-free
tests.
