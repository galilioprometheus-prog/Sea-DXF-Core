# M3.5 CLI dependency review

Status: approved for M3.5
Review date: 2026-07-27

## Decision and purpose

SeaCad accepts three pinned direct dependencies for the command-line boundary:

```toml
clap = { version = "=4.6.4", default-features = false, features = ["std", "help", "usage", "error-context", "suggestions"] }
serde = { version = "=1.0.229", default-features = false, features = ["derive", "std"] }
serde_json = { version = "=1.0.151", default-features = false, features = ["std"] }
```

Clap owns argument grammar, help, version, and usage errors. Serde and
serde_json serialize the stable JSON v1 report. These packages are scoped to
`seacad-cli`; `seacad-dxf-core` does not depend on them. No third-party source
is vendored, copied, translated, or modified.

Primary package sources:

- <https://crates.io/crates/clap/4.6.4>
- <https://crates.io/crates/serde/1.0.229>
- <https://crates.io/crates/serde_json/1.0.151>

## Locked registry tree

Checksums come from the committed Cargo.lock. License and MSRV values were
checked in the normalized manifests downloaded from crates.io.

| Package | Version | Registry checksum | License | MSRV |
| --- | ---: | --- | --- | ---: |
| anstyle | 1.0.14 | `940b3a0ca603d1eade50a4846a2afffd5ef57a9feac2c0e2ec2e14f9ead76000` | MIT OR Apache-2.0 | 1.66.0 |
| clap | 4.6.4 | `d91e0c145792ef73a6ad36d27c75ac09f1832222a3c209689d90f534685ee5b7` | MIT OR Apache-2.0 | 1.85 |
| clap_builder | 4.6.2 | `f09628afdcc538b57f3c6341e9c8e9970f18e4a481690a64974d7023bd33548b` | MIT OR Apache-2.0 | 1.85 |
| clap_lex | 1.1.0 | `c8d4a3bb8b1e0c1050499d1815f5ab16d04f0959b233085fb31653fbfc9d98f9` | MIT OR Apache-2.0 | 1.85 |
| itoa | 1.0.18 | `8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682` | MIT OR Apache-2.0 | 1.68 |
| memchr | 2.8.3 | `cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98` | Unlicense OR MIT | 1.61 |
| proc-macro2 | 1.0.107 | `985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9` | MIT OR Apache-2.0 | 1.71 |
| quote | 1.0.47 | `1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001` | MIT OR Apache-2.0 | 1.71 |
| serde | 1.0.229 | `4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba` | MIT OR Apache-2.0 | 1.56 |
| serde_core | 1.0.229 | `67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48` | MIT OR Apache-2.0 | 1.56 |
| serde_derive | 1.0.229 | `e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348` | MIT OR Apache-2.0 | 1.71 |
| serde_json | 1.0.151 | `c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14` | MIT OR Apache-2.0 | 1.71 |
| strsim | 0.11.1 | `7da8b5736845d9f2fcb837ea5d9e2628564b3b043a70948a3f0b778838c5fb4f` | MIT | 1.56 |
| syn | 3.0.3 | `53e9bae58849f64dfa4f5d5ae372c8341f7305f82a3868709269343628b659a3` | MIT OR Apache-2.0 | 1.71 |
| unicode-ident | 1.0.24 | `e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75` | (MIT OR Apache-2.0) AND Unicode-3.0 | 1.71 |
| zmij | 1.0.23 | `29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b` | MIT | 1.71 |

All declared MSRVs are below SeaCad's pinned Rust 1.97.1. The locked CLI tree
contains no duplicate versions and no package declares Cargo `links` or a
native-library link target.

## Features, macros, and build scripts

Clap defaults are disabled, so color, environment-variable parsing, derive,
shell completion, Unicode convenience, and deprecated compatibility features
are not enabled. Serde derive is the only procedural macro path. Serde_json
uses only `std`; arbitrary-precision, float-roundtrip, preserve-order, raw
value, and unbounded-depth features are disabled.

Custom build scripts exist in `proc-macro2`, `quote`, `serde`, `serde_core`,
`serde_json`, and `zmij`. The reviewed scripts probe the compiler/version and
conditional language support; proc-macro2 also creates and removes only its
own OUT_DIR probe directory, while serde/serde_core generate private modules
inside OUT_DIR. No reviewed script contains a downloader, network client,
URL, PowerShell, cmd.exe, curl, or wget execution path.

## Advisory review

The official RustSec advisory database was shallow-cloned only for this audit
at commit `29638ff054fdbb83d2844240f7ef7e576cb52629` dated 2026-07-25. An exact
package-name search under its `crates` advisory tree found zero advisory files
for every one of the 16 locked package names above.

- <https://github.com/RustSec/advisory-db/tree/29638ff054fdbb83d2844240f7ef7e576cb52629>

This is a point-in-time review, not a permanent safety guarantee. `cargo-audit`
remains intentionally deferred to its named quality milestone, and M13 must
repeat automated advisory and license checks against the release lockfile.

## Result

The pinned tree is accepted for M3.5. Any package version, registry checksum,
feature, or source change requires a new recorded review.
