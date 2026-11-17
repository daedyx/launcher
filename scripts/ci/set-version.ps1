# Schreibt die Version in Cargo.toml ([workspace.package]); alle Programme des Launchers erben sie.
# Der Launcher vergleicht beim Selbst-Update seine eingebaute Version (aus Cargo.toml) mit dem Katalog.
param([Parameter(Mandatory)][string]$Version)
$ErrorActionPreference = "Stop"
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Ungültige Version: $Version (erwartet x.y.z)" }
$file = Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) "Cargo.toml"
$text = [IO.File]::ReadAllText($file)
$new = [regex]::Replace($text, '(?m)^version = "[^"]+"', "version = `"$Version`"", 1)
if ($new -notmatch "(?m)^version = `"$([regex]::Escape($Version))`"") { throw "Keine Versionszeile in Cargo.toml gefunden." }
[IO.File]::WriteAllText($file, $new, (New-Object Text.UTF8Encoding $false))
Write-Output "Cargo.toml: Version $Version"
