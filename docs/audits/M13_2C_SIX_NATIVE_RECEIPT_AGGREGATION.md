# M13.2c Six-Native Receipt Aggregation Audit

## Outcome

M13.2c extends the manual native workflow with post-storage verification and
one matrix receipt covering all six reviewed target artifacts.

## Verification evidence

- The root set must exactly equal the six versioned target package names.
- Receipt JSON is bounded to 1 MiB and rejects unknown fields.
- Contract, schema, package, target, version, Rust, and commit must match.
- Payload rows are bounded, portable, strictly sorted, and digest-validated.
- Filesystem traversal rejects symlinks, special entries, depth/count excess,
  missing files, and extra files.
- Every payload is streamed through a fixed 64-KiB SHA-256 buffer.
- The expected non-executable payload set is rebuilt from the checkout.
- README, SBOM, and every legal artifact are byte/hash compared to the
  checkout, preventing a self-consistent forged payload/receipt pair.
- Native executable bytes remain bound by the target receipt.
- The aggregate binds all six receipt SHA-256 values and file/byte totals.
- Aggregate output is create-new at one exact root filename.

## Workflow evidence

- Aggregation depends on successful completion of the six-job package matrix.
- Download-artifact v4.3.0 uses exact commit
  `d3f86a106a0bac45b974a628896c90dbdf5c8093`.
- Checkout and upload actions retain their exact reviewed pins.
- Download keeps artifacts in distinct versioned package directories.
- Only the aggregate receipt is re-uploaded, with 14-day retention.
- Workflow permissions remain `contents: read`; no release mutation exists.

## Tests

A synthetic exact six-target matrix produces a stable aggregate with 24
payload rows. Independent mutations prove payload-hash failure and exact-set
failure when a target is missing. M13.2b's tests continue to cover the real
61-payload package shape for each target.

## Non-claims

No native workflow run or aggregate receipt is claimed at this checkpoint.
There is no tag/release publication, signing, installer, reproducible-build,
private-corpus, twenty-night, or Core 1.0 closure claim.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (600 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `.github/workflows/release-artifacts.yml` | 113 | `cd76b165ba9d752953c8065c01d6468fe5fd9869b4f81e2785a6994795e44c25` |
| `crates/seacad-schema-gen/Cargo.toml` | 35 | `9766cee6d740fe47306dc19be39a9245ad609f39dcb0d3e00db1434cc702bea7` |
| `crates/seacad-schema-gen/src/bin/seacad-release-packager.rs` | 609 | `2ecb5244fecc4f40eadf54a73d5f1c8762a1a09da0d9a835685056b82f879a58` |
| `crates/seacad-schema-gen/src/bin/seacad-release-receipts.rs` | 888 | `a100f54e6d47325a0ff2859053f33c07c10efcef5a1a9afa42cc448e46a7859f` |
| `THIRD_PARTY_NOTICES.md` | 216 | `47aab18e969b87c863a404998970e187e5d180c6395be11f2cc442a40dda86aa` |
| `release/legal/manifest.json` | 501 | `83a7ca58381bdde9371a23bda94a224ead894c51b07241246cc662ed20b1dfb0` |
| `docs/M13_2C_SIX_NATIVE_RECEIPT_CONTRACT.md` | 67 | `054e49bbbc452e7fa3f827ff7bee8beb10a21f3f60a5653cd65a44641503a36c` |
| `README.md` | 72 | `929c1d4b792cf2bf7aece084ee21b1e5c4179c86fc12c4496ed7cc76a44bf3f1` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,309 | `8ed207e7c75a394bd9e4ebc995406e97632a1dacda18c348b84307618b175053` |
| `docs/SUPPORT_MATRIX.md` | 1,053 | `268ef2cc005b4a659a8c91f11cfcbeb3f5a5d30c26327428dff991906df53cec` |
| `docs/TOOLCHAIN.md` | 208 | `4720426713a27f20777657f7418f00a6ffca48ef036cf640b15a3c67a01893c8` |
