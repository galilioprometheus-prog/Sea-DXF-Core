# M6.5a numeric HEADER schema foundation

M6.5a adds the first numeric wire shapes to the generated HEADER registry. It
does not yet decode numeric values into a public semantic view and makes no
claim about defaults, version applicability, enum meaning, valid ranges, or
cross-field behavior.

## Normative evidence

The reviewed rows come from Autodesk's published HEADER variable table:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The existing M6.1a audit independently checked the normalized 206-row HEADER
inventory across the Autodesk 2015, 2018, 2021, 2024, 2025, and 2026
references. All six snapshots produced the same receipt:
`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`.
M6.5a reuses the already registered `autodesk.header.2026` source identity.

| HEADER variable | Published group code | Generated storage |
| --- | ---: | --- |
| `$ACADMAINTVER` | 70 | `Int16` |
| `$ANGBASE` | 50 | `Double` |
| `$ANGDIR` | 70 | `Int16` |
| `$ATTMODE` | 70 | `Int16` |
| `$AUNITS` | 70 | `Int16` |
| `$AUPREC` | 70 | `Int16` |

Every new entry remains `optional`, `not_yet_reviewed` for applicability and
default policy, and `shape_only` for review state.

## Closed wire-family validation

The generator accepts `Double` only for one reviewed M5 floating-point group
code in 10-59, and `Int16` only for one reviewed M5 signed 16-bit group code
in 60-79. It rejects a numeric storage kind outside its family. This is a
schema-integrity rule, not a claim that every code in either family is a
valid HEADER field.

## Verification

Generator tests prove deterministic nine-field output and fail closed when a
`Double` or `Int16` field crosses its wire-family boundary. Core tests resolve
all nine generated names over the shared raw document API for every AC1009-
AC1032 version in both ASCII and Binary. The Binary fixture carries a real
little-endian IEEE-754 group-50 payload and real little-endian signed group-70
payloads, rather than encoding numeric values as strings. Duplicate, unknown,
collision, cancellation, and source-identity behavior from M6.4d remains
covered.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `90ba005ab58b6dede12d1dd33f45cc4df1b3c55bd96cc1b000b360219f212854` |
| `crates/seacad-schema-gen/src/main.rs` | `fdce73770471d63508a3fec00740583acb6cfdae354b983093a25c8aab8f0145` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `2caea373509dcc2924baac9ad4fb44d402a1eb85ca9e30650253e08d6c4fc754` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `48388fa1f35a1f8f97b96b44f572b071041f89cc177fb82685607317b2109693` |
