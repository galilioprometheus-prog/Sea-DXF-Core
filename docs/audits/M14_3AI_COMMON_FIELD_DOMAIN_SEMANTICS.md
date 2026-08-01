# M14.3ai Common Field Domain Semantics

## Scope

M14.3ai projects the eight reviewed M14.3ah common-field scalar domains from
already-open ASCII and Binary raw documents. It composes the existing generic
common-field semantic directory, preserving source identity, exact evidence,
cardinality, and the shared explicit/defaulted/absent/invalid state model.

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

The user-authorized legacy trees remained read-only. No external or legacy
code, fixture, data, or dependency was copied, translated, vendored, linked,
or added at runtime.

## Contract

- The domain directory owns the generic semantic directory it projects and has
  one ordinal-aligned entry for every common field of every indexed entity.
- The eight reviewed scalar fields reuse the edit-domain classifier. Usable
  explicit and defaulted values become typed domain values without changing
  their state or raw provenance.
- Scalar decode errors, duplicate singletons, missing required fields, wrong
  value kinds, and unsupported domain values remain typed and distinguishable.
- All other common fields are explicitly `Unreviewed` and retain their exact
  prior singleton or opaque-sequence semantics.
- Directory construction and lookup are source-bound, cancellation-aware,
  allocation-fallible, payload-redacted, and independent of group order.

## Nonclaims

M14.3ai does not validate text or symbol names; resolve handles/references;
interpret transparency; reconcile proxy size with group-310 chunks; validate
cross-field relations or version applicability; add family patches; insert
entities; allocate handles/owners; clone/delete closed sets; or advance an
entity topic to `Complete`.

## Verification

Paired ASCII/Binary fixtures cover all nine dialects, all eight valid reviewed
domains, out-of-order and unknown groups, every reviewed out-of-domain scalar,
invalid numeric syntax, duplicate singleton, missing required fields, defaults,
absence, unreviewed text and opaque sequences, source identity, cancellation,
lookup, debug redaction, and bounded public types. Final workspace counts,
gate results, production diff, and artifact hashes are recorded below. The
focused domain-semantic suite passed 4/4 tests, its classifier regression passed
4/4, and the full workspace passed 838/838 tests. Schema generation and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, workspace tests, and `git diff --check` all passed. The
production diff is 294 added lines: 277 in the semantic projection module, 11
in the shared domain module, and 6 exports. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 179 | `280ebb4822d329a730a704767f203665d1563baa61941a0d56483eefc25fb0e7` |
| `crates/seacad-dxf-core/src/entity_common_field_domain.rs` | 379 | `728c22f59f6e7cb14190f81d93c30fb5bcb2ef59d566e5dabfdd82f8ddd593c3` |
| `crates/seacad-dxf-core/src/entity_common_field_domain_semantic.rs` | 277 | `9a655cee874cb21447fb788b3bb7dc3363ea64a6b8e9060d74cccc633e159f30` |
| `crates/seacad-dxf-core/src/lib.rs` | 985 | `b62ae42009ab72b2d22827487ddda092ba4f8903cfe9e7587aca5ef40e1058f8` |
| `crates/seacad-dxf-core/tests/entity_common_field_domain_semantic_tests.rs` | 456 | `d29c1407ada26ade5c38a1e10b92fcc2435ee9c77e4305ee4636b7c7f062eea1` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 682 | `2a0a01111c14eaa57295bc3782eaac79d8dee454b85322f3014385a7652df0d2` |
| `docs/IMPLEMENTATION_PLAN.md` | 2109 | `ee829c6819b62c6104e34755cb584c59a7741eeae25c40c9585b1423fa7c901f` |
| `docs/SUPPORT_MATRIX.md` | 1767 | `eba5d5e0fd06de5cd5d1af8b503209ec4b1054e666bce703e16c128bfecee9fb` |
