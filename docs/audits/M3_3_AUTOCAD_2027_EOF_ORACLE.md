# M3.3 AutoCAD 2027 EOF-envelope oracle

Status: completed 2026-07-27

This receipt records isolated AutoCAD behavior around the ASCII DXF EOF
envelope. It informs SeaCad framing policy but does not make AutoCAD the source
of SeaCad semantics, and no AutoCAD output is used as byte-preservation proof.

## Oracle identity and isolation

| Item | Receipt |
| --- | --- |
| Product | AutoCAD Core Console 2027 |
| File/product version | `26.0.60.0.0` |
| Executable SHA-256 | `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77` |
| Script payload | `_STATUS<CR><LF>_QUIT<CR><LF>` |
| Script SHA-256 | `16ec33b53f3ac48bd05df46b7828b366f6382726c160a9cb4ed99ae2136004f3` |

Every input was generated in a new temporary directory outside the repository.
The invocation used this shape:

```text
accoreconsole /i <generated-copy> /s <reviewed-script>
  /isolate SeaCadM33-<case-id> <isolated-user-data> /readonly /safemode
```

The isolated registry/user-data identity prevented shared profile state,
`/readonly` prevented input writes, and `/safemode` disabled executable code
and drawing-directory autoload. No AutoCAD component is linked or distributed
with SeaCad. The committed receipt contains only synthetic case IDs, sizes,
hashes, exit codes, and summarized signals; local paths and console logs are
not committed.

An accepted case required process exit code `0`, successful `STATUS`, and an
object inventory. Rejected framing/structure cases returned `53`, discarded
the drawing, and never executed `STATUS`.

## Synthetic baseline

The baseline is the same empty AC1032 document used by M3.2:

```text
0 SECTION
2 HEADER
9 $ACADVER
1 AC1032
0 ENDSEC
0 SECTION
2 ENTITIES
0 ENDSEC
0 EOF
```

Conventional group-code padding and CRLF endings are used unless the case name
says otherwise. Accepted baseline-derived cases reported the same 120-object
empty drawing.

## Results

| Case ID | Bytes | SHA-256 | Exit | Result |
| --- | ---: | --- | ---: | --- |
| `baseline_exact` | 120 | `86f50c6bc9f161b4acd947aac6ee5e21f8744573ff32d2325206ef3721a8e33c` | 0 | Accepted |
| `minimal_eof` | 10 | `d2d5cf6387d6dc577d27369ddaf1d5338988de54151f50c253431578cc209f0d` | 53 | Rejected: invalid header |
| `eof_no_final_terminator` | 118 | `6d31a1acc2bda704abbec72091a10346ae225579acda5445a9ed756203568ff9` | 0 | Accepted |
| `eof_lowercase` | 120 | `783062156ca1053f029fb04db30bb9baed110eae9dbcb3134e240d62704e5189` | 53 | Rejected at EOF line |
| `eof_mixed_case` | 120 | `0383765d85c45f3ef4976f1a93c40479ed2fac121bd7c312cc56e1abf4c656e0` | 53 | Rejected at EOF line |
| `eof_leading_space` | 121 | `a992dcd546642fe03e163e3da3c1c1e5bc986e64f6a569c121f91277df86bfa7` | 53 | Rejected at EOF line |
| `eof_trailing_space` | 121 | `8096d62fd4388e6a55fbec48ad193825c29b83f1a83291fa29198fa3f3837685` | 53 | Rejected at EOF line |
| `eof_tab_padded` | 122 | `73a106b9d2099c3bef44a665d5e80873971b1867bd2733fc05aaa4d966e9de8e` | 53 | Rejected at EOF line |
| `eof_plus_zero_code` | 119 | `26c5652dbc6e1f282ff78ad78feb20f861ca97922a663cb3a69e9b2a6b8268f6` | 0 | Accepted |
| `missing_eof` | 110 | `6fa03844a56fcdcf9dd8afc7ea80fdb01625ccfd9f363452d7bf6ab39f5285a3` | 53 | Rejected |
| `missing_eof_no_final_terminator` | 108 | `c1661a32283355616dab5a3b71b83be552022241a4e6110c5f2d0e3a3c6f2718` | 53 | Rejected |
| `duplicate_eof` | 130 | `18abb2cd82b2da60101701f7dbc123012f02317d4796f68bbe794b8164630204` | 0 | Accepted; second EOF ignored |
| `eof_then_comment` | 136 | `94ac51a8d85f85ba34ed13d079b4897c0a68e178b4334b98192b7aec32e92939` | 0 | Accepted; later group ignored |
| `early_eof_then_section` | 88 | `9c082e202dfba5c5e08605355563c8fbed777c07cc8de07b9db893a950256097` | 53 | Rejected: invalid header structure |
| `trailing_blank_line` | 122 | `66fedbb44a2eb6e3b28346deb2ea427cf8cd4a59eb275b6b5d0e6d80287e4ab3` | 0 | Accepted |
| `trailing_space_line` | 123 | `38c282ae84511a6bef43caa7cf22dc92c71ab2dc4694c557c796e440462f15fa` | 0 | Accepted |
| `trailing_sub` | 121 | `782fa43a12c3da8c968052af978f066805d45f70e633642548c395577b39d94d` | 0 | Accepted |
| `trailing_sub_crlf` | 123 | `07498845169f54584e217e2e9fa622ff63afdf9ef47bfe9feee0cdd3ef1b53b1` | 0 | Accepted |

## SeaCad policy derived from the evidence

AutoCAD confirms that EOF value bytes are case-sensitive and are not trimmed,
that an unterminated final EOF value is complete, and that valid integer
spelling such as `+0` remains group code zero. Strict mode follows those
results.

AutoCAD also stops at the first valid EOF and ignores later bytes. SeaCad
Strict rejects a non-empty tail with `DXF-E0204` because the public DXF
structure defines EOF as the file terminator. Compatible preserves the exact
tail as opaque source bytes and emits `DXF-W0204`; it never parses or discards
the tail.

Two approved Compatible recoveries intentionally exceed AutoCAD behavior:
horizontal padding around an otherwise exact EOF value emits `DXF-W0202`, and
a fully paired stream without EOF emits `DXF-W0203`. Both make the raw document
inspect/verbatim-only. Case-folding is not performed. `minimal_eof` and
`early_eof_then_section` mix section/header validity with EOF placement, so
their failures are recorded but deferred to M4 semantics rather than enforced
by M3 framing.

## Normative references

- [Autodesk general DXF structure](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-D939EA11-0CEC-4636-91A8-756640A031D3.htm)
- [Autodesk group codes in numerical order](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm)
- [Autodesk SAFEMODE](https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-Core/files/GUID-23D32BD8-2F94-4463-9618-055CA798D465.htm)
