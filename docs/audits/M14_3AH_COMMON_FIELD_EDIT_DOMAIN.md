# M14.3ah Common Field Edit Domain

## Scope

M14.3ah adds a pure typed classifier for eight reviewed common-entity scalar
edit domains and invokes it before replacement or insertion planning. Invalid
reviewed values never enter an edit session queue or produce transaction bytes.

## Sources

- Autodesk common entity codes define groups 67, 62, 370, 48, 60, 92, 420,
  and 284, their defaults, enumerated modes, RGB high-byte rule, and proxy byte
  count meaning:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk's public `AcDb::LineWeight` wrapper enumerates -3, -2, -1 and the
  discrete hundredths-of-a-millimeter values through 211; internal-only -4 is
  deliberately rejected:
  `https://help.autodesk.com/cloudhelp/2022/ENU/OARX-ManagedRefGuide/files/OARX-ManagedRefGuide-Autodesk_AutoCAD_DatabaseServices_LineWeight.html`
- Autodesk `AcDbEntity` guidance constrains color indexes to 0 through 256 and
  `setLinetypeScale` requires a nonnegative scale; the common DXF table supplies
  negative layer-off color meaning:
  `https://help.autodesk.com/cloudhelp/2018/ENU/OARXMAC-RefGuide/files/OREFMAC-__MEMBERTYPE_Methods_AcDbEntity.html`

The user-authorized legacy trees under `D:\SeaCad\tham khảo\New folder`
remained read-only. No external or legacy code, fixture, data, or dependency
was copied, translated, vendored, linked, or added at runtime.

## Contract

- Model/paper space and visibility accept only 0 or 1. Shadow mode accepts only
  the documented values 0 through 3.
- Indexed color is a closed typed domain: BYBLOCK, nonzero ACI with an explicit
  layer-off flag, or BYLAYER. Its constructor and `NonZeroU8` representation
  prevent an invalid ACI state from being constructed.
- Lineweight accepts only public Autodesk enumeration values. The typed wrapper
  preserves the exact Int16 wire value and exposes the three inherited modes.
- Linetype scale must be finite and nonnegative. Proxy graphics size must be
  nonnegative but is not reconciled with group-310 chunks in this checkpoint.
- True color accepts 0 through `0x00ff_ffff`, preserves the packed value, and
  projects red, green, and blue bytes.
- Wrong edit-value kinds and reviewed out-of-domain values are typed failures.
  Every other common field returns `Unreviewed`; it is not silently called
  valid and remains subject to the existing wire encoder.
- The session preserves duplicate-target precedence, then applies the domain
  gate before any source planner. A rejection leaves its queue unchanged.

## Nonclaims

M14.3ah does not classify domains in already-open raw documents; validate text
or symbol names; resolve handles/references; interpret transparency; reconcile
proxy size with opaque chunks; validate cross-field relations or version
applicability; add family patches; insert entities; allocate handles/owners;
clone/delete closed sets; or advance an entity topic to `Complete`.

## Verification

Focused boundary tests cover every accepted lineweight value, adjacent invalid
values, indexed-color boundaries and round trips, non-finite/negative scales,
negative proxy sizes, true-color high-byte rejection and RGB channels, all
space/visibility/shadow modes, kind mismatch, and explicit `Unreviewed` state.
Session tests prove invalid values do not queue, duplicate precedence remains
stable, and accepted edits strictly verify in ASCII and Binary for AC1009
through AC1032 with byte-identical inverse restoration. The focused suite passed
4/4 tests, the edit-session regression passed 4/4, and the full workspace passed
834/834 tests. Schema generation and release-evidence checks, `cargo deny
--locked check`, formatting, workspace Clippy with warnings denied, workspace
tests, and `git diff --check` all passed. The production diff is 385 added and
2 removed lines: 368 in the domain module, 10 added and 1 removed in the edit
session, and 7 exports. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 174 | `24ec9ca6047707d9a94d55e53a35efc6e6029c38138161f557c82f9e0fff28b9` |
| `crates/seacad-dxf-core/src/entity_common_field_domain.rs` | 368 | `01d8972a1a7bf8319c0a053caebd6d1ea9cc2c16c8210c2a60b9d5c24ef54722` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 504 | `b6f93863bd82fc3150921678c6fb265e21022f7336077dc670ea8a49f977fdbb` |
| `crates/seacad-dxf-core/src/lib.rs` | 979 | `63801b1ce9c8d453dc5d6cffdbd0a794cc669d94c4d29aa7b04b8df6d9c624ca` |
| `crates/seacad-dxf-core/tests/entity_common_field_domain_tests.rs` | 489 | `ca83489900aa0e576df172519e90153b40bef22251cabdd85b870af7b6ce725c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 671 | `67158b20e54d84cbb4905b3648b4139c039ecb6b9adc6f607fcfdd84c56b9cec` |
| `docs/IMPLEMENTATION_PLAN.md` | 2097 | `5ec2c64d0537201507ce3fab48fe66b8026da19b47afd6226549375a581c5b66` |
| `docs/SUPPORT_MATRIX.md` | 1754 | `92dbcd86d498533c2a7583cf61811e8c21ec371f4655c343c219b86243549429` |
