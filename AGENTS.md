# SeaCad Agent Rules

## Mission

Build the independent SeaCad DXF core in small, evidence-backed milestones.
DXF Core 1.0 covers ASCII and Binary DXF AC1009 through AC1032. Unknown,
custom, proxy, and non-public payloads remain exact opaque data.

## Milestone discipline

- Follow `docs/IMPLEMENTATION_PLAN.md` in order.
- Stop after every micro-milestone for user approval.
- Use one commit and one annotated checkpoint tag per approved milestone.
- Do not increase a support claim without fixtures, tests, evidence, and an
  artifact SHA-256 receipt.
- Keep production changes near 200-500 lines per micro-milestone. Split work
  before it becomes difficult to review.

## Source and provenance

- Treat every earlier workspace outside the current SeaCad repository as
  read-only, regardless of later path moves. Known legacy trees include sibling
  directories matching `D:\SeaCad\cad_*` and the former `D:\Backups` location.
- Do not fork, vendor, copy, translate, or line-by-line port an external parser.
- Official Autodesk documentation is normative. External implementations may
  be isolated behavioral oracles only.
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

## Dependencies

- The core is standard-library-only until a named milestone approves an
  exception.
- Run a license, feature, transitive dependency, and security review before
  changing any `Cargo.toml` dependency.
- Do not install agent frameworks, MCP servers, GUI frameworks, ODA, Wasmtime,
  or scripting runtimes without the exact milestone approval.

## Required gates

Run the smallest relevant tests first, then before every checkpoint:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

Report changed files and line counts, commands, test results, artifact hashes,
deviations, and blockers.
