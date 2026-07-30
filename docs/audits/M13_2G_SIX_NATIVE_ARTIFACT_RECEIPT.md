# M13.2g Six-Native Artifact Receipt Audit

## Workflow receipt

- workflow: `Native Release Artifacts`;
- GitHub Actions run: `30557566354`;
- event: manual `workflow_dispatch`;
- source commit: `222eec2c9d9b18fbb7ff1b8d5f0120ba30633365`;
- status: success;
- total duration: 5 minutes;
- package matrix: 6/6 succeeded; and
- aggregate verifier: succeeded in 29 seconds.

The aggregate verifier downloaded all six same-run packages, validated every
per-target receipt and payload hash, compared non-executable payloads with the
checkout, and uploaded the seventh artifact.

## GitHub artifact digests

| Artifact | Size | GitHub SHA-256 |
|---|---:|---|
| `seacad-dxf-core-0.0.0-aarch64-apple-darwin` | 552 KB | `fbecfea5eb1908ce55f75b2747f7dabd65104e02895bdc49346b70af5521e784` |
| `seacad-dxf-core-0.0.0-aarch64-pc-windows-msvc` | 431 KB | `59d547ddbfd31ebfba846ae30e3fccd256f17e25f5c53b98f91e944442a9653f` |
| `seacad-dxf-core-0.0.0-aarch64-unknown-linux-gnu` | 590 KB | `66eb9af1a5c170a0c5597aaa5e128bb1e914f0588df37d51eb567ea134fef7a8` |
| `seacad-dxf-core-0.0.0-x86_64-apple-darwin` | 565 KB | `28627cc14d3cabc955dd7dff2bf6a477b2a89d8b7ef94d8cbd3b4250d9edd1fd` |
| `seacad-dxf-core-0.0.0-x86_64-pc-windows-msvc` | 437 KB | `738287234f9c3856331641322551bf6708697678dffbf1bd9308ad1099b8587d` |
| `seacad-dxf-core-0.0.0-x86_64-unknown-linux-gnu` | 600 KB | `3f45bf561b733e66419dbd07452f9a534bd1f3fa19c4e95629d7ed9e0289e39b` |
| `seacad-dxf-core-0.0.0-six-native-receipt` | 766 bytes | `d287c406f7c12e1d0e3686f294b1f4b5cd50567b25d3565c8e6fe0ba1cf6d97d` |

GitHub reported only the expected Node20-deprecation warnings for the
compatibility-pinned checkout/upload actions, which it forced to Node24. No
error annotation was present.

## Scope

The artifacts use the workflow's configured 14-day retention. This receipt
closes the first successful six-native packaging and aggregate verification
run. It does not claim permanent retention, signatures, reproducible archives,
private-corpus scale, twenty consecutive nightly runs, or final Core 1.0
authorization.

## Gates

Local gates passed on Windows x64 with Rust/Cargo 1.97.1:

- `cargo +1.97.1 fmt --all -- --check`;
- generated schema and release evidence `--check`;
- `cargo deny --locked check`;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`; and
- `cargo +1.97.1 test --workspace --quiet` (602 passed).

A second workflow run on the documentation checkpoint remains the external
receipt for the exact final packaged README.

## Documentation artifact receipt

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 74 | `e9298ba9714e32790db8702b341ef4b80fadc8f88521a41d63a015df8606a7c1` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,338 | `6c0fc45527cfafc9296c8bf44969b15796def8c09094f63989081bf3d21188c9` |
| `docs/SUPPORT_MATRIX.md` | 1,067 | `78e8c98491a84527c136b218bae2f3944319e7b6048ab063bcf7e72e0c7c88db` |
| `docs/TOOLCHAIN.md` | 248 | `989788bab594499227a4af60c2e24dcc1671dd9f58318eff748b9902056be4d7` |
