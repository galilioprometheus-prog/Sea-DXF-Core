# SeaCad Agent Rules

## Mission

Build SeaCad as independent, reusable CAD format cores and a workflow-driven
CAD product in small, evidence-backed milestones. The current active product
program remains DXF Core 1.0, covering ASCII and Binary DXF AC1009 through
AC1032. Unknown, custom, proxy, and non-public payloads remain exact opaque
data.

## Milestone discipline

- Follow the roadmap order in `docs/IMPLEMENTATION_PLAN.md`.
- While DXF Core 1.0 is active, also follow
  `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` and its linked entity
  completion subplan in order. The master plan never overrides a stricter
  active-subplan contract.
- Stop after every micro-milestone for user approval.
- Use one commit and one annotated checkpoint tag per approved milestone.
- Do not increase a support claim without fixtures, tests, evidence, and an
  artifact SHA-256 receipt.
- Keep production changes near 200-500 lines per micro-milestone. Split work
  before it becomes difficult to review.

## Mechanical delegation

- Mechanical checks may accumulate and be handed to Antigravity CLI as one
  bounded batch instead of interrupting implementation after every small check.
- Use `docs/ANTIGRAVITY_MECHANICAL_BATCH.md` as the single source of truth for
  that handoff. Codex must keep exactly one active batch marked `READY`, update
  its checkpoint, commands, expected evidence, and date as work advances, and
  retire or replace the batch after its returned evidence has been reviewed.
- Codex defines and reviews the batch; the user manually copies one explicit
  English prompt to Antigravity CLI. Do not depend on Agent Hub or another MCP
  coordination layer.
- The chat handoff prompt should only identify the exact repository root, order
  Antigravity to read `AGENTS.md` and the complete active batch note, execute
  the `READY` batch exactly, and return the note-defined report. Do not duplicate
  the full batch in chat unless the note is inaccessible.
- The active batch note, rather than the short chat prompt, must state the exact
  root, ordered commands, allowed writes, prohibited actions, stop conditions,
  acceptance criteria, expected results, and raw evidence format so the worker
  makes no scope decisions.
- Every active batch must specify one batch-specific report file outside the Git
  repository, under `D:\SeaCad\AntigravityReports`. Antigravity must write the
  complete note-defined report there for `PASS`, `FAIL`, or `BLOCKED`; chat
  output alone does not count as delivery.
- After Antigravity writes the report, the user only needs to tell Codex that it
  is ready. Codex independently reads the specified file and checks both its raw
  evidence and repository state before relying on it or authorizing more work.
- Keep architecture, normative-source interpretation, provenance, dependency
  approval, support claims, commit, push, tag, merge, and release decisions
  with Codex and the user.

## Source and provenance

- Treat every earlier workspace outside the current SeaCad repository as
  read-only, regardless of later path moves. Known legacy trees include sibling
  directories matching `D:\SeaCad\cad_*` and the former `D:\Backups` location.
- Do not fork, vendor, copy, translate, or line-by-line port an external parser.
- Official Autodesk documentation remains normative for DXF and applicable DWG
  evidence. Official Bentley documentation is normative for future DGN work.
  External implementations may be isolated behavioral oracles only.
- Move an old test, fixture, or observation into SeaCad only after M1 records
  its ownership, license, provenance, and SHA-256 when applicable.
- Keep private CAD corpus data outside this repository.

## Code safety

- SeaCad production Rust uses `#![forbid(unsafe_code)]`.
- Production code must not use `panic!`, `unwrap()`, `expect()`, `todo!`, or
  `unimplemented!`.
- Treat all CAD input as untrusted and enforce explicit resource limits.
- Preserve raw source bytes; never silently replace invalid or unrepresentable
  text.
- Writers create a new destination and never overwrite the source file.

## Localization

- SeaCad is multilingual. English (`en`) is the canonical fallback and
  Vietnamese (`vi`) is a first-class locale with complete user-facing catalog
  coverage before a feature is released.
- Additional languages use normalized BCP 47 locale tags and the same catalog
  contract; do not add language-specific branches to format, geometry,
  command, transaction, or rendering logic.
- Stable command ids, error codes, JSON keys and values, WIT/RPC/MCP schemas,
  logs, and receipts remain locale-neutral English identifiers.
- Never translate drawing text, layer/level names, symbol names, file payloads,
  or user script source automatically.

## Dependencies

- Each format core is standard-library-first until a named milestone approves
  an exact exception. Existing reviewed DXF exceptions remain governed by
  `docs/DEPENDENCY_POLICY.md`.
- Run a license, feature, transitive dependency, and security review before
  changing any `Cargo.toml` dependency.
- Do not install agent frameworks, MCP servers, GUI frameworks, ODA, Wasmtime,
  or scripting runtimes without the exact milestone approval.

## Required gates

Run the smallest relevant tests first, then before every checkpoint:

```text
cargo deny --locked check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

Report changed files and line counts, commands, test results, artifact hashes,
deviations, and blockers.
