[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $InnoCompilerPath,
    [Parameter(Mandatory)] [string] $PublishDirectory
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repoRoot = Split-Path -Parent $PSScriptRoot
$InnoCompilerPath = (Resolve-Path -LiteralPath $InnoCompilerPath).Path
$PublishDirectory = (Resolve-Path -LiteralPath $PublishDirectory).Path
$testId = [guid]::NewGuid().ToString()
$testRoot = Join-Path $repoRoot "artifacts/windows-installer-tests/$testId"
$installDirectory = Join-Path $testRoot 'installed'
New-Item -ItemType Directory -Path $testRoot -Force | Out-Null

# Exercise the real payload and installer logic with a unique uninstall key,
# protocol and shortcut names. Never install over the user's SunCode instance.
$source = Get-Content -LiteralPath (Join-Path $repoRoot 'apps/desktop-avalonia/packaging/windows/SunCode.iss') -Raw
$source = $source.Replace('8068245C-BA48-47BE-A55D-C8C7F484C506', $testId)
$source = $source.Replace('Software\Classes\suncode', "Software\Classes\suncode-installer-test-$testId")
$source = $source.Replace('Name: "{autoprograms}\SunCode"', "Name: `"{autoprograms}\SunCode Installer Test $testId`"")
$source = $source.Replace('Name: "{autodesktop}\SunCode"', "Name: `"{autodesktop}\SunCode Installer Test $testId`"")
$source = $source.Replace('AppName=SunCode', "AppName=SunCode Installer Test $testId")
$source = $source.Replace('UninstallDisplayName=SunCode', "UninstallDisplayName=SunCode Installer Test $testId")
$source = $source.Replace('SetupIconFile=..\..\Assets\suncode.ico', "SetupIconFile=$repoRoot\apps\desktop-avalonia\Assets\suncode.ico")
$source = $source.Replace('Source: "..\..\..\..\LICENSE"', "Source: `"$repoRoot\LICENSE`"")
$testScript = Join-Path $testRoot 'test.iss'
[IO.File]::WriteAllText($testScript, $source)

$currentVersion = [Diagnostics.FileVersionInfo]::GetVersionInfo((Join-Path $PublishDirectory 'SunCode.exe')).ProductVersion.Split('+')[0]
if ([version]$currentVersion -le [version]'0.0.0') { throw 'Downgrade testing needs a nonzero Desktop version.' }
foreach ($entry in @(@('current', $currentVersion), @('older', '0.0.0'))) {
    & $InnoCompilerPath /Q /NC "/DAppVersion=$($entry[1])" '/DAppTarget=win-x64' `
        "/DPublishDir=$PublishDirectory" "/O$testRoot" "/F$($entry[0])" $testScript
    if ($LASTEXITCODE -ne 0) { throw 'Test installer compilation failed.' }
}

$uninstallKey = "Software\Microsoft\Windows\CurrentVersion\Uninstall\{$testId}_is1"
$installerKey = "Software\SunCode\Installer\$testId"
$protocolKey = "Software\Classes\suncode-installer-test-$testId"
function Invoke-TestSetup([string] $Name, [bool] $ExpectSuccess) {
    $process = Start-Process -FilePath (Join-Path $testRoot "$Name.exe") -ArgumentList @(
        '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-', "/DIR=`"$installDirectory`"",
        "/LOG=`"$testRoot\$Name.log`""
    ) -WindowStyle Hidden -PassThru -Wait
    if (($process.ExitCode -eq 0) -ne $ExpectSuccess) {
        throw "Unexpected $Name installer result: $($process.ExitCode). See $testRoot"
    }
}

$uninstaller = Join-Path $installDirectory 'unins000.exe'
try {
    Invoke-TestSetup 'current' $true
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($uninstallKey)
    if (-not $key) { throw 'Installed application is absent from uninstall registration.' }
    $key.Dispose()
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($installerKey)
    if (-not $key) { throw 'Installer identity is absent.' }
    try { if ($key.GetValue('SunCodeTarget') -ne 'win-x64') { throw 'Installed architecture marker is incorrect.' } }
    finally { $key.Dispose() }
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("$protocolKey\shell\open\command")
    if (-not $key) { throw 'Activation protocol was not registered.' }
    try {
        if ($key.GetValue('') -ne "`"$installDirectory\SunCode.exe`" `"%1`"") { throw 'Activation command quoting is incorrect.' }
    }
    finally { $key.Dispose() }

    foreach ($file in @('SunCode.exe', 'suncode_agent.dll', 'coreclr.dll', 'vcruntime140.dll')) {
        if ((Get-FileHash -LiteralPath (Join-Path $installDirectory $file)).Hash -ne
            (Get-FileHash -LiteralPath (Join-Path $PublishDirectory $file)).Hash) {
            throw "Installed payload does not match: $file"
        }
    }

    # Reinstallation repairs a missing native library without removing files
    # that belong to the user. A newer install must refuse the old setup.
    $sentinel = Join-Path $installDirectory 'user-file.txt'
    [IO.File]::WriteAllText($sentinel, 'keep')
    Remove-Item -LiteralPath (Join-Path $installDirectory 'suncode_agent.dll')
    Invoke-TestSetup 'current' $true
    if (-not (Test-Path -LiteralPath (Join-Path $installDirectory 'suncode_agent.dll'))) { throw 'Reinstall did not repair the native library.' }
    Invoke-TestSetup 'older' $false
    if ((Get-Content -LiteralPath (Join-Path $testRoot 'older.log') -Raw) -notmatch 'newer version of SunCode') {
        throw 'The older setup failed for a reason other than the downgrade guard.'
    }
    Remove-Item -LiteralPath (Join-Path $installDirectory 'SunCode.exe')
    Invoke-TestSetup 'older' $false
    if ((Get-Content -LiteralPath (Join-Path $testRoot 'older.log') -Raw) -notmatch 'newer version of SunCode') {
        throw 'Downgrade protection failed with a missing installed executable.'
    }
    Invoke-TestSetup 'current' $true

    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($installerKey, $true)
    try { $key.SetValue('SunCodeTarget', 'win-arm64') }
    finally { $key.Dispose() }
    Invoke-TestSetup 'current' $false
    if ((Get-Content -LiteralPath (Join-Path $testRoot 'current.log') -Raw) -notmatch 'before changing architecture') {
        throw 'The setup failed for a reason other than the architecture guard.'
    }
}
finally {
    if (Test-Path -LiteralPath $uninstaller) {
        $process = Start-Process -FilePath $uninstaller -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -WindowStyle Hidden -PassThru -Wait
        if ($process.ExitCode -ne 0) { throw "Test uninstall failed: $($process.ExitCode). See $testRoot" }
    }
}
foreach ($path in @($uninstallKey, $installerKey, $protocolKey)) {
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($path)
    if ($key) { $key.Dispose(); throw "Uninstall left a test registration: $path" }
}
if (Test-Path -LiteralPath (Join-Path $installDirectory 'SunCode.exe')) { throw 'Uninstall left an installer-owned file.' }
if ((Get-Content -LiteralPath $sentinel -Raw) -ne 'keep') { throw 'Uninstall removed an unrelated file.' }
Write-Host 'Passed: install, payload integrity, reinstall repair, downgrade refusal (including missing executable), architecture refusal, uninstall, unrelated-file preservation.'
