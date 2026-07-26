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
