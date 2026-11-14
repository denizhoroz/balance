# Builds the release exe, then the installer: dist\Balance-<version>-setup.exe.
# Usage (from anywhere): powershell -ExecutionPolicy Bypass -File installer\build.ps1
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot
$version = (Select-String -Path "$root\app\Cargo.toml" -Pattern '^version\s*=\s*"(.+)"').Matches[0].Groups[1].Value

Push-Location "$root\app"
try {
    cargo build --release
    if ($LASTEXITCODE) { throw "cargo build failed" }
} finally { Pop-Location }

$iscc = @("$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe", "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe") |
    Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $iscc) { throw "Inno Setup 6 (ISCC.exe) not found" }

& $iscc "/DAppVersion=$version" "$PSScriptRoot\balance.iss"
if ($LASTEXITCODE) { throw "ISCC failed" }
"Built $root\dist\Balance-$version-setup.exe"
