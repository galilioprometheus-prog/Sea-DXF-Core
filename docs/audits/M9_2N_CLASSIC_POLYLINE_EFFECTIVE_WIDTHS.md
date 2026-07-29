# M9.2n Classic POLYLINE Effective Widths

Retrieved: 2026-07-30

## Normative boundary

Autodesk [AcDb2dPolyline default start width](https://help.autodesk.com/cloudhelp/2018/ENU/OARXMAC-RefGuide/files/OREFMAC-AcDb2dPolyline__setDefaultStartWidth_double.html)
and [Polyline2d default end width](https://help.autodesk.com/cloudhelp/2022/ENU/OARX-ManagedRefGuide/files/OARX-ManagedRefGuide-Autodesk_AutoCAD_DatabaseServices_Polyline2d_DefaultEndWidth.html)
state that DXFIN uses the parent POLYLINE default only when the VERTEX omits
the corresponding group `40` or `41`. Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
assigns those fields to the segment beginning at that vertex.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_segment_width.rs` resolves effective start
and end widths independently for each M9.2j classic 2D segment. An explicit
VERTEX value, including zero, wins. Only the semantic `Defaulted` state caused
by source absence selects the corresponding parent default. Results retain
`Vertex` or `ParentDefault` origin; invalid vertex/parent components and classic
3D width requests remain typed.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_segment_width_tests.rs` covers
ASCII/Binary parity across all nine supported dialects, parent fallback,
explicit-zero precedence, independent start/end selection and failures, typed
3D rejection, lookup bounds, cancellation, source identity, and public traits.

## Non-claims

M9.2n does not validate width sign/range, construct wide outlines, bevels or
joins, tessellate, diagnose file conformance, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 407 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_segment_width.rs` | 181 | `847caa11c1193630f07b8efe900f4ebc74934d72eba38c0fc8b291121c8a5582` |
| `crates/seacad-dxf-core/src/lib.rs` | 385 | `4aed8a4fe5ca00215ebad81b452d754a036184e8df45cc0745107d9d3860f72a` |
| `crates/seacad-dxf-core/tests/polyline_segment_width_tests.rs` | 212 | `15c49469d2edd3af191df227a2e178b3fd03fba6e6086622460848ca29164b95` |
| `docs/IMPLEMENTATION_PLAN.md` | 646 | `962168c2c15268b4fa2aa8699a02f8f512b7fda06e143be9e1f029b1d64cb71b` |
| `docs/SUPPORT_MATRIX.md` | 546 | `151aeba7c9bbb5064cb620f16acd54bd44c32f9a58fc2bfc902c33ce31c00fc1` |
