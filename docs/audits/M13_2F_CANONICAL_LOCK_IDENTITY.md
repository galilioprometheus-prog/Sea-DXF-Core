# M13.2f Canonical Cargo.lock Identity Audit

## Remote finding

GitHub Actions CI runs `30555367665` and `30556204169` rejected the committed
SBOM only on the Windows ARM64 supplemental runner. M13.2e run `30556646352`
reported:

```text
committed_sha256=ca7c5d001cf2f2745392def707e46c699d6fdc7f163fe514886fd526af8830d0
generated_sha256=6664578ce6e22f9af56ed8f0b6a0c5520719f5a905aaea14e5afc1351d422a26
missing_edges=0 []
unexpected_edges=0 []
```

The graph was identical. The remaining host input was the raw Cargo.lock hash
embedded in SBOM metadata and the legal manifest.

## Root-cause proof

The committed Cargo.lock has 248 LF line endings and no carriage returns.
Converting those line endings to CRLF produced Cargo.lock SHA-256
`cdda7a2358daff560715de5e51ac014dc305a322c31931074c2e2f9ed92316ce`.
Replacing only the LF lock hash in the committed SBOM with that CRLF hash
produced
`6664578ce6e22f9af56ed8f0b6a0c5520719f5a905aaea14e5afc1351d422a26`,
exactly matching the Windows ARM64 runner.

This proves that checkout line-ending conversion, not a dependency-edge
difference, caused the stale evidence.

## Correction

- normalize CRLF to LF before deriving Cargo.lock identity;
- reject a lone carriage return rather than accepting ambiguous text;
- pin `Cargo.lock text eol=lf` in `.gitattributes`; and
- retain the M13.2e bounded hash and edge diagnostics for future failures.

The checksum rows parsed from Cargo.lock are unchanged. No dependency, build
tool, runtime behavior, or support claim is added.

## Gates

Local gates passed on Windows x64 with Rust/Cargo 1.97.1:

- `cargo +1.97.1 fmt --all -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen --bin
  seacad-release-evidence -- --check`;
- `cargo deny --locked check`;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`; and
- `cargo +1.97.1 test --workspace --quiet` (602 passed).

Fresh remote CI and release-artifact receipts remain required after this
checkpoint is pushed.

## Artifact receipt

| Path | Lines | SHA-256 |
|---|---:|---|
| `.gitattributes` | 19 | `4431bd57472127cfc1a6e4d2e2ee655ff96fde25c96509a00b4eb7bccb2392d0` |
| `README.md` | 74 | `1fb11548deb9a4effdb3a7a9a9b43aea5ccdc419c8606d454b36339b100b3483` |
| `crates/seacad-schema-gen/src/bin/seacad-release-evidence.rs` | 1,092 | `69f2b280d3d44bfa6dc37bb973ffae3696d13d4c7eb9471bb68c1dba5cf381c4` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,328 | `49c04dbd966604536decb18a0a5516b1c3d9a22c6bf9047f66421924fda4db5b` |
| `docs/SUPPORT_MATRIX.md` | 1,062 | `0c1519f0e9692c2d20a1a6843ef2e5cfc6503b5a2850e4b48a8170afca2766c4` |
| `docs/TOOLCHAIN.md` | 238 | `ac7f472955f82bc3892597fb48688a207bb894b3cd30db2877d816e2ae6b18c7` |
| `docs/audits/M13_2D_SIX_PLATFORM_SBOM_UNION.md` | 59 | `a5d336cd024aeac17aa330b9b02d83dbf2e9c1f2e28bb77a5a941017a2aef0b0` |
| `release/sbom.cdx.json` (unchanged) | 886 | `ca7c5d001cf2f2745392def707e46c699d6fdc7f163fe514886fd526af8830d0` |
