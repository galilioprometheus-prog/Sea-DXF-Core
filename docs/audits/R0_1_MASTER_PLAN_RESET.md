# R0.1 Master Plan Reset

Date: 2026-08-08 (`Asia/Saigon`)

## Decision

R0.1 replaces the former DXF-only master entrypoint with a product-level SeaCad
master plan while preserving DXF Core 1.0 as the only active implementation
subplan. This checkpoint changes documentation ownership and navigation only.
It does not add a crate, dependency, runtime behavior, file-format capability,
license grant, or support claim.

## Baseline

- Baseline `HEAD`: `76883bbdda527a828ee01136835714bfb3b4c38e`.
- Documented completed code checkpoint: M14.3dc.
- Concurrent code work outside R0.1 is limited to
  `crates/seacad-dxf-core/src/lib.rs`,
  `crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs`, and untracked
  `crates/seacad-dxf-core/src/named_symbol_destination.rs`.
- R0.1 does not stage, remove, rename, or otherwise claim ownership of those
  three paths.

## Preserved plan mapping

| Former path | Preserved path | Pre-move SHA-256 | Disposition |
| --- | --- | --- | --- |
| `docs/IMPLEMENTATION_PLAN.md` | `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | `a3b180c50812c8ec8b1d605a819c89ab752a30dea6053ada96c2389344cb5a52` | Complete DXF plan preserved; one internal entity-subplan path is made relative to its new directory |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | `c3c622870f0176a3138beba293f7013cdcd41279451b0a2942ec78ff22ca1584` | Byte-identical relocation |

Historical audit receipts retain their original path and hash observations.
They are evidence about their checkpoint-era worktrees and must not be rewritten
to look current.

## Documentation changes

- `docs/IMPLEMENTATION_PLAN.md` becomes the stable master-plan entrypoint.
- Root agent instructions require both the master roadmap and the stricter
  active DXF subplan.
- English and Vietnamese README navigation points at the master and preserved
  DXF plans.
- English is locked as the canonical fallback, Vietnamese as a first-class
  locale, and later languages as normalized BCP 47 catalogs shared by GUI, CLI
  help, theme metadata, declarative plugin UI, and human-readable MCP text.
- `docs/ARCHITECTURE.md` distinguishes current DXF runtime boundaries from the
  planned multi-format product architecture.
- The planned `.seacad-theme` package receives a non-executable token contract,
  but no theme loader or GUI dependency is introduced.
- The superseded M14.3dc mechanical batch is replaced because its clean-tree and
  exact old-plan path/hash preconditions can no longer describe this worktree.

## Support and license boundary

- `docs/SUPPORT_MATRIX.md` remains unchanged and normative.
- DXF Core 1.0 remains incomplete; DWG, DGN, rendering, constraint solving, and
  native B-rep remain unimplemented.
- The proposed `MIT OR Apache-2.0` format-core license remains a future R0 gate.
  The repository's current proprietary license is unchanged and authoritative.
- No vendor SDK, oracle execution, external parser, fixture, corpus file, or
  third-party source enters the repository.

## Verification contract

R0.1 is acceptable only when all of the following are true:

1. Both preserved subplans exist, are nonempty, and retain their expected title.
2. The entity subplan hash equals its pre-move hash.
3. Live README, agent, architecture, master-plan, and active-subplan links resolve.
4. Agent, architecture, and master-plan text agree on English fallback,
   Vietnamese parity, locale-neutral protocol identifiers, and the ban on
   automatic translation of drawing or script content.
5. Historical audit receipts are unchanged.
6. No support-matrix, Rust, manifest, lockfile, schema, fixture, or release
   artifact changes belong to R0.1.
7. The three concurrent code-work paths are unchanged during mechanical R0.1
   verification.
8. Markdown whitespace and repository diff checks pass.
9. Required repository quality gates pass before any commit or checkpoint tag.

## Expected implementation effect

- Production concepts removed: none.
- Production lines changed: zero.
- Dependencies added, removed, or updated: zero.
- Serialized formats, public APIs, error codes, and support claims changed:
  none.
