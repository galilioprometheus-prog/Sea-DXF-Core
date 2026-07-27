# M5.1a Autodesk Binary DXF reference and AutoCAD oracle

Status: completed 2026-07-27

No external parser source was inspected, copied, translated, or ported.

## Normative Autodesk references

Autodesk, **About Binary DXF Files**:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm>

This establishes the 22-byte sentinel, pre-R13 one-byte group codes, the
pre-R13 byte-255 extended-data escape, R13-and-later 16-bit little-endian group
codes, every binary value representation, and the exclusion of group 999 from
conforming Binary DXF.

Autodesk, **Group Code Value Types Reference**:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

This provides the complete group-code-to-value-type range table and string
storage families. Autodesk, **About Extended Data**, distinguishes XDATA group
1004 as a binary chunk and documents its 127-byte semantic maximum:

<https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

An older Autodesk 2015 rendering of the Binary DXF page says one-byte group
codes apply before Release 14 and describes only older value families:

<https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm>

That wording conflicts with Autodesk's expanded current reference. SeaCad
records the conflict and uses the current pre-R13 rule, corroborated at the
available R12 and post-R13 boundaries by the oracle below. No R13/R14 fixture
claim is made from AutoCAD 2027 because SAVEAS exposes R12 then AutoCAD 2000 as
the adjacent available DXF export versions.

## AutoCAD 2027 isolated export oracle

Executable:

- version: `26.0.60.0.0`;
- SHA-256:
  `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`.

AutoCAD Core Console opened a known-valid AC1009 base under `/readonly`,
`/safemode`, and unique isolated profiles. `SAVEAS` selected DXF, an explicit
version, and Binary output, always writing a new path under the external oracle
directory. These files are behavioral fixtures only and are not committed.

| Artifact | SHA-256 |
| --- | --- |
| base AC1009 ASCII DXF | `5f2cab15b518ae14065daed7f16a9f7b6276d0bdb85caaf6cb3cbe7df57fd882` |
| R12 Binary export script | `ab2c57c39a8c9376edc3c1a7cafa04fe216d867b864093c8c7a7925cda4b2f69` |
| AutoCAD 2000 Binary export script | `4420bd259f045a1435680e7bd356f36aabf632fc9182bdd77defdaa3376f4c39` |
| AutoCAD 2018 Binary export script | `f1f39a347c04d0e73bfb50e4f39f8f980bd1a7598d3182d8dfb8bf1a8f2a6885` |
| oracle runner | `c30b0fb1d6472e2116cd4df706a8a3775fc1e0667c95b23acf3c9f9b0a4e109e` |
| R12/AC1009 Binary DXF, 3,926 bytes | `76f9f56bf9a4395c7bc324cf831c5e473cad68a5ee0a442975cc7647c08f5a04` |
| AutoCAD 2000/AC1015 Binary DXF, 112,153 bytes | `cd059a69f0c238f28807b2079b53aef6d04c68d0b486416ac2bede3df59de2c3` |
| AutoCAD 2018/AC1032 Binary DXF, 42,235 bytes | `4b5584619ef9ba4665f46caf268891b7dd54ad527955348325484d6f7c80b0ef` |

Observed bytes immediately after the sentinel:

- AC1009: `00 53 45 43 54 49 4f 4e 00` -- one-byte group 0 then `SECTION`;
- AC1015: `00 00 53 45 43 54 49 4f 4e 00` -- 16-bit group 0 then `SECTION`;
- AC1032 uses the same 16-bit group-code form.

The AC1032 output contains 80 raw occurrences of group 280. Examples include
`18 01 7f 00 09 00` and `18 01 02 00 09 00`: group 280, a two-byte value,
then group 9. Group 290 examples use `22 01 00 09 00`: group 290, one Boolean
byte, then group 9. This resolves the difference between ObjectARX API tables
that describe 280-series values as 8-bit integers and the Binary DXF wire
table, which encodes them as 16-bit values.

AutoCAD acceptance alone is not byte-identity evidence. The exports only
corroborate wire layout. M5.1a freezes those observations in allocation-free
decoder and registry tests; streaming and unchanged replay are later work.
