$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$lockFile = Join-Path $repoRoot 'scripts/fonts.lock'
$fontDir = Join-Path $repoRoot 'apps/desktop-avalonia/Assets/fonts'
New-Item -ItemType Directory -Force -Path $fontDir | Out-Null

# Remove the old variable face so it cannot shadow the static faces when an
# existing checkout is rebuilt on Windows.
Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $fontDir 'NotoSansSC-wght.ttf')

foreach ($line in Get-Content $lockFile) {
    if ([string]::IsNullOrWhiteSpace($line) -or $line.StartsWith('#')) { continue }
    $parts = $line -split '\|', 4
    $name = $parts[0]
    $url = $parts[1]
    $expectedSha = $parts[2]
    $destination = Join-Path $fontDir $name

    $valid = $false
    if (Test-Path $destination) {
        $valid = (Get-FileHash -Algorithm SHA256 $destination).Hash.ToLowerInvariant() -eq $expectedSha
    }
    if ($valid) { continue }

    $temporary = "$destination.tmp"
    Remove-Item -Force -ErrorAction SilentlyContinue $temporary
    Write-Host "prepare-fonts: downloading $name"
    Invoke-WebRequest -Uri $url -OutFile $temporary
    $actualSha = (Get-FileHash -Algorithm SHA256 $temporary).Hash.ToLowerInvariant()
    if ($actualSha -ne $expectedSha) {
        Remove-Item -Force -ErrorAction SilentlyContinue $temporary
        throw "prepare-fonts: SHA-256 mismatch for $name (expected $expectedSha, got $actualSha)"
    }
    Move-Item -Force $temporary $destination
}

Write-Host "prepare-fonts: verified fonts in $fontDir"
