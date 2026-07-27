# DXF Core 1.0 Implementation Plan

Status: M5 in progress through M5.1a Binary DXF group-code decoder and
value-family wire registry; awaiting user approval for M5.1b streaming cursor

1. M0: toolchain, clean private repository, workspace, policy, and CI.
2. M1: provenance audit of earlier tests, fixtures, documents, and code.
3. M2: bounded file/byte source, progress/cancel, diagnostics, and SHA-256.
4. M3: lossless ASCII framing, strict/compatible modes, verbatim writer, and
   initial `inspect`/`verify` CLI.
5. M4: AC1009-AC1032 dialects, encoding, sections, and group-0 indexes.
6. M5: lossless Binary DXF record stream and malformed-input bounds.
   M5.1a freezes explicit pre-R13/R13+ group-code decoding and the documented
   value-family registry. M5.1b adds bounded streaming value framing.
7. M6: provenance-backed schema/codegen and lazy document semantics.
8. M7: handles, ownership, references, dictionaries, XDATA, and reactors.
9. M8: exact basic geometry and coordinate-system preservation.
10. M9: polyline, mesh, spline, and helix families.
11. M10: blocks, text, hatch, dimensions, leaders, layouts, underlays, and
    exact-opaque ACIS/proxy/custom payloads.
12. M11: immutable atomic transactions, inverse journals, and handle policy.
13. M12: preserve-patch and canonical ASCII/Binary writers with reparse.
14. M13: evidence closure, 1,000-file/10-GB corpus gates, six native receipts,
    SBOM/notices, and DXF Core 1.0 release.

Every item is split into reviewable micro-milestones and stops after its own
passing checkpoint.
