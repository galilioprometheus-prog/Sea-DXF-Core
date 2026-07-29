# M8.1a Basic-Geometry Evidence Receipt

## Scope

M8.1a introduces a format-neutral, source-ordered evidence directory for exact
uppercase `POINT` and `LINE` records found in complete `BLOCKS` or `ENTITIES`
sections. Each matching record retains its raw record boundary and a compact
slice of coordinate-component occurrences. Each occurrence retains its raw
group, documented role, and either exact IEEE-754 binary64 bits or a typed
ASCII lexical failure.

The shared raw-double decoder is also used by existing typed HEADER double
semantics, so geometry and HEADER projections cannot drift between separate
ASCII/Binary decoding implementations.

## Evidence surface

- `crates/seacad-dxf-core/src/basic_geometry.rs` owns exact record matching,
  coordinate-role classification, source-order slices, cancellation, and the
  public borrowed-document adapters.
- `crates/seacad-dxf-core/src/raw_double.rs` owns bounded format-neutral double
  decoding and retains signed-zero bits.
- `crates/seacad-dxf-core/src/header_numeric.rs` now delegates its double
  decoding to that shared primitive without changing HEADER states.
- `crates/seacad-dxf-core/tests/basic_geometry_tests.rs` covers ASCII/Binary
  parity, source order, duplicate components, exact case, section filtering,
  invalid and underflowing ASCII numbers, empty component slices,
  cancellation, lookups, and public trait bounds.
- `docs/audits/M8_1A_AUTODESK_BASIC_GEOMETRY_REFERENCE.md` records the
  reviewed normative boundary and source URLs.

## Non-claims

M8.1a is evidence, not a complete geometry model. It does not:

- select one value from duplicate components or declare completeness;
- apply extrusion defaults or any WCS/OCS/UCS/DCS transformation;
- validate subclass markers, record-version legality, layer or ownership;
- expose thickness, POINT display angle, CIRCLE, ARC, or later entity families;
- edit, canonicalize, write, tessellate, render, snap, or perform topology.

## Verification

The following gates passed at the final dirty-diff state:

- `cargo deny --locked check`: advisories, bans, licenses, and sources all OK.
- `cargo +1.97.1 fmt --all -- --check`: exit 0.
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: exit 0.
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`:
  exit 0.
- `cargo +1.97.1 test --locked --workspace`: 300 passed, 0 failed.
- `cargo +1.97.1 test --locked -p seacad-dxf-core --test basic_geometry_tests`:
  4 passed, 0 failed; the parity case covers all nine AC1009-AC1032 dialects
  in both ASCII and Binary encodings.
- `git diff --check`: exit 0.

## SHA-256 artifact receipt

| Artifact | SHA-256 |
|---|---|
| `crates/seacad-dxf-core/src/basic_geometry.rs` | `4EFEC9E071E56E7256F4C8C4E8359D52AEF90567BA576DFC4332D1E29B1FD384` |
| `crates/seacad-dxf-core/src/raw_double.rs` | `5C44096495D0F103591F499F163ADC4A935D8C12AEFDB76A28DFE340EB8A1DB2` |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `6CCC0745AF05AEBDFA31E6B491F5FF9425D335FF5FB62101821849C59C44936D` |
| `crates/seacad-dxf-core/src/lib.rs` | `757305CEEEC90A7FD7EE990265AC4EF5CA9959A357A829F730116DC12F6541A9` |
| `crates/seacad-dxf-core/tests/basic_geometry_tests.rs` | `718A4EA3158A7FC134190233A825F020EAA94CC425382E3DE2D21376F3836FBF` |
| `docs/IMPLEMENTATION_PLAN.md` | `3AC8878B2ACC2C0656A1B34E58BBD8BFF524FCD03C7481790359033F58FF5E01` |
| `docs/SUPPORT_MATRIX.md` | `2DFCED40368F5CD4F305D325869A775C37D5E391652C9AC5F17EE04B331DEB81` |
| `docs/audits/M8_1A_AUTODESK_BASIC_GEOMETRY_REFERENCE.md` | `F1731BA25280A6E053AF13D689454C8617353FC8775A77429B52391F149DD1E8` |

The receipt itself is intentionally excluded from its own hash table. Its final
hash is reported in the checkpoint handoff before any commit or tag decision.
