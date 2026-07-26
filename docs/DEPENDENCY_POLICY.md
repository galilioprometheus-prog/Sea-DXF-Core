# Dependency Policy

## Runtime

- `seacad-dxf-core` began standard-library-only and keeps runtime exceptions
  milestone-scoped and pinned.
- `sha2 = 0.11.0` with default features disabled is approved at M2.3 only for
  streaming SHA-256 source identity.
- `encoding_rs` may be proposed only at the encoding milestone.
- `seacad-cli` may add `clap`, `serde`, and `serde_json` when versioned CLI
  output is implemented.

## Development

- `proptest` may be added when property tests begin.
- `cargo-fuzz` runs as a separately installed tool on supported CI hosts.
- `cargo-audit`, `cargo-deny`, `cargo-llvm-cov`, `cargo-cyclonedx`, and
  `cargo-semver-checks` are installed only at their named quality milestones.

Every addition requires a recorded purpose, exact version, license, enabled
features, transitive dependency review, advisory review, and
`THIRD_PARTY_NOTICES.md` update when applicable.
