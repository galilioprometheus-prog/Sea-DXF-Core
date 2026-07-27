# M6.5b source-anchored HEADER numeric view

M6.5b exposes the six shape-reviewed M6.5a fields as an immutable
`DxfHeaderNumericView`. It does not add schema defaults, version applicability,
enum meanings, valid-value ranges, unit conversion, mutation, caching, or a
writer.

## Normative basis

The field names and group codes remain backed by Autodesk's published HEADER
table:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

Autodesk's group-code value-type reference assigns group 50 to double
precision and group 70 to a 16-bit integer:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

Autodesk's Binary DXF reference specifies little-endian two-byte integers and
little-endian eight-byte IEEE double-precision values:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm>

These sources establish physical type and representation only. They do not
authorize SeaCad to invent a missing value or silently normalize malformed
input.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`header_numeric_view(cancellation)`. The fixed-size, `Copy + Send + Sync` result
contains:

- `$ACADMAINTVER` as `i16`;
- `$ANGBASE` as `DxfDouble`;
- `$ANGDIR`, `$ATTMODE`, `$AUNITS`, and `$AUPREC` as `i16`.

Each field is a `DxfSemanticValue` with generated schema provenance and exact
raw group occurrence/span evidence. A field is `Explicit`, `Absent`, or
`Invalid`; M6.5b never emits `Defaulted`.

Structural invalidity distinguishes wrong group code, a marker without a value
group, multiple value groups under one marker, and multiple exact markers.
Conflict evidence points to the second value or second marker occurrence when
available. Internal index/source inconsistencies fail closed as fatal typed
I/O data errors.

`DxfDouble` stores the exact binary64 bit pattern so equality and hashing do
not collapse signed zero or make NaN behavior ambiguous. Binary payloads,
including non-finite IEEE bit patterns, remain exact physical values; later
field semantics may reject them only with separate reviewed evidence.

## ASCII numeric policy and bounds

ASCII parsing is locale-independent. It trims only horizontal ASCII space and
tab around the token, accepts an optional sign, decimal mantissa, and optional
`e`/`E` exponent, and preserves the original value span. Integer overflow,
floating overflow, empty input, `NaN`, infinity spellings, and the first
unexpected byte are typed invalid states. A finite parsed `-0.0` retains its
sign bit.

There is no arbitrary short numeric cutoff. Source framing has already applied
the selected Safe or Large value-byte limit. Values up to 128 bytes use a
stack buffer; larger values use fallible allocation bounded by that profile.
Source reads are split into 8 KiB chunks with cooperative cancellation checks,
so a hostile but profile-valid numeric line does not create an unbounded or
uncancellable semantic read.

## AutoCAD 2027 isolated oracle

Autodesk's public reference does not fully enumerate accepted ASCII number
spellings. AutoCAD Core Console 2027 (`26.0.60.0.0`, executable SHA-256
`fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`)
opened synthetic AC1032 inputs under `/readonly`, `/safemode`, `en-US`, and a
unique `/isolate` profile outside the repository.

| Case | Bytes | SHA-256 | Exit | Open result |
| --- | ---: | --- | ---: | --- |
| `0.5` | 102 | `bbeb124a527360408ae59b1787a53d9ce1e54450614c3c8a5720873e5cd51c3b` | 0 | accepted |
| space + `+5.000000000000000E-1` | 122 | `0ede7c69badb10932207599a9bf037c81c342b579ae6e7e76104d71eee018b4c` | 0 | accepted |
| `.5` | 101 | `87137567a6f95ca7af468d12f53249454f604ad75c222d03e41047c5ce7f46f8` | 0 | accepted |
| `NaN` | 102 | `749af83bf51c46f86e3b30e9d77218c82c560a5d18513226574a64cb1995f1e2` | 53 | rejected at value line |
| `inf` | 102 | `44b49d6673d082103039fab7995fff45a4fd407f615840049ab1a5c4a11a3f24` | 53 | rejected at value line |
| `1e9999` | 105 | `857941975f90006b4ae4345c7fdf284a03ea4b3d567258732bab2f7c90c41a38` | 0 | accepted, coerced |
| `1e-9999` | 106 | `daafc1957cdc2c8706a09b39198f562e87792c0a2aa7d60f6ac7d40bf971fde0` | 0 | accepted, coerced |

The first three accepted forms all produced AutoCAD's internal `ANGBASE`
value `0.00872665` radians, consistent with the same raw DXF angle of 0.5
degrees. SeaCad deliberately returns the raw DXF numeric value `0.5` and does
not perform that application-level conversion.

AutoCAD accepted `1e9999` but reported internal `ANGBASE` as `0.0`. SeaCad
classifies that spelling as `OutOfRange` instead of silently coercing it,
because raw bytes remain authoritative and a lossy substitute is forbidden.
AutoCAD also accepted `1e-9999` but reported `1.74533e-101` radians rather than
the literal exponent's IEEE result. SeaCad detects a nonzero decimal
underflowing to binary zero and likewise returns `OutOfRange`.
The open-marker script SHA-256 is
`3b2eaa5efc74a439a7f66ad38376fe03473143fdf0b849c0a12ea08ac8f688ca`;
the value-report script SHA-256 is
`aed182a29941543b0c1602a58d635f6cc056e9b6595fba921d7c0b24409c9002`.
Oracle fixtures, profiles, and logs remain outside the source repository.

## Verification

Integration tests cover all nine AC1009-AC1032 versions in ASCII and Binary,
all six fields, schema/source/raw provenance, signed `i16` boundaries,
scientific notation, horizontal padding, leading-dot decimals, signed zero,
floating overflow and underflow,
an 8,193-byte profile-valid value, exact Binary NaN payload bits, absence,
wrong group code, missing value, multiple value groups, duplicate variables,
empty/malformed/overflow ASCII values, and pre-cancellation.

Git records 627 added production lines including the public module
registration, 127 lines above the approximate 200-500 target. Keeping the
common bounded reader, both physical decoders, structural state machine, and
the six-field view in one checkpoint avoids publishing a temporary decoder
API that would immediately be replaced. The implementation remains one
cohesive module with a separate 452-line integration-test artifact.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `d709b6f06bdc29fff1c9985a01d649f5dc000c1070acdc3ecbab9e4f16700dc9` |
| `crates/seacad-dxf-core/src/lib.rs` | `155446219df8be55b8aaf3171a8cc65a68fa40ab18a309476dcc049023454ce4` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `21508f6413d6f9aeb029b1cad84515593807579489a3a30d1a7feafb1c92379c` |
