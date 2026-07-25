# M1 legacy DXF audit artifacts

These artifacts describe, but do not contain, assets from the read-only legacy
workspace identified as `legacy-cad-2026-07-23`.

- `legacy-dxf-assets.csv` is the complete file-level inventory selected by the
  M1 rules. Hashes cover the exact bytes observed during the audit.
- `legacy-dxf-fixtures.csv` is the 15-file CAD fixture subset.
- `summary.json` is the stable `seacad-legacy-dxf-audit/v1` receipt and binds
  the complete manifest by SHA-256.

`direct_transfer_approved=false` is intentional. The legacy repository has no
root license, most scoped assets are untracked, and two real drawings are
private corpus inputs. A future approval may add a new SeaCad-authored fixture
or re-expressed test; it must not silently change these historical receipts.

Regenerate the inventory with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass `
  -File tools/audit-legacy-dxf.ps1 `
  -LegacyRoot <read-only-legacy-workspace>
```

The script records only a source ID and relative paths. It does not copy legacy
assets or write into the legacy workspace.
