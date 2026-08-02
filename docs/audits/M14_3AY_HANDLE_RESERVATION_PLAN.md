# M14.3ay Handle Reservation Plan

## Scope

M14.3ay pairs a bounded handle range for not-yet-inserted records with the
exact successor `$HANDSEED` transaction and proves composition with M14.3ax
entity placements.

## Evidence boundary

This checkpoint adds no new wire fact. It reuses the Autodesk-backed M11.2a
meaning of `$HANDSEED` as the next available object handle, the fail-closed
identity inventory, M11 transaction/inverse rules, and M14.3ax placement
anchors. The user-authorized legacy trees under
`D:\SeaCad\tham khảo\New folder` were searched read-only for a future-record
handle reservation primitive; none was found. No external or legacy code,
fixture, data, or dependency was copied, translated, vendored, or linked.

## Contract

- `plan_handle_reservation` requires a policy and raw document with the same
  exact `DxfSourceId`.
- The selected resource profile bounds requested handle count before a plan can
  escape.
- Policy-unavailable and `u64` exhaustion outcomes remain typed and retain the
  exact M11.2a state/seed/request evidence.
- A planned range begins at the current `$HANDSEED`, is consecutive and
  constant-space, and retains its successor seed.
- A nonempty reservation replaces only the exact `$HANDSEED` value payload
  with canonical uppercase hexadecimal. ASCII/Binary delimiters and every
  other byte remain untouched.
- A zero-count reservation retains an allocation proposal and emits an empty
  source-bound transaction.
- M11.2b existing-record assignment and future-record reservation share one
  internal uppercase hexadecimal encoder.
- Construction checks cancellation before policy use, before mutation planning,
  and when the transaction builder finishes.
- ASCII, Binary, and format-neutral document views expose the same API.
- “Reservation” is optimistic source-bound planning. It is not a process-wide,
  cross-session, or distributed lock; source verification prevents a stale
  plan from being applied silently.

## Integration proof

For every AC1009-through-AC1032 ASCII/Binary pair, a range of three handles
advances `$HANDSEED`, strict-reparses, and inverses to the byte-identical source.
A second matrix reserves one handle, composes its seed transaction with the
M14.3ax `ENTITIES` placement, inserts a minimal identified POINT record,
strict-reparses the unique identity and successor seed, and restores the exact
source through the composed inverse.

## Nonclaims

This checkpoint does not encode a typed entity draft, select or validate owner
group 330, hold a concurrent lock, write a destination by itself, verify family
semantics, remap graph references, or implement insert/clone/delete closed sets.

## Verification

Focused tests cover all 18 dialect/format pairs, range order, zero count,
unavailable policy, exhaustion, source mismatch, resource exhaustion,
cancellation, public Send/Sync bounds, strict reparse, placement composition,
unique inserted identity, successor seed, and exact inverse. Full gate results
and artifact hashes are recorded below. The focused suite passed 4/4 tests and
the full workspace passed 886/886 tests. Generated schema and release-evidence
checks, `cargo deny --locked check`, formatting, workspace Clippy with warnings
denied, workspace tests, forbidden-production-macro scan, and
`git diff --check` all passed. Production adds 184 lines and removes 21 for a
net 163 lines: 156 in the reservation module, 22 shared-encoder lines, two
module/export lines, and a net 17-line reduction in the existing assignment
planner. This is below the usual 200-line target because the checkpoint reuses
the reviewed allocation policy and transaction builder. No production
dependency changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 252 | `424d279ac49e97261a2cfd799ee88f4fe74c1694fb2a43bd76d9da8b980c71a2` |
| `crates/seacad-dxf-core/src/handle.rs` | 223 | `c7e5dce1b22fb0215f1181ae1808f48c6ffb1ae49f716fd84640a6481d98bd68` |
| `crates/seacad-dxf-core/src/handle_assignment_plan.rs` | 421 | `5dc3c82a9c34f41781fb9c6a243315b3e80e13c1c75497e4fc14859890c7beb7` |
| `crates/seacad-dxf-core/src/handle_reservation_plan.rs` | 156 | `3c31123d42acacfdf0398dfe4995ca0d55d89d88bf8c337e8d102b563329c0ed` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,045 | `77871f29eb53a0b6d5118bf63352cd442ad2d08c02476077ce3625ac27006e62` |
| `crates/seacad-dxf-core/tests/handle_reservation_plan_tests.rs` | 382 | `8e8f89bf13e015f4deddbb040447684773185e72794a3a54110e915869135538` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 886 | `3f7fd74f93b05ba7ca0ec5ce6e9726927254b458af38d2f74663b84dea424d42` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,313 | `f4d3bbe9ed76e8e8bea30d2904d8779fa8c7025800469f8f05e20d9c0df480bd` |
| `docs/SUPPORT_MATRIX.md` | 1,968 | `39539fc3446331dfb5be0537051883ff001782fde4c11ad2232cf4ac3dd0674b` |
