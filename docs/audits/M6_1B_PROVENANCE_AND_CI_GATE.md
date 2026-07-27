# M6.1b provenance registry and CI gate

M6.1a embedded one normative source inside its single HEADER family. That was
adequate for a bootstrap but would let every later family redefine the same
source independently. M6.1b makes provenance a shared ordered registry.

`schema/dxf/v1/sources.json` is now authoritative for source identity, Autodesk
topic ID, normalized-facts SHA-256, and evidence-anchor kind. Family fields hold
only a stable `source_id` plus their exact evidence anchor. Generation rejects:

- an absent, malformed, duplicate, or out-of-order source ID;
- a malformed Autodesk topic ID or duplicate topic/SHA-256 receipt pair;
- a malformed normalized-facts SHA-256;
- an unknown field source reference;
- evidence that does not match the registered evidence kind;
- an unknown JSON key or a manifest path outside one lowercase JSON filename.

The generated internal registry carries each source ID, topic ID, and source
facts SHA-256. Its normalized input receipt includes the manifest, source
registry, and every family in explicit manifest order.

CI runs `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check` on Windows
x64, Linux x64, and macOS ARM64 after formatting and before lint/test. Check
mode is read-only and fails with stable `SCHEMA_OUTPUT_DIFF` when committed Rust
differs; `--locked` prevents dependency resolution from changing the lockfile.

This milestone changes no core runtime dependency and makes no new DXF semantic
support claim.
