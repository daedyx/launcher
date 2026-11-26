# Baut das Setup des Daedyx Launchers (NSIS, pro Benutzer) und den Katalog-Eintrag dazu.
#   scripts\build-setup.ps1 -Notes "Was neu ist" -BaseUrl https://github.com/daedyx/releases/releases/download/launcher-v0.2.0/
# Ergebnis in dist\<Version>\: Daedyx-Launcher_<Version>_x64-setup.exe und launcher-catalog-entry.json
# (gleiche Form wie catalog-entry.json von Desk; die Pipeline trägt sie mit
#  `daedyx-catalog add-release --app launcher --entry launcher-catalog-entry.json` in den Katalog ein).
param(
  [string]$Version,                      # Standard: Version aus Cargo.toml
  [string]$Notes = "",
  [string]$BaseUrl = "",                 # ohne Angabe: relative Adresse packages/<Datei> (lokaler Katalogordner)
  [ValidateSet("release","debug")][string]$Configuration = "release",
  [switch]$SkipBuild,                    # Programme nicht neu bauen (Proben)
  [switch]$RequireRealKey                # bricht ab, solange assets\catalog.pub ein Platzhalter ist
)
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo
if (-not $Version) {
  $Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
}
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Ungültige Version: $Version" }
if ($BaseUrl -and -not ($BaseUrl -match '^https://' -or $BaseUrl -match '^http://127\.0\.0\.1[:/]')) {
  throw "BaseUrl muss mit https:// beginnen (lokal: http://127.0.0.1)"
}
$env:Path += ";$env:USERPROFILE\.cargo\bin"
# Der Launcher kennt seine Version nur aus Cargo.toml (Selbst-Update vergleicht sie mit dem Katalog):
# Setup-Version und eingebaute Version müssen gleich sein, sonst bietet er sich nach der Installation selbst ein Update an.
if (-not $SkipBuild) {
  $cargoVersion = (Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
  if ($cargoVersion -ne $Version) { throw "Version $Version passt nicht zu Cargo.toml ($cargoVersion). Zuerst scripts\ci\set-version.ps1 -Version $Version." }
}

if ($RequireRealKey) {
  # Seit U3 prüft der Launcher keinen Katalog mehr. Der öffentliche Schlüssel dient nur noch publish.ps1, das die
  # Brücke 0.2.0 ein letztes Mal in den alten Katalog einträgt (plan-ohne-secrets 3, Weg A).
  $key = Get-Content "crates\launcher-core\assets\catalog.pub" -Raw
  if ($key -notmatch 'minisign public key: [0-9A-F]{16}\s+RW[A-Za-z0-9+/=]{50,}') { throw "In crates\launcher-core\assets\catalog.pub steht noch kein echter öffentlicher Schlüssel." }
}
$flag = if ($Configuration -eq "release") { "--release" } else { "" }
if (-not $SkipBuild) {
  cargo build $flag -p launcher-ui -p launcher-cli
  if ($LASTEXITCODE -ne 0) { throw "Bauen fehlgeschlagen" }
}
$bin = Join-Path $repo "target\$Configuration"
foreach ($f in "daedyx-launcher.exe","daedyx.exe") {
  if (-not (Test-Path (Join-Path $bin $f))) { throw "$f fehlt in $bin" }
}

$makensis = @("$env:LOCALAPPDATA\tauri\NSIS\makensis.exe","${env:ProgramFiles(x86)}\NSIS\makensis.exe","$env:ProgramFiles\NSIS\makensis.exe") |
  Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $makensis) { $makensis = (Get-Command makensis -ErrorAction SilentlyContinue).Source }
if (-not $makensis) { throw "makensis nicht gefunden (NSIS installieren)" }

$out = Join-Path $repo "dist\$Version"
New-Item -ItemType Directory -Force $out | Out-Null
$name = "Daedyx-Launcher_${Version}_x64-setup.exe"
$setup = Join-Path $out $name
& $makensis /V2 "/DVERSION=$Version" "/DBINDIR=$bin" "/DOUTFILE=$setup" (Join-Path $repo "installer\launcher.nsi")
if ($LASTEXITCODE -ne 0) { throw "makensis fehlgeschlagen" }

$hash = (Get-FileHash $setup -Algorithm SHA256).Hash.ToLower()
$size = (Get-Item $setup).Length
$url = if ($BaseUrl) { ($BaseUrl.TrimEnd('/') + "/" + $name) } else { "packages/$name" }
$entry = [ordered]@{
  version = $Version
  date = (Get-Date -Format "yyyy-MM-dd")
  notes = $Notes
  packages = [ordered]@{ "windows-x64" = [ordered]@{ url = $url; sha256 = $hash; size = $size; kind = "nsis-silent" } }
}
# UTF-8 ohne BOM, wie Node es schreibt
[IO.File]::WriteAllText((Join-Path $out "launcher-catalog-entry.json"), ($entry | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding($false)))
Write-Output "Setup:   $setup ($size Bytes)"
Write-Output "Eintrag: $(Join-Path $out 'launcher-catalog-entry.json')"
Write-Output "SHA-256: $hash"
