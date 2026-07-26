# M2.3 sha2 dependency review

Status: approved for M2.3
Review date: 2026-07-27

## Decision and purpose

SeaCad accepts the RustCrypto `sha2` crate solely to calculate SHA-256 over
the exact source bytes observed by a bounded scan. The digest is a stable
source identity and integrity precondition; it is not authentication.

The direct requirement is pinned as:

```toml
sha2 = { version = "=0.11.0", default-features = false }
```

No upstream source is vendored, copied, translated, or modified.

## Direct package review

| Field | Reviewed value |
| --- | --- |
| Package | `sha2 0.11.0` |
| Registry checksum | `446ba717509524cb3f22f17ecc096f10f4822d76ab5c0b9822c5f9c284e825f4` |
| License | MIT OR Apache-2.0 |
| MSRV | Rust 1.85 |
| SeaCad toolchain | Rust 1.97.1 |
| Enabled direct features | none; `alloc`, `oid`, and `zeroize` are disabled |
| Upstream tag | `sha2-v0.11.0` in `RustCrypto/hashes` |

Primary sources:

- <https://crates.io/crates/sha2/0.11.0>
- <https://github.com/RustCrypto/hashes/tree/sha2-v0.11.0/sha2>
- <https://raw.githubusercontent.com/RustCrypto/hashes/sha2-v0.11.0/sha2/Cargo.toml>

## Locked transitive packages

| Package | Version | Registry checksum | License | MSRV |
| --- | ---: | --- | --- | ---: |
| block-buffer | 0.12.1 | `d2f6c7dbe95a6ed67ad9f18e57daf93a2f034c524b99fd2b76d18fdfeb6660aa` | MIT OR Apache-2.0 | 1.85 |
| cfg-if | 1.0.4 | `9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801` | MIT OR Apache-2.0 | 1.32 |
| cpufeatures | 0.3.0 | `8b2a41393f66f16b0823bb79094d54ac5fbd34ab292ddafb9a0456ac9f87d201` | MIT OR Apache-2.0 | 1.85 |
| crypto-common | 0.2.2 | `ce6e4c961d6cd6c9a86db418387425e8bdeaf05b3c8bc1411e6dca4c252f1453` | MIT OR Apache-2.0 | 1.85 |
| digest | 0.11.3 | `f1dd6dbb5841937940781866fa1281a1ff7bd3bf827091440879f9994983d5c2` | MIT OR Apache-2.0 | 1.85 |
| hybrid-array | 0.4.13 | `818356c5132c1fede50f837ca96afbe78ff42413047f4abb886217845e1b6c8c` | MIT OR Apache-2.0 | 1.85 |
| libc | 0.2.189 | `3eaf3ede3fee6db1a4c2ee091bf8a8b4dccdc6d17f656fb07896ee72867612f2` | MIT OR Apache-2.0 | 1.65 |
| typenum | 1.20.1 | `b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20` | MIT OR Apache-2.0 | 1.41.0 |

`cargo tree --target all -e features` records the active transitive feature
edges: cfg-if/default, cpufeatures/default, digest/default plus block-api,
block-buffer/default, hybrid-array/default, crypto-common/default, and
typenum/default plus const-generics. No duplicate package versions resolve.

## Build and runtime surface

- No package exposes a Cargo `links` value or native-library link step.
- No package is a procedural macro.
- RustCrypto retains automatic target-specific CPU detection with a portable
  software fallback. SeaCad forces no experimental backend configuration.

- `libc 0.2.189` is the only package with a custom build target. Its reviewed
  build script probes the Rust compiler/wrapper and, only on matching targets,
  `freebsd-version` or `emcc`; no network API, URL, downloader, PowerShell, or
  `cmd.exe` invocation was found.
- The source scanner uses one fixed 64 KiB buffer and no source-sized
  allocation. The dependency has no file or network access in SeaCad code.

## Advisory review

The RustSec package index and package-specific entries were reviewed on the
date above. RustSec lists one historical advisory for `sha2`:
`RUSTSEC-2021-0100`, affecting 0.9.7 and patched in 0.9.8. The locked 0.11.0
is outside the affected range. None of the eight locked transitive package
names appears in the RustSec package index, which lists packages having
advisories.

- <https://rustsec.org/packages/sha2.html>
- <https://rustsec.org/advisories/RUSTSEC-2021-0100.html>
- <https://rustsec.org/packages/>

`cargo-audit` remains intentionally deferred to its named quality milestone;
M13 must repeat the automated advisory and license audit against the then
current lockfile.

## Result

The locked tree is accepted for M2.3. All nine registry packages are dual
licensed MIT OR Apache-2.0, their MSRVs are below SeaCad's pinned Rust 1.97.1,
and the reviewed advisory does not affect the selected versions. Any version,
feature, or source change requires a new review.
