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

## M4.3b1 text-decoding dependency

SeaCad uses pinned `encoding_rs 0.8.35` for a reviewed subset of
replacement-free legacy byte-to-UTF-8 decoding. Default features are disabled;
only `alloc` is enabled. Its sole runtime dependency, `cfg-if 1.0.4`, was
already present in the M2.3 lock tree.

| Package | Version | License |
| --- | ---: | --- |
| encoding_rs | 0.8.35 | (Apache-2.0 OR MIT) AND BSD-3-Clause |

Upstream: <https://github.com/hsivonen/encoding_rs>

`encoding_rs` is copyright Mozilla Foundation. Its non-generated code is
offered under the
[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) or the
[MIT License](https://opensource.org/licenses/MIT), at the recipient's option.
The crate includes data derived from WHATWG Encoding Standard files:

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft).

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors
   may be used to endorse or promote products derived from this software
   without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

The exact archive checksum, feature tree, called API surface, full upstream
BSD-3-Clause conditions, safety boundary, and advisory evidence are recorded
in `docs/audits/M4_3B1_ENCODING_RS_ADOPTION_RECEIPT.md`. No source is vendored.
Required standalone Apache-2.0, MIT, and BSD-3-Clause license files will be
packaged with distributable artifacts at the M13 release audit.
