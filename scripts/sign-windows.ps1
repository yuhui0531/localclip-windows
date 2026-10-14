param([Parameter(Mandatory=$true)][string]$File)
$ErrorActionPreference = 'Stop'
if (-not $env:CODESIGN_PFX_BASE64) {
    Write-Host 'UNSIGNED manual-test artifact: code-signing certificate is not configured.'
    exit 0
}
$cert = Join-Path $env:RUNNER_TEMP ('localclip-sign-' + [guid]::NewGuid() + '.pfx')
try {
    [IO.File]::WriteAllBytes($cert, [Convert]::FromBase64String($env:CODESIGN_PFX_BASE64))
    $tool = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe' | Sort-Object FullName -Descending | Select-Object -First 1
    if (-not $tool) { throw 'Windows signing tool unavailable' }
    & $tool.FullName sign /fd SHA256 /f $cert /p $env:CODESIGN_PFX_PASSWORD /tr http://timestamp.digicert.com /td SHA256 $File
    if ($LASTEXITCODE -ne 0) { throw 'Code signing failed' }
    & $tool.FullName verify /pa $File
    if ($LASTEXITCODE -ne 0) { throw 'Code signature verification failed' }
} finally { Remove-Item $cert -Force -ErrorAction SilentlyContinue }
