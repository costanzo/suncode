[CmdletBinding()]
param(
    [ValidateSet('win-x64', 'win-arm64')]
    [string] $RuntimeIdentifier = 'win-x64',
    [string] $InnoCompilerPath,
    [string] $DistributionConfiguration,
    [string] $VcRuntimeDirectory
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'Windows packaging must run on Windows.'
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$project = Join-Path $repoRoot 'apps/desktop-avalonia/SunCode.Desktop.csproj'
$installerScript = Join-Path $repoRoot 'apps/desktop-avalonia/packaging/windows/SunCode.iss'

if (-not $InnoCompilerPath) {
    $compilerCommand = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($compilerCommand) { $InnoCompilerPath = $compilerCommand.Source }
    else {
        foreach ($programRoot in @($env:ProgramFiles, ${env:ProgramFiles(x86)}, $env:LOCALAPPDATA)) {
            if (-not $programRoot) { continue }
            foreach ($compilerDirectory in @('Inno Setup 7', 'Inno Setup 6', 'Programs/Inno Setup 7', 'Programs/Inno Setup 6')) {
                $candidate = Join-Path $programRoot "$compilerDirectory/ISCC.exe"
                if (Test-Path -LiteralPath $candidate -PathType Leaf) {
                    $InnoCompilerPath = $candidate
                    break
                }
            }
            if ($InnoCompilerPath) { break }
        }
    }
}
if (-not $InnoCompilerPath -or -not (Test-Path -LiteralPath $InnoCompilerPath -PathType Leaf)) {
    throw 'Install Inno Setup 7 (recommended) or 6.3+, or pass -InnoCompilerPath with the full path to ISCC.exe.'
}
$InnoCompilerPath = (Resolve-Path -LiteralPath $InnoCompilerPath).Path

Get-Command dotnet.exe, cargo.exe -ErrorAction Stop | Out-Null
# The managed SDK currently builds the host Rust target. Do not accidentally
# ship an x64 native library beside an ARM64 .NET executable, or vice versa.
$rustInfo = & rustc.exe -vV
if ($LASTEXITCODE -ne 0) { throw 'Could not identify the Rust host target.' }
$requiredHost = if ($RuntimeIdentifier -eq 'win-arm64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
if ($rustInfo -notcontains "host: $requiredHost") {
    throw "$RuntimeIdentifier requires the $requiredHost Rust host toolchain. Cross-compilation is not supported by the current managed SDK build."
}

# Rust's MSVC DLL needs vcruntime140.dll even in a self-contained .NET
# publish. Bundle the redistributable app-local CRT, never DLLs from System32.
if (-not $VcRuntimeDirectory) {
    $vsWhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    if (Test-Path -LiteralPath $vsWhere -PathType Leaf) {
        $vsRoot = & $vsWhere -latest -products '*' -property installationPath
        if ($LASTEXITCODE -ne 0) { throw 'Could not locate the Visual C++ redistributable directory.' }
        if ($vsRoot) {
            $crtArchitecture = $RuntimeIdentifier.Substring(4)
            $crtDirectories = @(Get-ChildItem -Path (Join-Path $vsRoot "VC/Redist/MSVC/*/$crtArchitecture/Microsoft.VC*.CRT") -Directory |
                Sort-Object FullName -Descending)
            if ($crtDirectories.Count) { $VcRuntimeDirectory = $crtDirectories[0].FullName }
        }
    }
}
if (-not $VcRuntimeDirectory -or -not (Test-Path -LiteralPath (Join-Path $VcRuntimeDirectory 'vcruntime140.dll') -PathType Leaf)) {
    throw 'Install the Visual C++ redistributable build tools, or pass -VcRuntimeDirectory pointing to the matching Microsoft.VC*.CRT redistributable directory.'
}
$crtFiles = @(Get-ChildItem -LiteralPath $VcRuntimeDirectory -Filter '*.dll' -File)

[xml] $projectXml = Get-Content -LiteralPath $project -Raw
$versionNode = $projectXml.SelectSingleNode('/Project/PropertyGroup/Version')
$appVersion = $versionNode.InnerText
if ($appVersion -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' -or
    @($appVersion.Split('.') | Where-Object { [long]$_ -gt 65535 }).Count -gt 0) {
    throw 'Desktop Version must be MAJOR.MINOR.PATCH with components between 0 and 65535.'
}

$outputRoot = Join-Path $repoRoot "artifacts/windows/$appVersion/$RuntimeIdentifier"
$publishDirectory = Join-Path $outputRoot 'publish'
$installerDirectory = Join-Path $outputRoot 'installer'
$publishArguments = @(
    'publish', $project, '-c', 'Release', '-r', $RuntimeIdentifier,
    '--self-contained', 'true', '-o', $publishDirectory,
    '-p:PublishSingleFile=false', '-p:PublishTrimmed=false', '-p:SunCodeBundleCef=false'
)
if ($DistributionConfiguration) {
    $configurationPath = (Resolve-Path -LiteralPath $DistributionConfiguration).Path
    $publishArguments += "-p:SunCodeUpdateDistributionFile=$configurationPath"
}

function Assert-PeArchitecture([string] $Path, [int] $ExpectedMachine) {
    $stream = [IO.File]::OpenRead($Path)
    $reader = [IO.BinaryReader]::new($stream)
    try {
        if ($reader.ReadUInt16() -ne 0x5A4D) { throw "Not a PE file: $Path" }
        $stream.Position = 0x3C
        $peOffset = $reader.ReadInt32()
        if ($peOffset -lt 0 -or $peOffset -gt $stream.Length - 6) { throw "Invalid PE header: $Path" }
        $stream.Position = $peOffset
        if ($reader.ReadUInt32() -ne 0x00004550 -or $reader.ReadUInt16() -ne $ExpectedMachine) {
            throw "Unexpected native architecture in $Path"
        }
    }
    finally { $reader.Dispose() }
}

Push-Location $repoRoot
try {
    & dotnet.exe @publishArguments
    if ($LASTEXITCODE -ne 0) { throw "Desktop publish failed with exit code $LASTEXITCODE." }

    $expectedMachine = if ($RuntimeIdentifier -eq 'win-arm64') { 0xAA64 } else { 0x8664 }
    foreach ($nativeFile in @('SunCode.exe', 'suncode_agent.dll', 'coreclr.dll', 'hostfxr.dll')) {
        Assert-PeArchitecture (Join-Path $publishDirectory $nativeFile) $expectedMachine
    }
    foreach ($crtFile in $crtFiles) {
        Assert-PeArchitecture $crtFile.FullName $expectedMachine
        Copy-Item -LiteralPath $crtFile.FullName -Destination $publishDirectory -Force
    }
    $publishedVersion = [Diagnostics.FileVersionInfo]::GetVersionInfo((Join-Path $publishDirectory 'SunCode.exe')).ProductVersion.Split('+')[0]
    if ($publishedVersion -ne $appVersion) { throw 'Published Desktop version does not match the installer version.' }

    New-Item -ItemType Directory -Path $installerDirectory -Force | Out-Null
    $outputBaseName = "SunCode-$appVersion-$RuntimeIdentifier-Setup"
    & $InnoCompilerPath /Q "/DAppVersion=$appVersion" "/DAppTarget=$RuntimeIdentifier" `
        "/DPublishDir=$publishDirectory" "/O$installerDirectory" "/F$outputBaseName" $installerScript
    if ($LASTEXITCODE -ne 0) { throw "Inno Setup compilation failed with exit code $LASTEXITCODE." }

    $installer = Join-Path $installerDirectory "$outputBaseName.exe"
    if (-not (Test-Path -LiteralPath $installer -PathType Leaf)) { throw 'Inno Setup did not produce the expected installer.' }
    $digest = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText("$installer.sha256", "$digest  $outputBaseName.exe`n", [Text.UTF8Encoding]::new($false))
    Write-Host "Installer: $installer"
    Write-Host "SHA-256: $digest"
}
finally { Pop-Location }
