# M14.2d MTEXT/TOLERANCE Scalar Semantics

Retrieved: 2026-07-31

## Normative boundary

Autodesk's [MTEXT
reference](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm)
defines insertion, nominal height, reference width, attachment, drawing
direction, extrusion, x-axis/rotation input, read-only extents, line spacing,
background, and column fields. It explicitly gives extrusion default
`(0, 0, 1)`, makes x-axis and group 50 alternative rotation inputs with
last-input precedence, and reuses group 50 for column heights.

Autodesk's [TOLERANCE
reference](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-ADFCED35-B312-4996-B4C1-61C53757B3FD.htm)
defines required insertion and x-axis vectors and the same optional extrusion
default.

M14.2d selects only scalars whose roles are unambiguous before structural or
subclass ownership. It does not invent defaults for fields where Autodesk does
not publish one.

## Legacy evidence boundary

`D:\SeaCad\cad_2026-07-23_source` was inspected only as a read-only behavioral
oracle. Its Q4.2 notes and semantic diagnostics identify partial vectors,
missing core layout values, invalid/duplicate numbers, read-only extents,
line-spacing bounds, columns, group-50 ambiguity, and background/common-color
collisions as compatibility risks. No parser, semantic implementation,
validator, test fixture, dependency, or source text was copied, translated,
vendored, or linked. Autodesk documentation remains normative.

## Implementation contract

`DxfMTextToleranceScalarDirectory` retains the M14.2b card/evidence chain and
constructs lazy public semantics for exact MTEXT and TOLERANCE records.
Required, defaulted, absent, explicit, invalid ASCII, and multiple-value states
use the shared M14.2c field/raw provenance contract.

MTEXT exposes required insertion, nominal height, reference width, attachment,
and drawing direction; defaulted extrusion; and independently optional x-axis
components, actual extents, line spacing, background fill/index/transparency,
fill-box scale, and column controls. TOLERANCE exposes required insertion and
x-axis vectors plus defaulted extrusion.

MTEXT `RotationOrColumnHeight` and
`BackgroundRgbOrEntityTrueColor`/`BackgroundNameOrEntityColorName` cards remain
reachable as raw evidence but have no selected M14.2d semantic field. This
prevents accidental last-wins or common-property guesses.

## Test evidence

`mtext_tolerance_scalar_tests.rs` covers all nine AC1009--AC1032 dialects in
ASCII and Binary; pre-R13 one-byte group-code limits; bit-exact scalar/default
parity; required, optional, read-only, signed-16, signed-32, background, and
column fields; partial optional vectors; missing required values; invalid ASCII
numbers; duplicates; first raw diagnostic provenance; family scope; explicit
group-50 and group-420 evidence-only boundaries; source identity; lookup
bounds; cancellation; and public `Copy`/`Send`/`Sync` bounds.

## Non-claims

M14.2d does not choose group-50 rotation versus column heights; apply
rotation/x-axis last-input precedence; assign group 420/430 to MTEXT rather
than common entity properties; validate attachment/direction/spacing/background
or column values; enforce read-only behavior during writing; order/merge text
chunks; decode strings; transform coordinates; resolve styles; derive glyphs;
edit; or write.

## Required checkpoint gates

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 630 workspace tests with zero failures/ignored tests, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `4eef2ab758039c70206b5ede5fe76ea3def5cb270b6e3d563b22e7a15d74cb4b` |
| `crates/seacad-dxf-core/src/lib.rs` | 672 | `5cd46fbc1cc89aab71f61d1fb9f524d09aaf7241d4efebcbc084a8d8eea82d11` |
| `crates/seacad-dxf-core/src/mtext_tolerance_scalar.rs` | 314 | `b483c9991e7343d0bd1a5f939dff6bd5f1d6611c0fc0a53e955a3b57c3e7ce05` |
| `crates/seacad-dxf-core/src/mtext_tolerance_scalar_build.rs` | 333 | `c430420930c2666c29b319630f8548565ed7424d1e6eadabeedd75a44f78e1fd` |
| `crates/seacad-dxf-core/src/text_shape_scalar.rs` | 300 | `df313f4396788452b7f18190e6e8ca2a5dbdc392fb7500a9f615130652526e9f` |
| `crates/seacad-dxf-core/src/text_shape_scalar_build.rs` | 302 | `6e1d46f4c545e916c60d500b5647e950a2054fb88c12cc1dc3571b24748d005c` |
| `crates/seacad-dxf-core/src/text_symbol_scalar_value.rs` | 174 | `0b7e49463204192fccbf24796d81cf7ca7391ec571baaa9093f261dd9194efc8` |
| `crates/seacad-dxf-core/tests/mtext_tolerance_scalar_tests.rs` | 515 | `d36762e6e7c8cfddc5664aaa7abb7f6f8cc9cbf19c63358e0d1eb9808e1bb9fa` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 120 | `cef3e52499aed75875c3cf0d0f7e18dbfbb4d5df1b2f28bd39766ab2241ac9b2` |
| `docs/IMPLEMENTATION_PLAN.md` | 1428 | `4e096583005f56065b8b63b688a6333372d2ceef71118e7c89e4ac03ce346c3f` |
| `docs/SUPPORT_MATRIX.md` | 1141 | `67e6a7c1af99df8034f0403ab6df91d05596d4fddf4cddaeff3c8eba4169bb7c` |
