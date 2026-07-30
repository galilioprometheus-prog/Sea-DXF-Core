# M13.2a Corpus Release Threshold Audit

## Outcome

M13.2a adds a real, redacted 1,000-file/10-GiB release threshold to the offline
corpus harness while preserving the original Q2.2 v1 behavior.

## Contract

- Manifest/receipt v1 remains ceiling-only and byte-compatible in behavior.
- Manifest/receipt v2 is selected only by exact contract and schema version.
- Committed v2 minimums are 1,000 verified files and 10 GiB.
- Every selected file must pass strict `Safe` ASCII/Binary framing; one invalid
  file prevents release verification.
- Independent v2 ceilings are depth 32, 20,000 entries, 2,000 selected files,
  and 20 GiB.
- Positive minimums must not exceed their configured maxima.
- Requirements/limits are validated before any corpus traversal.
- Receipt v2 reports public requirements and four aggregate threshold states.
- Threshold shortfall emits a completed failed receipt without a false
  per-file failure code.
- Exit `0` requires the conjunction of file, byte, and zero-invalid thresholds.
- All v1 path/identity/hash/timestamp privacy exclusions remain enforced.

## Evidence

- Existing six v1 tests remain unchanged and pass.
- A small synthetic v2 policy proves simultaneous file/byte/zero-invalid
  success and receipt v2 selection.
- Independent synthetic shortfalls prove file and byte threshold failure while
  retaining an empty per-file failure map.
- Requirements above configured traversal limits fail with `CORPUS-E0007`.
- The committed v2 manifest is parsed and asserts exact 1,000/10-GiB minimums
  plus 2,000/20-GiB maxima.

## Non-claims

M13.2a implements release policy and receipt semantics. No private corpus was
read or committed, and no v2 receipt claiming threshold achievement exists.
This milestone does not create a cryptographic corpus commitment, close
six-native evidence, build/sign native release archives, assign a Core 1.0
version, or authorize release.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (596 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-cli/src/bin/seacad-corpus-receipt.rs` | 859 | `5bd3cd4ac42cd8acdba0de5606a84b267d8d0acd3143fab046dd50cf5f2618a6` |
| `corpus/release-manifest.json` | 20 | `7d835f10a8cb83b6376f1d93d6dcada6dce11736fc805ca5dc0bd6cb3faefb15` |
| `docs/M13_2A_CORPUS_RELEASE_GATE.md` | 54 | `d99fdd3d1c91681f00b59b27784895123ccafe8f126d6f4dee6869abafe64d86` |
| `README.md` | 70 | `001a55274dee9de30fc9b37af4bdf52ed72830aedd96b7eeae61984c8aac2c78` |
| `docs/IMPLEMENTATION_PLAN.md` | 1278 | `b139fc87df1663dab181c76fe7ff0f43ab1cc99866ee7856c24ece95010e12e4` |
| `docs/SUPPORT_MATRIX.md` | 1034 | `3d72f7b42434fde63e79b0b669926e2365d185cc7a5a710b1d11354457639782` |
| `docs/TOOLCHAIN.md` | 156 | `9da94f52fa6a58e1593f9ffe3b494ba1b5df257ac3563efc750e09bc19cb869c` |
