[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$LegacyRoot,

    [string]$OutputRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$sourceId = "legacy-cad-2026-07-23"
$contract = "seacad-legacy-dxf-audit/v1"
$legacy = (Resolve-Path -LiteralPath $LegacyRoot).Path.TrimEnd("\")
$legacyGitPath = $legacy.Replace("\", "/")
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $PSScriptRoot "../audits/m1"
}
$output = [System.IO.Path]::GetFullPath($OutputRoot)

function Convert-ToRelativePath {
    param([string]$FullName)

    return $FullName.Substring($legacy.Length + 1).Replace("\", "/")
}

function Test-IsExcludedPath {
    param([string]$RelativePath)

    return $RelativePath -match "(^|/)(\.git|target|scratch|bin|ErrorReports)(/|$)"
}

function Test-IsScopedAsset {
    param([string]$RelativePath)

    $path = $RelativePath.ToLowerInvariant()
    $name = [System.IO.Path]::GetFileName($path)
    $extension = [System.IO.Path]::GetExtension($path)

    if ($path.StartsWith("crates/cad-dxf/") -or
        $path.StartsWith("crates/cad-dxf-lossless/") -or
        $path.StartsWith("cad-file/dxf/")) {
        return $true
    }

    if ($path.StartsWith("crates/cad-re/")) {
        return $path.Contains("dxf") -or
            $path -match "^crates/cad-re/src/(corpus|evidence_bundle|native_receipt|sha256|source_path)" -or
            $path -match "^crates/cad-re/tests/evidence_bundle" -or
            $name -eq "cargo.toml" -or
            $path -eq "crates/cad-re/src/lib.rs"
    }

    if ($path.StartsWith("crates/cad-io/")) {
        return $name.StartsWith("dxf") -or
            $name.StartsWith("native_") -or
            $name.StartsWith("lwpolyline_") -or
            $name -eq "native_bytes_reader_tests.rs" -or
            $extension -in ".dxf", ".dxb"
    }

    if ($path.StartsWith("docs/")) {
        return $name.Contains("dxf") -or $name -eq "support_matrix.md"
    }

    if ($path.StartsWith("public-roadmaps/")) {
        return $path.Contains("dxf")
    }

    if ($path.StartsWith("tools/")) {
        return $name.Contains("dxf") -or
            $name.Contains("autocad") -or
            $name.Contains("accoreconsole") -or
            $name.Contains("oda")
    }

    if ($path.StartsWith("fuzz/")) {
        return $path.Contains("dxf") -or
            $name -in "cargo.toml", "cargo.lock", "support.rs"
    }

    return $extension -in ".dxf", ".dxb"
}

function Get-AssetKind {
    param([string]$RelativePath)

    $path = $RelativePath.ToLowerInvariant()
    $extension = [System.IO.Path]::GetExtension($path)

    if ($extension -in ".dxf", ".dxb") { return "cad_fixture" }
    if ($path.StartsWith("fuzz/corpus/")) { return "fuzz_seed" }
    if ($path.EndsWith(".md")) { return "documentation" }
    if ($path.StartsWith("cad-file/dxf/")) { return "evidence" }
    if ($path.Contains("/tests/") -or $path.Contains("/fuzz_targets/")) { return "test" }
    if ($path.EndsWith(".rs") -and $path.Contains("/src/")) { return "production_code" }
    if ($path.StartsWith("tools/") -or $path.EndsWith(".ps1") -or $path.EndsWith(".lsp")) {
        return "tool"
    }
    return "metadata"
}

function Get-Disposition {
    param(
        [string]$RelativePath,
        [string]$Kind
    )

    $path = $RelativePath.ToLowerInvariant()
    if ($Kind -eq "cad_fixture" -and $path.Contains("/real_")) {
        return "private_corpus_only"
    }
    if ($Kind -eq "cad_fixture") { return "quarantine_pending_attestation" }
    if ($Kind -eq "fuzz_seed") { return "quarantine_generated_seed" }
    if ($Kind -eq "production_code") { return "rewrite_clean_room" }
    if ($Kind -eq "test") { return "reexpress_behavior" }
    if ($Kind -eq "tool") { return "rewrite_if_needed" }
    return "reference_only"
}

function Get-Provenance {
    param(
        [string]$RelativePath,
        [string]$Kind,
        [bool]$Tracked
    )

    $path = $RelativePath.ToLowerInvariant()
    if ($Kind -eq "cad_fixture" -and $path.Contains("/real_")) {
        return "user-cad-private"
    }
    if ($Kind -eq "fuzz_seed") { return "legacy-fuzz-output-unverified" }
    if ($Tracked) { return "legacy-git-history" }
    return "legacy-local-untracked"
}

function Get-Notes {
    param(
        [string]$RelativePath,
        [string]$Kind
    )

    $path = $RelativePath.ToLowerInvariant()
    if ($Kind -eq "cad_fixture" -and $path.Contains("/real_")) {
        return "Never commit to the SeaCad source repository."
    }
    if ($Kind -eq "cad_fixture") {
        return "Byte hash recorded; direct transfer is blocked until ownership is attested."
    }
    if ($Kind -eq "fuzz_seed") {
        return "Mutation history and seed license are not documented."
    }
    if ($path.StartsWith("crates/cad-io/")) {
        return "Legacy adapter context includes acadrust 0.4.0; do not port implementation."
    }
    if ($Kind -eq "production_code") {
        return "Use behavior and public specifications only; no line-by-line translation."
    }
    if ($Kind -eq "test") {
        return "Re-express the requirement in a new SeaCad-authored test."
    }
    return "No direct transfer approved by M1."
}

function New-StringSet {
    param([string[]]$Values)

    $set = @{}
    foreach ($value in $Values) {
        if (-not [string]::IsNullOrWhiteSpace($value)) {
            $set[$value.Replace("\", "/")] = $true
        }
    }
    return $set
}

function Get-GitLines {
    param([string[]]$Arguments)

    $result = @(& git -c "safe.directory=$legacyGitPath" -c core.safecrlf=false -C $legacy @Arguments)
    if ($LASTEXITCODE -ne 0) {
        throw "git failed: $($Arguments -join ' ')"
    }
    return $result
}

function Get-GroupedCounts {
    param(
        [object[]]$Rows,
        [string]$Property
    )

    $counts = [ordered]@{}
    foreach ($group in ($Rows | Group-Object -Property $Property | Sort-Object Name)) {
        $counts[$group.Name] = $group.Count
    }
    return $counts
}

function Write-Utf8NoBom {
    param(
        [string]$Path,
        [string[]]$Lines
    )

    $text = ($Lines -join "`n") + "`n"
    [System.IO.File]::WriteAllText($Path, $text, [System.Text.UTF8Encoding]::new($false))
}

$candidateRoots = @(
    "crates/cad-dxf",
    "crates/cad-dxf-lossless",
    "crates/cad-re",
    "crates/cad-io",
    "cad-file/dxf",
    "docs",
    "public-roadmaps",
    "tools",
    "fuzz",
    "tests/fixtures"
)

$assetsByPath = @{}
foreach ($candidateRoot in $candidateRoots) {
    $fullRoot = Join-Path $legacy $candidateRoot
    if (-not (Test-Path -LiteralPath $fullRoot -PathType Container)) { continue }
    foreach ($file in (Get-ChildItem -LiteralPath $fullRoot -Recurse -File)) {
        $relativePath = Convert-ToRelativePath $file.FullName
        if (-not (Test-IsExcludedPath $relativePath) -and (Test-IsScopedAsset $relativePath)) {
            $assetsByPath[$relativePath] = $file
        }
    }
}

$tracked = New-StringSet (Get-GitLines @("ls-files"))
$modified = New-StringSet (Get-GitLines @("diff", "--name-only"))
$staged = New-StringSet (Get-GitLines @("diff", "--cached", "--name-only"))
$legacyHead = [string](Get-GitLines @("rev-parse", "HEAD") | Select-Object -First 1)

$paths = @($assetsByPath.Keys)
[Array]::Sort($paths, [System.StringComparer]::Ordinal)
$rows = @()
foreach ($relativePath in $paths) {
    $file = $assetsByPath[$relativePath]
    $isTracked = $tracked.ContainsKey($relativePath)
    $gitState = if (-not $isTracked) {
        "untracked"
    } elseif ($staged.ContainsKey($relativePath)) {
        "tracked_staged"
    } elseif ($modified.ContainsKey($relativePath)) {
        "tracked_modified"
    } else {
        "tracked_clean"
    }
    $kind = Get-AssetKind $relativePath
    $disposition = Get-Disposition $relativePath $kind
    $rows += [pscustomobject][ordered]@{
        source_id = $sourceId
        relative_path = $relativePath
        kind = $kind
        bytes = $file.Length
        sha256 = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        git_state = $gitState
        provenance = Get-Provenance $relativePath $kind $isTracked
        license_status = "unverified-no-repo-license"
        disposition = $disposition
        direct_transfer_approved = "false"
        notes = Get-Notes $relativePath $kind
    }
}

[System.IO.Directory]::CreateDirectory($output) | Out-Null
$manifestPath = Join-Path $output "legacy-dxf-assets.csv"
$fixturePath = Join-Path $output "legacy-dxf-fixtures.csv"
Write-Utf8NoBom $manifestPath @($rows | ConvertTo-Csv -NoTypeInformation)
$fixtureRows = @($rows | Where-Object kind -eq "cad_fixture")
Write-Utf8NoBom $fixturePath @($fixtureRows | ConvertTo-Csv -NoTypeInformation)

$summary = [ordered]@{
    contract = $contract
    schema_version = 1
    source_id = $sourceId
    legacy_git_head = $legacyHead
    selection = "DXF parser, lossless core, DXF adapters/tests/evidence/docs/tools, CAD fixtures, and DXF fuzz assets; build outputs excluded"
    asset_count = $rows.Count
    total_bytes = ($rows | Measure-Object -Property bytes -Sum).Sum
    manifest_sha256 = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
    cad_fixture_count = $fixtureRows.Count
    direct_transfer_approved_count = @($rows | Where-Object direct_transfer_approved -eq "true").Count
    counts_by_kind = Get-GroupedCounts $rows "kind"
    counts_by_git_state = Get-GroupedCounts $rows "git_state"
    counts_by_disposition = Get-GroupedCounts $rows "disposition"
}
$summaryLines = @($summary | ConvertTo-Json -Depth 5)
Write-Utf8NoBom (Join-Path $output "summary.json") $summaryLines

Write-Output "asset_count=$($summary.asset_count)"
Write-Output "total_bytes=$($summary.total_bytes)"
Write-Output "cad_fixture_count=$($summary.cad_fixture_count)"
Write-Output "direct_transfer_approved_count=$($summary.direct_transfer_approved_count)"
Write-Output "manifest_sha256=$($summary.manifest_sha256)"
