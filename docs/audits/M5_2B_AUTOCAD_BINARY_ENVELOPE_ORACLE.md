# M5.2b AutoCAD Binary envelope/index oracle

Status: completed 2026-07-27

No external parser source was inspected, copied, translated, or ported. The
external checker depends only on the current local `seacad-dxf-core` path.

## Inputs

The three files were produced by the isolated AutoCAD Core Console 2027 flow
recorded in M5.1a and remain outside the repository.

| Dialect | Bytes | SHA-256 |
| --- | ---: | --- |
| AC1009/R12 | 3,926 | `76f9f56bf9a4395c7bc324cf831c5e473cad68a5ee0a442975cc7647c08f5a04` |
| AC1015/AutoCAD 2000 | 112,153 | `cd059a69f0c238f28807b2079b53aef6d04c68d0b486416ac2bede3df59de2c3` |
| AC1032/AutoCAD 2018 | 42,235 | `4b5584619ef9ba4665f46caf268891b7dd54ad527955348325484d6f7c80b0ef` |

AutoCAD executable version `26.0.60.0.0` and SHA-256
`fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`
are inherited from the committed generation receipt.

## Assertions and result

For every file the checker requires strict conformance, the known encoding and
supported `$ACADVER`, exact group count/source identity, EOF as the last group,
no document or index diagnostic, no tail, complete inside/outside accounting,
section-range sum equality, exact frozen section/group-zero counts, and Closed
closure for every section.

```text
AC1009 groups=535 sections=4 zero_groups=41 inside=534 outside=1 eof=534 status=ok
AC1015 groups=10281 sections=6 zero_groups=225 inside=10280 outside=1 eof=10280 status=ok
AC1032 groups=5898 sections=7 zero_groups=171 inside=5897 outside=1 eof=5897 status=ok
total_groups=16714 total_sections=17 total_zero_groups=437 status=ok
```

All 16,714 groups are accounted exactly once: 16,711 inside 17 non-overlapping
closed sections and three terminal EOF groups outside sections.

## External checker receipt

| Artifact | SHA-256 |
| --- | --- |
| `Cargo.toml` | `73a420337c00358b14bb8611af9f549af1f8716b5d24d28f34d52db4cbe63cbd` |
| `Cargo.lock` | `fa56a924922f8a65e535c96799f37457d115e5be3f06b983625d04da0bd7a1d8` |
| `src/main.rs` | `0ccb9393f56ee5f5a48ee84fdf54165486bb375bfb8e0bb24ac3a5286d315de8` |
| `oracle-results.txt` | `df87beeb3b915f730e294d190d9a9f88f37eb30c5b71a3df22f217ba0b548790` |
| checker executable | `fe400986771c6096923ba9164a598db167566aa35b4e8795228adaf5ce3ebdcb` |

Command: `cargo run --quiet` from the external checker directory. The checker,
build output, log, and CAD sources are not committed.

This proves strict Binary envelope/index behavior for these three AutoCAD
fixtures. Synthetic committed tests provide malformed section and Compatible
recovery evidence. M5.2c replay/CLI, semantic decoding, edits, and writers are
not claimed.
