# Q1 cargo-deny dependency-policy receipt

Date: 2026-07-28

## Scope

Q1 adds an independent dependency-policy checkpoint with `cargo-deny 0.20.2`.
It changes packaging and quality gates only. No Rust API, DXF parser behavior,
schema, CLI output, runtime dependency, or `Cargo.lock` entry changes.

All three workspace crates are private and non-publishable. The local required
gate is:

```text
cargo deny --locked check
```

The required GitHub status check is `dependency-policy`.

## Reviewed policy

- The complete all-feature graph is evaluated for Windows x64, Windows ARM64,
  Linux x64, and macOS ARM64.
- Vulnerability, notice, unmaintained, unsound, and yanked findings are not
  ignored. Unmaintained and unsound scopes are `all`; yanked is `deny`.
- Development dependencies are included. Allowed licenses are only MIT,
  Apache-2.0, BSD-3-Clause, and Unicode-3.0.
- Duplicate versions, registry wildcard dependencies, unaudited build scripts,
  native executables, interpreted files, archives, unknown registries, and Git
  sources fail the gate. Wildcard internal path dependencies remain allowed
  because every workspace package has `publish = false`.
- `sha2`, `encoding_rs`, `clap`, `serde`, and `serde_json` use exact reviewed
  feature sets.
- Build-script permission is limited to the exact crate versions recorded in
  `deny.toml`.

`libc 0.2.189` packages the Python maintenance helper
`etc/libc-util.py`. The build-content scan detected it even though SeaCad does
not execute it. Q1 therefore adds a single exact-path bypass locked to SHA-256
`a99fdefe6354c28c52eeac9c1e851041b69dad25b8a3b521d4f57b761517c798`.
This evidence-driven addition is narrower than disabling interpreted-file
checks or granting a crate-wide exception.

## Tool and CI pins

| Artifact | Pin | License |
| --- | --- | --- |
| `cargo-deny` | 0.20.2 | MIT OR Apache-2.0 |
| `EmbarkStudios/cargo-deny-action` | `3c6349835b2b7b196a839186cb8b78e02f7b5f25` (v2.1.1) | MIT OR Apache-2.0 |
| `actions/checkout` | `de0fac2e4500dabe0009e67214ff5f5447ce83dd` (v6.0.2) | MIT |
| Rust toolchain | 1.97.1 | repository pin |

The workflow grants only `contents: read`, disables persisted checkout
credentials, uses a ten-minute timeout, and runs on push, pull request, a daily
schedule, and manual dispatch.

## Verification

The reviewed host reported `cargo 1.97.1`, `rustc 1.97.1`, and
`cargo-deny 0.20.2`.

| Command or control | Result |
| --- | --- |
| `cargo deny --locked check` | passed: advisories, bans, licenses, sources |
| Remove `BSD-3-Clause` from a temporary config | failed with exit 4; `encoding_rs` license rejected |
| Remove crates.io from allowed registries | failed with exit 8; registry sources not allowed |
| Remove `libc 0.2.189` from build-script allowlist | failed with exit 2; build script not allowed |
| Remove enabled `help` from exact `clap` features | failed with exit 2; exact feature mismatch |
| `cargo metadata --locked --format-version 1 --no-deps` | all three packages reported `publish = []` |
| `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check` | passed |
| `cargo +1.97.1 fmt --all -- --check` | passed |
| `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo +1.97.1 test --workspace` | passed: 239 tests, 0 failed |
| `git diff --check` | passed |

The four negative-control configs were created under
`D:\Backups\q1_negative_controls`, outside the working tree, and removed after
the run.

## Diff and lockfile

The implementation diff before adding this receipt contains 10 files, 198
insertions, and 10 deletions. No production Rust file is changed.

`Cargo.lock` had SHA-256
`f48f459ed5a7c7b9fb9d184d78099c5c6b31b260b3d13df632dbf539ea07ad70`
before Q1 and the same byte-for-byte hash after all checks.

## Artifact SHA-256

| Artifact | SHA-256 |
| --- | --- |
| locally built `cargo-deny.exe` used for verification (8,885,248 bytes) | `776dedcf7ec895a4995720d32be561bbccf39e388532a1308391b4ad6f091861` |
| `deny.toml` | `21f3baeaad5c35cbbd8fdff5172be1da5ddbb4e867149c7c8ca9a57e93cbf317` |
| `.github/workflows/dependency-policy.yml` | `6fe03f846c7362f0ea8d99468776a230572832f49675fe4a71155785be21e6dd` |
| `.github/workflows/ci.yml` | `08865f59d6ad76cfea371f472e4ef7f50fd70d4606ae21ff588efad88b42fddf` |
| `crates/seacad-dxf-core/Cargo.toml` | `cbd433bf8daa8f679d823e14d4490e63f71334b2146adb6397df3e5533695e78` |
| `crates/seacad-cli/Cargo.toml` | `2597843c9c1a39eebd514f4595a2b26d3bf10daac0e643cf424761e20ec58ba5` |
| `AGENTS.md` | `25967b57978018733025b59f46343d972dae135fa90e0afb6c1b8ab1a2c9dfba` |
| `docs/DEPENDENCY_POLICY.md` | `71882fc20f370a13d00c4b397c2fabfc91d0b512772cf19cf11ff4f5132cabe1` |
| `docs/TOOLCHAIN.md` | `11c261358dd67edbee14b2970920f5cf6057498a3c16c49184258524f46762db` |
| `docs/IMPLEMENTATION_PLAN.md` | `e6ca79344107cfaa9bd583f38dbab75e6e4e32540d898ae7fcbd8fad70dd2c92` |
| `THIRD_PARTY_NOTICES.md` | `6511ef41909b03d650d3b24f5822aa496e0f6763fb0db43ef4404052f418f5af` |
| `Cargo.lock` | `f48f459ed5a7c7b9fb9d184d78099c5c6b31b260b3d13df632dbf539ea07ad70` |

No commit, CI run, or annotated tag was created. Those actions remain deferred
until checkpoint approval.
