param(
    [Parameter(Mandatory = $true)]
    [string]$MappingPath,
    [string]$OutputPath
)

$ErrorActionPreference = 'Stop'
$expectedSourceSha256 = '7dcda2d5d2cfc5ddf43757d589a0106e020ef0d9b84de47f08e18297ae0fe1ec'
$expectedCanonicalSha256 = '5f038ab2832fc3b597115b3138480142d5dccc97a39edcf61567b0dff8385a84'
$expectedTableSha256 = 'd04a1a13d5f4706df6fa46394cda98a817570e774d601acd042e0fa57249f7ea'
$mappingSha256 = (
    Get-FileHash -Algorithm SHA256 -LiteralPath $MappingPath
).Hash.ToLowerInvariant()
if ($mappingSha256 -ne $expectedSourceSha256) {
    throw "The CP1361 source artifact does not match the frozen SHA-256."
}

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class SeaCadJohabNls
{
    private const uint CodePage = 1361;
    private const uint ErrorOnInvalid = 0x00000008;

    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    private static extern int MultiByteToWideChar(
        uint codePage,
        uint flags,
        byte[] input,
        int inputLength,
        [Out] char[] output,
        int outputLength);

    public static string Decode(byte[] input)
    {
        char[] output = new char[4];
        int written = MultiByteToWideChar(
            CodePage,
            ErrorOnInvalid,
            input,
            input.Length,
            output,
            output.Length);
        return written == 0 ? null : new string(output, 0, written);
    }
}
'@

function Parse-HexU16 {
    param([string]$Text)
    if (-not $Text.StartsWith('0x', [StringComparison]::OrdinalIgnoreCase)) {
        throw "Expected hexadecimal token, got '$Text'."
    }
    return [Convert]::ToUInt16($Text.Substring(2), 16)
}

function Add-Mapping {
    param(
        [Collections.Generic.Dictionary[uint16, uint16]]$Map,
        [uint16]$Code,
        [uint16]$Unicode
    )
    if ($Map.ContainsKey($Code)) {
        throw ('Duplicate mapping for 0x{0:X4}.' -f $Code)
    }
    $Map.Add($Code, $Unicode)
}

function Get-CanonicalHash {
    param([Collections.Generic.Dictionary[uint16, uint16]]$Map)
    $stream = [IO.MemoryStream]::new()
    try {
        foreach ($code in @($Map.Keys | Sort-Object)) {
            $unicode = $Map[$code]
            $record = [byte[]]@(
                [byte]($code -band 0xFF),
                [byte](($code -shr 8) -band 0xFF),
                [byte]($unicode -band 0xFF),
                [byte](($unicode -shr 8) -band 0xFF)
            )
            $stream.Write($record, 0, $record.Length)
        }
        $sha256 = [Security.Cryptography.SHA256]::Create()
        try {
            return ([BitConverter]::ToString(
                $sha256.ComputeHash($stream.ToArray())
            )).Replace('-', '').ToLowerInvariant()
        } finally {
            $sha256.Dispose()
        }
    } finally {
        $stream.Dispose()
    }
}

$official = [Collections.Generic.Dictionary[uint16, uint16]]::new()
$undefinedEudc = [Collections.Generic.HashSet[uint16]]::new()
$mode = ''
$remaining = 0
$lead = 0

:mappingLines foreach ($rawLine in Get-Content -LiteralPath $MappingPath -Encoding Default) {
    $data = ($rawLine -split ';', 2)[0].Trim()
    if ($data.Length -eq 0) {
        continue
    }
    $fields = $data -split '\s+'
    switch ($fields[0]) {
        'MBTABLE' {
            $mode = 'single'
            $remaining = [int]$fields[1]
            continue mappingLines
        }
        'DBCSTABLE' {
            $match = [regex]::Match(
                $rawLine,
                'LeadByte\s*=\s*0x([0-9A-Fa-f]{2})'
            )
            if (-not $match.Success) {
                throw "Missing lead-byte evidence in '$rawLine'."
            }
            $lead = [Convert]::ToUInt16($match.Groups[1].Value, 16)
            $mode = 'double'
            $remaining = [int]$fields[1]
            continue mappingLines
        }
        'WCTABLE' {
            break mappingLines
        }
    }

    if ($remaining -le 0) {
        continue
    }
    $left = Parse-HexU16 $fields[0]
    $unicode = Parse-HexU16 $fields[1]
    if ($mode -eq 'single') {
        Add-Mapping $official $left $unicode
        if ($rawLine -match 'Undefined\s*->\s*EUDC') {
            [void]$undefinedEudc.Add($left)
        }
    } elseif ($mode -eq 'double') {
        $code = [uint16](($lead -shl 8) -bor $left)
        Add-Mapping $official $code $unicode
    } else {
        throw "Mapping record outside a decode table."
    }
    $remaining--
}

$strictOfficial = [Collections.Generic.Dictionary[uint16, uint16]]::new()
foreach ($code in $official.Keys) {
    if (-not $undefinedEudc.Contains($code)) {
        $strictOfficial.Add($code, $official[$code])
    }
}

