# M4.3c1 Autodesk and Unicode escape reference audit

Status: completed 2026-07-27

This audit records the public evidence used for the CIF decoder and the
fail-closed MIF boundary. No external implementation, table, or fixture was
copied, translated, or ported.

## Primary references

- Autodesk,
  [Group Code Value Types Reference](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm):
  DXF string storage may use ASCII, UTF-8, CIF, and MIF; AutoCAD 2004 and older
  write plain ASCII and CIF; Unicode values are represented by control
  sequences. Autodesk publishes an exact `\U+hhhh` sequence example and lists
  the string-valued group-code ranges.
- Autodesk,
  [About Working with Drawings in Earlier Versions](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-Core/files/GUID-65D81338-0F34-4B9B-B500-CFEAFEDA58AD.htm):
  unconvertible text may be stored using CIF `U+nnnn` or MIF `M+nxxxx` codes.
- Autodesk,
  [AcString](https://help.autodesk.com/view/OARX/2026/ENU/?guid=OARX-RefGuide-AcString):
  Autodesk describes its string class as aware of CIF `U+xxxx` and MIF
  `M+nxxyy`.
- Unicode Consortium,
  [The Unicode Standard 17.0, Chapter 3](https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/):
  a high-surrogate code unit followed by a low-surrogate code unit represents
  one scalar; isolated surrogate units are ill-formed and are not scalar
  values.

References were reviewed on 2026-07-27.

## Evidence-backed decisions

- SeaCad recognizes only the exact documented uppercase control prefixes.
  Raw bytes remain available if another producer uses a case variant.
- Four hexadecimal positions are parsed as one 16-bit unit. Both ASCII cases
  of hexadecimal digits have the same numeric value.
- Adjacent surrogate controls are combined according to UTF-16. Isolated or
  reversed surrogate controls fail without replacement.
- MIF syntax is recognized and surfaced with its selector and 16-bit payload,
  but no Unicode mapping is claimed. The reviewed Autodesk references do not
  provide the complete selector mapping needed for such a claim.
- Escape decoding remains a layer after storage decoding. It cannot replace
  the source ID, codepage decision, occurrence, or raw byte span.

## AutoCAD oracle attempt

An isolated AutoCAD Core Console 2027 run was attempted with synthetic
AC1009 `TEXT` inputs, `/readonly`, `/safemode`, and per-case user data outside
the repository. The process entered Autodesk's error reporter before opening
the first fixture. It was terminated after bounded observation.

No oracle stdout, semantic result, profile, error report, or fixture is
committed or used as evidence. The failed run neither increases nor decreases
the support claim. CIF behavior in this checkpoint is grounded only in the
published Autodesk syntax and the Unicode surrogate rules above; MIF remains
explicitly unsupported.
