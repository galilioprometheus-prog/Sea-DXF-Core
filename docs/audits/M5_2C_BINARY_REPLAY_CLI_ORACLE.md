# M5.2c AutoCAD Binary replay and CLI oracle

Status: completed 2026-07-27

No external parser source was inspected, copied, translated, or ported. The
external checker depends only on the current local `seacad-dxf-core` path. The
AutoCAD-generated fixtures, replay outputs, checker, executable, and logs stay
outside the repository.

## Inputs

The three files were produced by the isolated AutoCAD Core Console 2027 flow
recorded at M5.1a.

| Dialect | Bytes | SHA-256 |
| --- | ---: | --- |
| AC1009/R12 | 3,926 | `76f9f56bf9a4395c7bc324cf831c5e473cad68a5ee0a442975cc7647c08f5a04` |
| AC1015/AutoCAD 2000 | 112,153 | `cd059a69f0c238f28807b2079b53aef6d04c68d0b486416ac2bede3df59de2c3` |
| AC1032/AutoCAD 2018 | 42,235 | `4b5584619ef9ba4665f46caf268891b7dd54ad527955348325484d6f7c80b0ef` |

AutoCAD executable version `26.0.60.0.0` and SHA-256
`fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`
are inherited from the committed generation receipt.

## Replay assertions and result

For each file the checker opens a Strict `DxfBinaryRawDocument`, writes to a
new path through `write_verbatim_to_new_file`, and requires receipt source ID,
output ID, and byte count to match. It independently reads both files and
requires exact byte equality. A second write to the existing destination must
fail and leave its bytes unchanged.

```text
AC1009 bytes=3926 source=76f9f56bf9a4395c7bc324cf831c5e473cad68a5ee0a442975cc7647c08f5a04 output=76f9f56bf9a4395c7bc324cf831c5e473cad68a5ee0a442975cc7647c08f5a04 overwrite=refused exact=true status=ok
AC1015 bytes=112153 source=cd059a69f0c238f28807b2079b53aef6d04c68d0b486416ac2bede3df59de2c3 output=cd059a69f0c238f28807b2079b53aef6d04c68d0b486416ac2bede3df59de2c3 overwrite=refused exact=true status=ok
AC1032 bytes=42235 source=4b5584619ef9ba4665f46caf268891b7dd54ad527955348325484d6f7c80b0ef output=4b5584619ef9ba4665f46caf268891b7dd54ad527955348325484d6f7c80b0ef overwrite=refused exact=true status=ok
files=3 total_bytes=158314 byte_identical=true status=ok
```

All 158,314 bytes replayed byte-identically. The three output SHA-256 values
are the same as the corresponding input values in the table.

## CLI assertions and result

The actual `seacad` executable, not a test-only entry point, ran `inspect
--json` and `verify --json` on all three inputs. Every report retained schema
`v1`, physical format `binary`, Strict conformance, an empty diagnostic list,
the exact source identity/group/EOF accounting, and a redacted null path.

```text
AC1009 inspect=ok verify=verified groups=535 eof=534 path=null source=76f9f56bf9a4395c7bc324cf831c5e473cad68a5ee0a442975cc7647c08f5a04
AC1015 inspect=ok verify=verified groups=10281 eof=10280 path=null source=cd059a69f0c238f28807b2079b53aef6d04c68d0b486416ac2bede3df59de2c3
AC1032 inspect=ok verify=verified groups=5898 eof=5897 path=null source=4b5584619ef9ba4665f46caf268891b7dd54ad527955348325484d6f7c80b0ef
schema=v1 physical=binary strict=3 diagnostics=0 status=ok
```

Synthetic committed CLI tests separately freeze Compatible missing-EOF and
trailing-byte recovery, strict invalid-opening failure, stable core/CLI codes,
Vietnamese human text, and default path redaction.

## External artifact receipt

| Artifact | SHA-256 |
| --- | --- |
| `Cargo.toml` | `aaf850853cd104bff80271160ff477b0a1b2a94a9ea4db256cf0d132ed4e67f5` |
| `Cargo.lock` | `23773581ac2b7c3d7a1253356ba47d1fbd2b20f594a52295702a54370a06f3ef` |
| `src/main.rs` | `310d7e1de3f41d29f09aef10976d5561ae3c5083815c18a02c1e659f1b5a305d` |
| `oracle-results.txt` | `ccedb4090ee4ec8472c530a43e43dc4bc32bde671bdcd1791872ea8bf07f5b56` |
| `cli-results.txt` | `7b0930048abdfdc438ddbe5aaef66343ecaef9c7d517e235591ac36faaa7c13a` |
| checker executable | `a994a62b85ad0233869117bfbfccf1834134ccfcc7fe02976c7e7d496d8f0f01` |
| tested `seacad.exe` | `4ed914cacbd72e7746ef6f330914f70529de869970418f8d6a92c4bb6177a96e` |

Commands were `cargo run --quiet` in the external checker and `cargo run -p
seacad-cli --quiet -- inspect|verify FILE --json` in the repository.

This proves unchanged Binary replay and CLI raw-framing behavior for these
three AutoCAD fixtures. It does not claim semantic decoding, editing,
ASCII/Binary conversion, PreservePatch, or canonical writing.
