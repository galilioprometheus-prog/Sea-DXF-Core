# Dependency Policy

## Runtime

- `seacad-dxf-core` began standard-library-only and keeps runtime exceptions
  milestone-scoped and pinned.
- `sha2 = 0.11.0` with default features disabled is approved at M2.3 only for
  streaming SHA-256 source identity.
- `encoding_rs` may be proposed only at the encoding milestone.
- `seacad-cli` uses pinned `clap = 4.6.4`, `serde = 1.0.229`, and
  `serde_json = 1.0.151` with default features disabled for argument parsing
  and stable JSON v1 output. Their M3.5 review is recorded in
  `docs/audits/M3_5_CLI_DEPENDENCY_REVIEW.md`.

## Development

- `proptest` may be added when property tests begin.
- `cargo-fuzz` runs as a separately installed tool on supported CI hosts.
- `cargo-audit`, `cargo-deny`, `cargo-llvm-cov`, `cargo-cyclonedx`, and
  `cargo-semver-checks` are installed only at their named quality milestones.

Every addition requires a recorded purpose, exact version, license, enabled
features, transitive dependency review, advisory review, and
`THIRD_PARTY_NOTICES.md` update when applicable.
