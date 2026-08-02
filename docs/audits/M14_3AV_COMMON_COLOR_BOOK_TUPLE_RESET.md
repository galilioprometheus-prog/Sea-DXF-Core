# M14.3av Common Color-Book Tuple Reset

## Scope

M14.3av adds one typed composite reset that removes every explicit common
entity group 62, 420, and 430 as a single logical session request with rollback
on partial planning failure.

## Sources

- The frozen `autodesk.common_entity_codes.2024` registry receipt defines group
  62 as optional indexed color with default 256, and groups 420/430 as optional
  true color and color name without generated defaults. Its normative source is
  Autodesk's common entity-code table:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- M14.3au established from Autodesk `acad_truecolordlg` that a color-book
  selection is represented by the related 62/420/430 tuple:
  `https://help.autodesk.com/cloudhelp/2018/DEU/AutoCAD-AutoLISP-Reference/files/GUID-E6FF435F-9E66-4F37-8770-2E3FB87B8E0B.htm`

The Autodesk pages could not be reopened during this checkpoint because the
web connector returned a revoked-token error. No support fact was added from
that failed lookup; implementation uses only the already reviewed registry and
M14.3au receipts. The two user-authorized legacy snapshots under
`D:\SeaCad\tham khảo\New folder` were inspected read-only. They independently
decode MTEXT background groups 420/430 and contain no reusable common tuple
reset contract. No external or legacy code, fixture, data, or dependency was
copied, translated, vendored, or linked.

## Contract

- `DxfEntityPatch::ResetCommonColorBook` preflights pending conflicts for all
  three fields before planning any deletion.
- A unique explicit member is deleted through the reviewed singleton reset
  primitive. An absent optional member is already implicit and queues nothing.
- If any later member is duplicate, structurally unavailable, cancelled, or
  exceeds a session resource bound, every member appended by the request is
  truncated. Earlier unrelated pending edits remain intact.
- A wholly implicit tuple returns `AlreadyImplicit`; otherwise the logical
  request returns `Composite` and reports the resulting queue size.
- Strict post-image semantics require color 62 to be `Defaulted(Int16(256))`
  and true color 420 plus color name 430 to be `Absent`.
- Reset only deletes complete source groups, so it requires no new group-code
  encoding and applies consistently to ASCII and Binary AC1009 through AC1032.
- Every accepted deletion participates in generic semantic/raw verification
  and the executable inverse journal restores the byte-identical source.

## Nonclaims

This checkpoint does not read `.acb` files, verify external book/name mappings,
define effective rendered color, review broader field applicability, implement
entity-family patches, insert/clone/delete entity graphs, or advance an entity
to `Complete`.

## Verification

Paired ASCII/Binary fixtures cover all nine dialects, complete tuples, missing
name, missing true color, wholly implicit modern tuples, AC1009's expressible
indexed-color member, duplicate-source rollback after an earlier component,
pending-field conflicts, strict default/absence semantics, generic edit
verification, and byte-identical inverse restoration. The focused suite passed
5/5 tests and the full workspace passed 873/873 tests.

Generated schema and release-evidence checks, `cargo deny --locked check`,
formatting, workspace Clippy with warnings denied, workspace tests, and
`git diff --check` passed. The production delta is 50 added and 1 removed line
in the edit session. This is below the usual 200-line micro-milestone target
because the operation composes the existing reviewed reset, transaction,
verification, and inverse primitives without adding a second kernel. No
production dependency changed.

Artifact hashes are recorded below after the final gate. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 234 | `d5e0c655adfb466a1f79fdec117bf79a575fecfb5637d16744a3b9a066da6bf0` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 863 | `1e3fd906ccaad380985da4227a935ede6be405a05e02b19848f01857f053fa82` |
| `crates/seacad-dxf-core/tests/entity_common_color_book_edit_tests.rs` | 714 | `b519537257a71b8312625ae5d17805e627660f50dd6ee9c1a3937d5aecc8270a` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 843 | `26d44163555d0a18dbcf2eea2611c9d8ac373f02968740a759af1ccc8cd2e146` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,273 | `130e771b13a88bd33e9ba21dbbe527cf7c102c27a3c8306e6bce4e4a30dd9293` |
| `docs/SUPPORT_MATRIX.md` | 1,931 | `7ae61a987ebaa81bdfa95fa2e9138de205fdf566ff8810a30498042f9edff540` |