$nls = [Collections.Generic.Dictionary[uint16, uint16]]::new()
$multiOutputCount = 0
for ($code = 0; $code -le 0xFFFF; $code++) {
    if ($code -le 0xFF) {
        $bytes = [byte[]]@([byte]$code)
    } else {
        $bytes = [byte[]]@(
            [byte](($code -shr 8) -band 0xFF),
            [byte]($code -band 0xFF)
        )
    }
    $decoded = [SeaCadJohabNls]::Decode($bytes)
    if ($null -eq $decoded) {
        continue
    }
    if ($decoded.Length -ne 1) {
        $multiOutputCount++
        continue
    }
    $nls.Add([uint16]$code, [uint16][char]$decoded[0])
}

$missingFromNls = [Collections.Generic.List[string]]::new()
$extraInNls = [Collections.Generic.List[string]]::new()
$different = [Collections.Generic.List[string]]::new()
foreach ($code in $official.Keys) {
    if (-not $nls.ContainsKey($code)) {
        $missingFromNls.Add(('0x{0:X4}->U+{1:X4}' -f $code, $official[$code]))
    } elseif ($nls[$code] -ne $official[$code]) {
        $different.Add(
            ('0x{0:X4}:official=U+{1:X4},nls=U+{2:X4}' -f
                $code, $official[$code], $nls[$code])
        )
    }
}
foreach ($code in $nls.Keys) {
    if (-not $official.ContainsKey($code)) {
        $extraInNls.Add(('0x{0:X4}->U+{1:X4}' -f $code, $nls[$code]))
    }
}

$officialSingles = @($official.Keys | Where-Object { $_ -le 0xFF }).Count
$strictOfficialSingles = @(
    $strictOfficial.Keys | Where-Object { $_ -le 0xFF }
).Count
$nlsSingles = @($nls.Keys | Where-Object { $_ -le 0xFF }).Count
$strictCanonicalSha256 = Get-CanonicalHash $strictOfficial
$nlsCanonicalSha256 = Get-CanonicalHash $nls
if (
    $official.Count -ne 17395 -or
    $strictOfficial.Count -ne 17384 -or
    $undefinedEudc.Count -ne 11 -or
    $missingFromNls.Count -ne 11 -or
    $extraInNls.Count -ne 0 -or
    $different.Count -ne 0 -or
    $strictCanonicalSha256 -ne $expectedCanonicalSha256 -or
    $nlsCanonicalSha256 -ne $expectedCanonicalSha256
) {
    throw "CP1361 source and Windows NLS no longer match the frozen contract."
}

$tableHash = $null
$tableLength = 0
if ($OutputPath) {
    $table = [byte[]]::new(0x10000 * 2)
    foreach ($code in $strictOfficial.Keys) {
        $unicode = [uint32]$strictOfficial[$code]
        if ($unicode -eq 0xFFFF) {
            throw 'U+FFFF cannot use the plus-one table sentinel.'
        }
        $stored = $unicode + 1
        $index = [int]$code * 2
        $table[$index] = [byte]($stored -band 0xFF)
        $table[$index + 1] = [byte](($stored -shr 8) -band 0xFF)
    }
    [IO.File]::WriteAllBytes($OutputPath, $table)
    $tableLength = $table.Length
    $tableHash = (
        Get-FileHash -Algorithm SHA256 -LiteralPath $OutputPath
    ).Hash.ToLowerInvariant()
    if ($tableHash -ne $expectedTableSha256) {
        throw "Generated CP1361 table does not match the frozen SHA-256."
    }
}

$summary = [ordered]@{
    source_file = [IO.Path]::GetFileName($MappingPath)
    mapping_sha256 = $mappingSha256
    official_count = $official.Count
    official_single_count = $officialSingles
    official_double_count = $official.Count - $officialSingles
    official_canonical_sha256 = Get-CanonicalHash $official
    explicit_undefined_eudc_count = $undefinedEudc.Count
    strict_official_count = $strictOfficial.Count
    strict_official_single_count = $strictOfficialSingles
    strict_official_double_count = $strictOfficial.Count - $strictOfficialSingles
    strict_official_canonical_sha256 = $strictCanonicalSha256
    nls_count = $nls.Count
    nls_single_count = $nlsSingles
    nls_double_count = $nls.Count - $nlsSingles
    nls_canonical_sha256 = $nlsCanonicalSha256
    nls_multi_output_count = $multiOutputCount
    missing_from_nls_count = $missingFromNls.Count
    extra_in_nls_count = $extraInNls.Count
    value_mismatch_count = $different.Count
    missing_from_nls_first = @($missingFromNls | Select-Object -First 20)
    extra_in_nls_first = @($extraInNls | Select-Object -First 20)
    value_mismatch_first = @($different | Select-Object -First 20)
    output_artifact = if ($OutputPath) {
        [IO.Path]::GetFileName($OutputPath)
    } else {
        $null
    }
    table_length = $tableLength
    table_sha256 = $tableHash
}

$summary | ConvertTo-Json -Depth 4
