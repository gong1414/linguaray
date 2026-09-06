$ErrorActionPreference = 'Stop'
$payload = (Resolve-Path 'apps/desktop/flutter/build/windows/x64/runner/Release').Path
$dist = Join-Path $PWD 'apps/desktop/flutter/dist'
New-Item -ItemType Directory -Force $dist | Out-Null
$certificate = Join-Path $env:RUNNER_TEMP 'linguaray-signing.pfx'
$signed = $false
try {
  if ($env:WINDOWS_CERTIFICATE_BASE64 -or $env:WINDOWS_CERTIFICATE_PASSWORD) {
    if (-not $env:WINDOWS_CERTIFICATE_BASE64 -or -not $env:WINDOWS_CERTIFICATE_PASSWORD) {
      throw 'Incomplete Windows signing configuration'
    }
    [IO.File]::WriteAllBytes($certificate, [Convert]::FromBase64String($env:WINDOWS_CERTIFICATE_BASE64))
    $signTool = Get-ChildItem "${env:ProgramFiles(x86)}/Windows Kits/10/bin/*/x64/signtool.exe" |
      Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
    if (-not $signTool) { throw 'Windows SDK signtool.exe was not found' }
    $sign = {
      param([string]$Path)
      & $signTool sign /fd SHA256 /td SHA256 /tr https://timestamp.digicert.com `
        /f $certificate /p $env:WINDOWS_CERTIFICATE_PASSWORD $Path
      if ($LASTEXITCODE -ne 0) { throw "Signing failed: $Path" }
    }
    Get-ChildItem $payload -Recurse -File |
      Where-Object { $_.Extension -in '.exe', '.dll' } |
      ForEach-Object { & $sign $_.FullName }
    $signed = $true
  }
  $version = $env:RELEASE_TAG -replace '^v', ''
  $script = (Resolve-Path 'apps/desktop/flutter/windows/packaging/exe/LinguaRay.iss').Path
  $compiler = Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6/ISCC.exe'
  if (-not (Test-Path $compiler)) { $compiler = (Get-Command ISCC.exe -ErrorAction Stop).Source }
  & $compiler "/DSourceDir=$payload" "/DAppVersion=$version" "/O$dist" $script
  if ($LASTEXITCODE -ne 0) { throw 'Inno Setup compilation failed' }
  $installer = Join-Path $dist 'LinguaRay-windows-x64.exe'
  if ($signed) {
    & $sign $installer
    foreach ($file in @((Join-Path $payload 'linguaray.exe'), $installer)) {
      if ((Get-AuthenticodeSignature -LiteralPath $file).Status -ne 'Valid') {
        throw "Authenticode verification failed: $file"
      }
    }
  }
  @{ platformSigned = $signed } | ConvertTo-Json | Set-Content (Join-Path $dist 'windows-signing.json')
} finally {
  Remove-Item -Force -ErrorAction SilentlyContinue $certificate
}
