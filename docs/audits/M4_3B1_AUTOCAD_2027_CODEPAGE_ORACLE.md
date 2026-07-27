# M4.3b1 AutoCAD 2027 codepage oracle

Status: completed 2026-07-27

This receipt records isolated behavioral evidence for representative legacy
DXF text bytes. Autodesk output is not used to prove byte preservation, and a
single vector is not an exhaustive codepage conformance claim.

## Oracle identity and isolation

| Item | Receipt |
| --- | --- |
| Product | AutoCAD Core Console 2027 |
| File/product version | `26.0.60.0.0` |
| Executable SHA-256 | `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77` |
| Script SHA-256 | `45ee3f7f1470a3a04e0d4fecd4cee3a7162876bc42aa7b271cdd0f230e713453` |
| Locale | `en-US` |

Every input was generated in a temporary directory outside the repository.
Each process used a unique registry/user-data identity and this invocation
shape:

```text
accoreconsole /i <generated-copy> /s <reviewed-script> /l en-US
  /isolate SeaCadM43b1-<case> <isolated-user-data> /readonly /safemode
```

`/readonly` prevented source writes and `/safemode` disabled executable code
and drawing-directory autoload. The reviewed script ran `LIST` on all objects,
then discarded the imported drawing and quit. AutoCAD's UTF-16LE stdout was
decoded explicitly before scalar inspection. No executable, fixture, profile,
stdout log, or user path is committed.

## Synthetic input

Each case is a minimal AC1009 ASCII DXF with exact HEADER `$DWGCODEPAGE`, one
R12 `TEXT` entity, and exact terminal EOF. Only the declaration token and
group-code 1 text bytes vary.

| Case | Token | Source bytes | Input bytes | Input SHA-256 |
| --- | --- | --- | ---: | --- |
| `ansi_1252` | `ANSI_1252` | `E9` | 186 | `405ed788caed3496c8bdec74d2a10931872182aded3e70bc7bdb6e13fb2e3a30` |
| `ansi_1258` | `ANSI_1258` | `D0` | 186 | `588cb7c1489a8af7d5147272f8e54813ec6d03d68de75d620451e75cd172f14b` |
| `ansi_874` | `ANSI_874` | `A1` | 185 | `b99337955049c8d5a80b0e2d4a62f6c0ede1681168f7b5506d585c00e1f4ae60` |
| `ansi_932` | `ANSI_932` | `82 A0` | 186 | `20dc7d86e25ee380b8246c3174df83306c90724db723e007a3327c9d260ebca4` |
| `ansi_936` | `ANSI_936` | `C4 E3` | 186 | `4bef98858c0bc26eaaf711a863c3ce25268cced54e69c49fa6065f7813745933` |
| `ansi_949` | `ANSI_949` | `B0 A1` | 186 | `6d3f29cf67f7d22d17aa9f740aa18683322a40112ccc299c4c210874cf8865d8` |
| `ansi_950` | `ANSI_950` | `A4 40` | 186 | `5e92270b3bb04d956892d9833348b765b37fbb74b654a82403a11ffc49495414` |

## Results

All processes exited `0`, opened the DXF, selected exactly one `TEXT` entity,
and reported the expected decoded scalar.

| Case | AutoCAD text | Scalar | Stdout SHA-256 |
| --- | --- | --- | --- |
| `ansi_1252` | `é` | `U+00E9` | `60a6061d8eb17e66404ed6724ed9b182e7693507f8bcc3139f686bb463f8f5bd` |
| `ansi_1258` | `Đ` | `U+0110` | `92604e14bbb1f849c37db9d00154c6a7af95e22801c0abadf6adb7a5653ad330` |
| `ansi_874` | `ก` | `U+0E01` | `b74122cba84516e421cb919227a2fb127685d4bf5df5795415eecae850fe072a` |
| `ansi_932` | `あ` | `U+3042` | `6c170ef3286a21430642740a0c63cfcc55d2464e63276298afa1274af8c40289` |
| `ansi_936` | `你` | `U+4F60` | `5e230b74e9e6c04c05b7a9c05695a253be7926b82f12f2a9767995ae25dda2e0` |
| `ansi_949` | `가` | `U+AC00` | `9e3bb3a70d044d9d2e4e79c9bbaf8c2183ae826be351050c6067864301ec64fe` |
| `ansi_950` | `一` | `U+4E00` | `299b73d5472d46bfe81e06374440177170e8bd48b182a796df7e822a387a12d6` |

The evidence supports these token/vector pairs and agrees with SeaCad's named
WHATWG decoder mappings. Remaining registry entries are covered by primary
mapping-table vectors and SeaCad unit tests, not by an AutoCAD locale matrix.

## Primary references

- [Autodesk HEADER variables](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm)
- [Autodesk DXF string value storage](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm)
- [Microsoft code page identifiers](https://learn.microsoft.com/windows/win32/intl/code-page-identifiers)
- [WHATWG Encoding Standard](https://encoding.spec.whatwg.org/)
