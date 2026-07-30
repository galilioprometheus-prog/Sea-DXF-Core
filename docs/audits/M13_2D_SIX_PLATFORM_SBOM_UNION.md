# M13.2d Six-Platform SBOM Union Audit

## Finding

The M13.2c push triggered GitHub Actions CI run `30555367665`. Its Windows
ARM64 smoke job rejected the committed SBOM with `RELEASE_SBOM_STALE`.
M13.2d treated the previously unfiltered Cargo resolve graph as the first
cross-host variable to remove. Later M13.2e diagnostics showed that the remote
edge set was identical; M13.2f records and corrects the actual Cargo.lock line
ending cause.

## Correction

M13.2d makes the release-evidence graph explicit and deterministic:

- run `cargo metadata --locked --format-version 1 --filter-platform` once for
  each of the six reviewed target triples;
- require every filtered graph to report the exact same workspace-member set;
- union packages by exact Cargo package identity;
- union each node's resolved dependency identities in sorted order; and
- render one matrix-wide CycloneDX SBOM for all six native packages.

The legal bundle continues to use the same union package inventory. No target
installation or cross-compilation is required to generate or verify the
evidence.

## Scope

This correction changes release-evidence generation only. It does not add a
dependency, a runtime-format claim, or a platform-support claim. A fresh remote
six-native CI run remains the authoritative confirmation that every reviewed
host reproduces the same committed evidence.

## Gates

Local gates passed on Windows x64 with Rust/Cargo 1.97.1:

- `cargo +1.97.1 fmt --all -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen --bin
  seacad-release-evidence -- --check`;
- `cargo deny --locked check`;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`; and
- `cargo +1.97.1 test --workspace --quiet` (600 passed).

Remote CI run `30556204169` showed that the graph hardening was deterministic
but insufficient by itself. The follow-up root-cause evidence is recorded in
the M13.2f audit.

## Artifact receipt

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 73 | `bdcac5bd0c0d35123e0ff7465608ac0ea1d8c57d6c368d2d55391341170a1f5c` |
| `crates/seacad-schema-gen/src/bin/seacad-release-evidence.rs` | 1,006 | `51ebc3e00d545277f17dabd1c7c09b779d26ada5ee64bc7aa6ae70a918967e29` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,322 | `5bef0df9a12a7197c8bc31100c4f8aa250197585d92cfb4b4cb7c7571b8f2a77` |
| `docs/SUPPORT_MATRIX.md` | 1,060 | `1a0fc6a67e897b9a9ef6e5280d7a78d9e7252ae75b21e722adcbb475511e2afe` |
| `docs/TOOLCHAIN.md` | 224 | `b4a10ea64e20f51ddccbb2b036f2f2d319accf44319f5945266cf877a873890f` |
| `release/sbom.cdx.json` | 886 | `ca7c5d001cf2f2745392def707e46c699d6fdc7f163fe514886fd526af8830d0` |
