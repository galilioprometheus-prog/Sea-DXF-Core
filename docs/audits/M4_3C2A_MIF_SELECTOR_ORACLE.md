# M4.3c2a MIF selector and AutoCAD oracle audit

Status: completed 2026-07-27

This audit establishes the MIF selector table and observable AutoCAD behavior
without copying, translating, or porting external source.

## Published references

- Autodesk,
  [AcString](https://help.autodesk.com/view/OARX/2026/ENU/?guid=OARX-RefGuide-AcString):
  the current ObjectARX reference publishes the MIF shape `M+nxxyy`.
- Autodesk,
  [About Working with Drawings in Earlier Versions](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-Core/files/GUID-65D81338-0F34-4B9B-B500-CFEAFEDA58AD.htm):
  AutoCAD may store otherwise unconvertible text with `M+nxxxx`.
- Microsoft,
  [Code Page Bitfields](https://learn.microsoft.com/windows/win32/intl/code-page-bitfields)
  and
  [Code Page Identifiers](https://learn.microsoft.com/windows/win32/intl/code-page-identifiers/):
  932 is Japanese/Shift-JIS, 936 is Simplified Chinese, 949 is Korean
  Wansung, 950 is Traditional Chinese/Big5, and 1361 is Korean Johab.

These public pages identify the syntax and Windows pages, but do not connect
each MIF selector to one page.

## Historical Autodesk-authored evidence

The community-maintained
[`ADN-DevTech/objectarx_sdks`](https://github.com/ADN-DevTech/objectarx_sdks)
archive contains an Autodesk-copyrighted ObjectARX 2012 header at commit
`08b6206d911efc53bc52146d487f823d53ccca09`,
`Arx2012/inc/AdCharFmt.h`, Git blob
`68a298b0bd87e3e67b3d17c4197f31028bfe0e9a`.

The reviewed comments and implementation associate selectors 1 through 5
with Windows pages 932, 950, 949, 1361, and 936 respectively, treat
`00xx` as one byte, preserve high-byte/low-byte order otherwise, and accept
upper- or lowercase `M`. This archive is supplemental evidence rather than a
normative DXF reference. No header text, table, algorithm, SDK, or binary is
stored in or linked into SeaCad.

## AutoCAD 2027 behavioral oracle

Oracle executable:

- path: `C:\Program Files\Autodesk\AutoCAD 2027\accoreconsole.exe`;
- product/file version: `26.0.60.0.0`;
- SHA-256:
  `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`.

Each synthetic AC1009 ASCII DXF was opened from a temporary directory with
`/readonly`, `/safemode`, and an isolated per-case registry/data profile.
The script ran `LIST` over its single `TEXT` entity. No user drawing, plugin,
autoload directory, save, or network operation was involved.

AutoCAD returned the listed semantic text with no `ERROR` line. This Core
Console build returned process code 1 at script exhaustion after producing
the complete `LIST` result; therefore the support evidence is the extracted
entity text and its hashes, not the process code.

| Case | Fixture SHA-256 | Extracted text | UTF-8 result SHA-256 |
| --- | --- | --- | --- |
| `\M+182A0` | `c90f09963600ebd437f3abcf977a020d3538a0ff38d7e6240cbe59683a79a0c4` | `AあB` | `cadd59533d79c4805497d046f2ee757066ccb92e568f28c8d34f3f0479457725` |
| `\M+2A440` | `fdf5e68f844502c0c790c02e4c9445bbd7811a6ff15c4281357c50084ffc0f87` | `A一B` | `80e45846315126d80b8831f75fa2f90d5ee9e951804344ab83556370085b1c57` |
| `\M+3B0A1` | `3050b52f88aecf19d16962eb8bd5372ed078ca0ee4f8d77975813180a7d41680` | `A가B` | `748344ad86703abf5fe5f4893be39080ed7d872241e22eaf79b33a863cf4074a` |
| `\M+48861` | `087a166f46eba64a5f138abfcf490f891bbbb7d3d69cb002b7bce81b727b79a5` | `A가B` | `748344ad86703abf5fe5f4893be39080ed7d872241e22eaf79b33a863cf4074a` |
| `\M+5C4E3` | `fbe4f43b6b4acc28f7f7103327747e73387b306ea3316e427de410f87bc321c0` | `A你B` | `f65f181f144750127db520059212031af9a1fc1fd14db441e84417aacb855de4` |
| `\M+10041` | `477f3883c5ccde9b61846d4f8eb2afe18e480ddc0ff9e03064c99833c08ba879` | `AAB` | `c50128cf99c06e860afbffcc4ddda158ddf9b1e7cce03899251cf667229066ab` |
| `\m+182A0` | `07e800b6fca562dfc2ef5538ec93947931ce15568915d4184ae80850ef130117` | `AあB` | `cadd59533d79c4805497d046f2ee757066ccb92e568f28c8d34f3f0479457725` |
| selector 0 | `f73fc5508dd6fdbc5e46ce9153da1ae92a0f8d109a8bdc96b695824e8d1f3231` | `A\M+00041B` | `84ee39d98d018aa8ae0ef3bb3c8516095347b0b58bbb953dfc791ac6a968d8c4` |
| selector 6 | `63f287a4efe6d8e167a21ba396ba712ae1336b8bdae27e246992a770c84e3ebd` | `A\M+60041B` | `39df4e01a375ba9e9146ec1de434d523989f4f7c3920288d12eca34134515dfc` |
| selector 1, two single bytes `41 42` | `c8674fc1bb5cf58b21a96280d8bb6fbcd23eadcab4f13fc311568e5a0d383949` | `A\M+14142B` | `bccd5c07ba594466363948d714ed915272c4bfc257857ddc97c009493343a424` |

The fixture files, logs, registry keys, and isolated profiles remain outside
the repository. Only their hashes and extracted observations are retained.
The two-single-byte case also establishes that a nonzero high byte must decode
as one multibyte character; two independently valid single-byte characters do
not form a valid MIF control.

## CP1361 boundary

Windows PowerShell's strict `Encoding.GetEncoding(1361)` maps `88 61` to
`U+AC00` (`가`), and AutoCAD independently produced the same result for
selector 4. That proves the selector meaning and one vector, not the complete
cross-platform CP1361 mapping.

The Unicode Consortium's historical `JOHAB.TXT` is under an `OBSOLETE` tree
and explicitly is not a maintained Windows CP1361 authority. M4.3c2a therefore
does not generate a table from it. CP1361 remains fail-closed pending a
separate audited M4.3c2b implementation.
