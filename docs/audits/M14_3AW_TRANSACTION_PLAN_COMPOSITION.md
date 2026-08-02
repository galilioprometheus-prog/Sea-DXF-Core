# M14.3aw Transaction Plan Composition

## Scope

M14.3aw adds one atomic composition boundary for independently prepared
source-bound transactions and lets a verifiable entity edit retain its semantic
postconditions while absorbing supplemental handle, owner, or placement work.

## Evidence boundary

This checkpoint adds no new DXF wire or semantic fact. It composes already
reviewed contracts:

- M11.1a immutable transaction plans and conflict/resource enforcement;
- M11.1b exact post-image comparison and executable inverse journals;
- M11.2a Autodesk-backed `$HANDSEED` allocation policy; and
- M11.2b Autodesk-backed record identity insertion and successor seed update.

The two user-authorized legacy snapshots under
`D:\SeaCad\tham khảo\New folder` and the current repository were searched
read-only for an existing transaction composer. None was present. No external
or legacy code, fixture, data, or dependency was copied, translated, vendored,
or linked.

## Contract

- `compose_transaction_plans` accepts borrowed immutable plans and rebuilds
  them against one exact `DxfRawDocumentView`.
- Every input must match source identity, length, and physical format before
  any composed result can escape.
- Input-plan count is bounded by the selected profile. Every patch is replayed
  through the M11 builder, reapplying patch-count, replacement-value, projected
  source-size, span-bound, source-read, and cancellation limits.
- Cross-plan overlapping replacements, insertions inside replacements, and
  duplicate insertion offsets retain the existing typed conflict error.
- Disjoint patches are emitted in raw source order independently of caller plan
  order. Empty input produces one empty source-bound plan.
- The builder freshly captures inverse bytes from the unchanged source rather
  than trusting or concatenating input inverse buffers.
- ASCII and Binary document wrappers expose the same operation.
- `DxfEntityEditPlan::compose_supplemental_transactions` consumes the entity
  plan, composes its transaction with supplemental raw plans, and retains the
  original field semantic expectations for later strict verification/write.

## Integration proof

For every AC1009-through-AC1032 dialect in ASCII and Binary, one source entity
without identity receives the M11.2b allocated group-5 handle, HEADER
`$HANDSEED` advances, and an M14 common indexed-color reset deletes group 62 in
the same composed transaction. Strict reparse proves the assigned handle,
successor seed, reviewed BYLAYER default 256, entity semantic expectation, raw
transaction bytes, and byte-identical executable inverse.

## Nonclaims

This checkpoint does not reserve handles across sessions, encode a new entity
draft, choose an entity insertion anchor, assign or validate owner group 330,
remap references, or implement insert/clone/delete closed sets. Supplemental
raw plans receive byte verification; family-specific semantic expectations must
still be added by the operation that creates them.

## Verification

Focused tests cover all 18 dialect/format pairs, caller-order independence,
empty composition, boundary insertion, overlap, duplicate insertion, source
mismatch, cancellation, plan-count and value-size limits, debug redaction,
M11 handle-assignment integration, strict reparse, retained M14 semantic
verification, and exact inverse restoration. The focused suite passed 5/5
tests and the full workspace passed 878/878 tests.

Generated schema and release-evidence checks, `cargo deny --locked check`,
formatting, workspace Clippy with warnings denied, workspace tests, and
`git diff --check` passed. Production adds 114 lines: 85 in the composition
module, 28 in entity-edit verification, and one module declaration. This is
below the usual 200-line target because the checkpoint deliberately reuses the
reviewed M11 builder and verifier rather than duplicating their conflict,
resource, or inverse logic. No production dependency changed.

Artifact hashes are recorded below after the final gate. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 240 | `acdfee22a7a7e1c137bf83172505d2528b35eb2d8cdcc06c948ddd4ca29b71d0` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 641 | `413f3fe83e8cfd841e42fa5877f9608aa2e49e73b30d5892a1b3a5ec654f0500` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,038 | `dc9a4c90050b043491f4db95346cd1c26ede40241b6913f806a621308e0f2513` |
| `crates/seacad-dxf-core/src/transaction_plan_composition.rs` | 85 | `0e8be7fcf8cae092d631c5369c0fa43debd4643dadb4b792d190ff341a23cd0d` |
| `crates/seacad-dxf-core/tests/transaction_plan_composition_tests.rs` | 501 | `9d46db90b0f8a275f480cd26e203091f6f6e53b1266de1320108c8e738706598` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 856 | `1e315b06565ea1e591206f13aad30a0f1a0e2c24bdaa23d4872c398f8c80da92` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,287 | `8c0a40e6065e9c8b2a4fda3055d09b30aea65d8aec4c87fc7de05f8d70f0d015` |
| `docs/SUPPORT_MATRIX.md` | 1,945 | `9d6ba0f29162dc342c1fa0932648856008e340258b99d8c7c03776f38b9e4336` |
