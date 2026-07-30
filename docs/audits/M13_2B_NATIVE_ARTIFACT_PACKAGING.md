# M13.2b Native Artifact Packaging Audit

## Outcome

M13.2b adds deterministic payload assembly and a manually dispatched workflow
for the six reviewed native target triples.

## Contract evidence

- Target validation accepts exactly the six Linux/Windows/macOS x64/ARM64
  triples.
- A commit identity must be exactly 40 lowercase hexadecimal characters.
- Packaging is create-new and refuses an existing output directory.
- The executable is required to be a nonempty, non-symlink regular file.
- README, SBOM, and all legal entries must be non-symlink regular files.
- Portable relative paths and duplicate destinations fail closed.
- Output inside the legal source tree, depth above 16, and more than 4,096
  legal entries fail before an unbounded package can escape.
- Copies and SHA-256 hashing use a fixed 64-KiB buffer.
- Receipt payload rows are sorted and bind path, byte count, and SHA-256.
- The receipt also binds package, version, target, commit, and Rust 1.97.1.
- Assembly failure removes only the directory created by that invocation.

## Workflow evidence

- `workflow_dispatch` is the sole trigger.
- Workflow permissions are `contents: read`.
- All six builds run on their matching native hosted runner and target triple.
- Checkout v4.2.2 and upload-artifact v4.6.2 use exact reviewed commit pins.
- Credential persistence is disabled.
- Schema, release-evidence, and workspace-test gates precede release builds.
- Upload failure on a missing artifact is explicit; retention is 14 days.

## Tests

One table-driven test assembles all six variants and verifies the target,
commit, sorted 61-file payload inventory, hashes, and platform executable name.
A negative test rejects an unreviewed target and confirms that an existing
destination remains untouched.

## Non-claims

No workflow run or uploaded artifact is claimed by this repository checkpoint.
The workflow cannot publish a GitHub Release. No tag, signature, installer,
reproducible archive, private-corpus achievement, twenty-night completion, or
Core 1.0 authorization is claimed.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (598 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `.github/workflows/release-artifacts.yml` | 80 | `54725fc0789500127430423836a8a79608f3c600ac8b51e56f64ca198123b380` |
| `crates/seacad-schema-gen/Cargo.toml` | 31 | `40237d780161153b559875fd4762062346e51f41f4c0e26b9315a12bc11ff1c2` |
| `crates/seacad-schema-gen/src/bin/seacad-release-packager.rs` | 607 | `27f36f492627456b9965bfed8947398f7cd936a614a3c51c286c2b7fc1fc1e16` |
| `THIRD_PARTY_NOTICES.md` | 212 | `82a98b6b5177de9078df2ba3110b8c76cf22ddb7d54614ec32b9b0afd2a15afe` |
| `release/legal/manifest.json` | 501 | `c4e868e1da7c218df8a9d83c1dd05e3ca3adb9536359ca54b5fce67d692993af` |
| `docs/M13_2B_NATIVE_ARTIFACT_CONTRACT.md` | 62 | `e2d710e7e80c4229369779a627977650b2b107b6c0c04fd703cdef71b860e396` |
| `README.md` | 71 | `caf8f54d9c9faad3608bf0622f63645402adc009b6fc97a96ecc532443ff163f` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,293 | `cccc82241975a397a2ca23ef3eeabb03f8420d20727f34472069087169959dbf` |
| `docs/SUPPORT_MATRIX.md` | 1,044 | `f758aab3b35343305ca7f1c1460a9d50ce8060da2bbf25ad19b3dd30b755eb68` |
| `docs/TOOLCHAIN.md` | 185 | `2511bc7c6980a9f64a342a54aeac973263da2b6b85cdf59a6a4d72cf8f4353d0` |
