# M4.1 Autodesk dialect reference audit

Status: completed 2026-07-27

This audit freezes the public evidence used for SeaCad's typed DXF dialect
registry and HEADER `$ACADVER` discovery. No external implementation or source
code was copied, translated, or ported.

## Normative public references

- Autodesk, [HEADER Section Group Codes](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm):
  `$ACADVER` has group code 1 and lists the release-code mapping.
- Autodesk, [HEADER Section](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-EA9CDD11-19D1-4EBC-9F56-979ACF679E3C.htm):
  each HEADER variable begins with group code 9 and is followed by its value
  group.
- Autodesk, [HEADER Section Example](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-2A01D125-C1C9-4B20-B916-0F5598C8F19E.htm):
  shows `$ACADVER` followed by group code 1 in an ASCII HEADER.
- Autodesk, [General DXF File Structure](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-D939EA11-0CEC-4636-91A8-756640A031D3.htm):
  sections start with `0/SECTION`, identify their name with group code 2, and
  end with `0/ENDSEC`.
- Autodesk, [Numerical Group Codes](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm):
  group code 9 identifies a HEADER variable.

References were reviewed on 2026-07-27.

## Frozen supported registry

| Exact code | Autodesk release name |
| --- | --- |
| `AC1009` | AutoCAD R11/R12 |
| `AC1012` | AutoCAD R13 |
| `AC1014` | AutoCAD R14 |
| `AC1015` | AutoCAD 2000 |
| `AC1018` | AutoCAD 2004 |
| `AC1021` | AutoCAD 2007 |
| `AC1024` | AutoCAD 2010 |
| `AC1027` | AutoCAD 2013 |
| `AC1032` | AutoCAD 2018 |

Autodesk also documents `AC1006` for AutoCAD R10. It is deliberately outside
SeaCad Core 1.0's approved AC1009-AC1032 scope and therefore classifies as
`Unsupported`, not as a supported dialect. Unknown future values are handled
the same way.

## Clean-room interpretation rules

- Matching is byte-exact and case-sensitive. SeaCad does not trim or normalize
  section names, `$ACADVER`, or its value.
- Only an exact group-code 9 `$ACADVER` inside an exact HEADER section is
  considered.
- Its immediate next group must be group code 1.
- Unsupported values remain in the immutable raw source. The report stores
  their byte span instead of copying or rewriting the value.
- Missing, malformed, and duplicate values are reported conservatively. No
  version is inferred from entity content, file dates, or nearby text.

AutoCAD 2027 is not required as an oracle for this checkpoint because the
implemented behavior is directly documented and the ambiguity policy is a
SeaCad fail-closed rule. The existing M3.2 AutoCAD receipt already supplies an
independent AC1032 framing vector.
