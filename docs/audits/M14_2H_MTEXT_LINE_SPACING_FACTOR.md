# M14.2h MTEXT Line-Spacing Factor

## Scope

M14.2h projects the optional MTEXT group-44 scalar from M14.2d into Autodesk's
documented inclusive line-spacing-factor domain, `0.25..=4.00`.

The semantic value retains the exact IEEE-754 binary64 payload and field/raw
provenance. Values outside the domain fail typed without normalization.
Absence stays `Absent`; no default is invented.

## Normative basis

Autodesk's DXF MTEXT entity reference defines group 44 as an optional percentage
of default line spacing and states that valid values range from 0.25 to 4.00:
<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>.

## Clean-room oracle boundary

`D:\SeaCad\cad_2026-07-23_source` was inspected read-only as behavioral
evidence. Its historical MTEXT validation note reported one invalid
line-spacing factor in that project's corpus, which made boundary and
out-of-range coverage a high-value compatibility case. No implementation,
parser, type, fixture bytes, test code, dependency, or source text was copied,
translated, linked, vendored, or imported. Autodesk documentation and current
SeaCad scalar provenance remain normative.

## Behavior locked

- All AC1009-AC1032 dialects have ASCII/Binary parity.
- Exact `0.25` and `4.00` boundary values are accepted.
- The adjacent representable values below and above the domain are rejected.
- Zero and all other finite out-of-range values retain their exact payload and
  raw provenance in a typed issue.
- Invalid ASCII and duplicate values retain the underlying typed scalar issue.
- Missing group 44 remains `Absent`.
- Cancellation, family scope, raw-record bounds, source identity, and public
  `Copy`/`Send`/`Sync` contracts are covered.

## Explicit nonclaims

M14.2h does not validate nominal height, reference width, read-only extents,
fill-box scale, column dimensions, or background/column code domains. It does
not resolve group-50 rotation precedence, transform coordinates, decode text,
resolve styles, derive glyph geometry, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 645 workspace tests with zero failures/ignored tests, prohibited
production-macro scanning, and `git diff --check`. No dependency manifest or
lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `fd651ca5694dbb006d866b2453c48e5f537724c608322c4db95deaa5c276a699` |
| `crates/seacad-dxf-core/src/lib.rs` | 696 | `9d9fd20beb90886c53b424e02676c0d0feb8f31b1668cf33011112f9281f7b7a` |
| `crates/seacad-dxf-core/src/mtext_numeric_domain.rs` | 192 | `a6fee167a417d982942f1bba1b1e1979b32532e071ee84df1c9ac9994987d2ea` |
| `crates/seacad-dxf-core/tests/mtext_numeric_domain_tests.rs` | 284 | `dd6431bb6ffb2155c7ab176627f0bb522f0d79d71b6fd247c8ff200f30b33a88` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 143 | `7eb52f3abf569ea6fe24949d9121973e79c9b8a7ef3caa7f7397e8c68691f299` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,464 | `9515423784da0626bded1ee3ec6d5b079acc97be74946e79d54213c30aeb53d5` |
| `docs/SUPPORT_MATRIX.md` | 1,168 | `9a67f0fbc8c2aa0e43f159644fa3436ca62f4ae27ebb1138532d96be94e411a8` |
