# DXF Core 1.0 Implementation Plan

Status: M5 completed through M5.2c verified Binary replay and CLI integration;
M6.4a adds a shared exact HEADER variable directory

1. M0: toolchain, clean private repository, workspace, policy, and CI.
2. M1: provenance audit of earlier tests, fixtures, documents, and code.
3. M2: bounded file/byte source, progress/cancel, diagnostics, and SHA-256.
4. M3: lossless ASCII framing, strict/compatible modes, verbatim writer, and
   initial `inspect`/`verify` CLI.
5. M4: AC1009-AC1032 dialects, encoding, sections, and group-0 indexes.
6. M5: lossless Binary DXF record stream and malformed-input bounds.
   M5.1a freezes explicit pre-R13/R13+ group-code decoding and the documented
   value-family registry. M5.1b adds bounded streaming value framing. M5.2a
   adds encoding/dialect agreement and the immutable raw document; M5.2b adds
   envelope/index; M5.2c adds verified unchanged replay and CLI integration.
7. M6: provenance-backed schema/codegen and lazy document semantics. M6.1a
   adds a deterministic internal Rust generator and shape-only HEADER bootstrap
   for `$ACADVER`, `$DWGCODEPAGE`, and `$HANDSEED`. M6.1b moves normative source
   identities into a shared, strictly validated registry and requires generated
   code to match on every CI platform. M6.2 adds compact field/raw provenance
   and distinguishes `Explicit`, `Defaulted`, `Absent`, and `Invalid` without
   conflating states. M6.3a connects the generated schema to the already
   evidence-backed `$ACADVER` report through one O(1) typed HEADER view shared by
   ASCII and Binary. M6.3b runs the same `$DWGCODEPAGE` state machine over both
   physical formats and exposes reviewed/unrecognized declarations separately
   from decoder policy. M6.3c runs one exact `$HANDSEED` state machine in both
   parse loops, preserves source spelling/provenance, and exposes the parsed
   64-bit handle without claiming allocation or topology policy. M6.4a indexes
   every exact group-code 9 marker and its complete value-group range in exact
   HEADER sections, including unknown and multi-value variables, without a
   second source scan or semantic guesses. A 1 GiB evidence gate is required
   before any large-file claim.
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
