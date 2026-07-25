# Dependency Policy

## Runtime

- `seacad-dxf-core` is standard-library-only through the initial source and
  framing milestones.
- `encoding_rs` may be proposed only at the encoding milestone.
- `sha2` may be proposed only at the transaction/hash milestone.
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
