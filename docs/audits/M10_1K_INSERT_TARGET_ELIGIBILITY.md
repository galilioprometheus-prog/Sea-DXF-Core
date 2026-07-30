# M10.1k INSERT Target Eligibility

Retrieved: 2026-07-30

## Policy boundary

M10.1k treats an INSERT target as expansion-eligible only when M10.1j resolves
it uniquely, the target definition is `Closed`, and traversing uniquely
resolved closed BLOCK-member targets cannot reach a cycle. Raw definitions and
resolution evidence remain authoritative.

This is a SeaCad fail-closed expansion policy built on Autodesk's documented
[INSERT block-name
field](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-28FA4CFB-9D5E-4880-9F11-36C97578252F.htm);
it is not an Autodesk claim about file acceptance.

## Implementation contract

`crates/seacad-dxf-core/src/insert_target_eligibility.rs` publishes:

- not-uniquely-resolved, target-not-closed, recursive, or eligible state for
  every M10.1j INSERT resolution;
- one graph edge for each uniquely resolved INSERT member inside a BLOCK;
- owner-local edge lookup retaining exact INSERT and target evidence.

Iterative three-color traversal detects direct self-reference, indirect cycles,
and cycles reachable deeper below a target. Only closed target definitions are
traversed. Cancellation is checked during graph construction and traversal,
and traversal memory is bounded by indexed definition count.

## Test evidence

`crates/seacad-dxf-core/tests/insert_target_eligibility_tests.rs` covers
ASCII/Binary parity across all nine supported dialects, eligible closed
targets, unclosed and interrupted targets, direct self-reference, indirect and
deep reachable cycles, acyclic nested chains, ambiguous/missing resolution,
graph ranges, bounded lookups, cancellation, source identity, and public
traits.

## Non-claims

M10.1k does not validate numeric/count domains, follow ATTRIB/SEQEND, calculate
OCS axes or a transform matrix, expand geometry, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 470 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_target_eligibility.rs` | 391 | `601997bbc6de8871b9d9872e18865cfe8709437f2980dab7fb03fbf7eacca167` |
| `crates/seacad-dxf-core/src/lib.rs` | 480 | `aaff87fc121f81350236663b91200a1649d83e6ae622f8d1705c557cb217b8cf` |
| `crates/seacad-dxf-core/tests/insert_target_eligibility_tests.rs` | 253 | `4e3e5c963c65458f8855e7d990dbd049b04687b6b43bc8b20870b71ceaf7458c` |
| `docs/IMPLEMENTATION_PLAN.md` | 833 | `6c741d9f410423735afaf90fda0fa25ddd00fb0b004db1f277ead96c069b0806` |
| `docs/SUPPORT_MATRIX.md` | 710 | `2c4bad4de76564af43bedf9b055bb8707d0d63c2e3d95592af068cc7cd7c4060` |
