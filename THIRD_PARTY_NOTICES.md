# Third-Party Notices

M0 contains no third-party Rust runtime dependencies.

Development uses the Rust toolchain and GitHub Actions under their respective
licenses. Runtime and tooling dependencies added by later milestones must be
recorded here after license and supply-chain review.

## M2.3 runtime dependency

SeaCad uses `sha2 0.11.0` from RustCrypto for bounded SHA-256 source
identification. Its default features are disabled. Cargo.lock pins the complete
registry tree below:

| Package | Version | License |
| --- | ---: | --- |
| sha2 | 0.11.0 | MIT OR Apache-2.0 |
| block-buffer | 0.12.1 | MIT OR Apache-2.0 |
| cfg-if | 1.0.4 | MIT OR Apache-2.0 |
| cpufeatures | 0.3.0 | MIT OR Apache-2.0 |
| crypto-common | 0.2.2 | MIT OR Apache-2.0 |
| digest | 0.11.3 | MIT OR Apache-2.0 |
| hybrid-array | 0.4.13 | MIT OR Apache-2.0 |
| libc | 0.2.189 | MIT OR Apache-2.0 |
| typenum | 1.20.1 | MIT OR Apache-2.0 |

Upstream: <https://github.com/RustCrypto/hashes/tree/sha2-v0.11.0/sha2>

The exact checksums, enabled feature tree, build-script review, and advisory
review are recorded in `docs/audits/M2_3_SHA2_DEPENDENCY_REVIEW.md`. No
third-party source is vendored into SeaCad. Required license texts will be
included with distributable artifacts during the M13 release audit.

## M3.5 CLI dependencies

SeaCad's CLI uses pinned versions of Clap for argument parsing and Serde plus
serde_json for JSON v1 serialization. Default features are disabled and only
the reviewed minimal feature sets are enabled. Cargo.lock pins this complete
additional registry tree:

| Package | Version | License |
| --- | ---: | --- |
| anstyle | 1.0.14 | MIT OR Apache-2.0 |
| clap | 4.6.4 | MIT OR Apache-2.0 |
| clap_builder | 4.6.2 | MIT OR Apache-2.0 |
| clap_lex | 1.1.0 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.151 | MIT OR Apache-2.0 |
| strsim | 0.11.1 | MIT |
| syn | 3.0.3 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| zmij | 1.0.23 | MIT |

Upstream projects:

- <https://github.com/clap-rs/clap>
- <https://github.com/serde-rs/serde>
- <https://github.com/serde-rs/json>

Exact checksums, enabled features, build-script review, and advisory evidence
are recorded in `docs/audits/M3_5_CLI_DEPENDENCY_REVIEW.md`. No third-party
source is vendored into SeaCad. Required license texts will be included with
distributable artifacts during the M13 release audit.
