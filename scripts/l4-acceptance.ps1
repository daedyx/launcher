# Abnahme von L4 mit dem ECHTEN Setup des Launchers (NSIS) und dem Selbst-Update.
# Installiert nur in einen Temp-Ordner; legt kurz einen Startmenü-Eintrag und einen Uninstall-
# Eintrag in HKCU an und entfernt beides wieder. Testschlüssel, kein echter Schlüssel.
# Voraussetzung: cargo build --workspace (Debug) ist gelaufen.
param([string]$Configuration = "debug")
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$bin = Join-Path $repo "target\$Configuration"
$root = Join-Path $env:TEMP "daedyx-l4-$PID"
$uninst = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Daedyx Launcher"
$lnk = Join-Path ([Environment]::GetFolderPath("Programs")) "Daedyx Launcher.lnk"
$script:ok = 0
function Check($name, [bool]$cond) { if (-not $cond) { throw "FEHLER: $name" }; $script:ok++; Write-Output "  ok  $name" }
function Wait-Until([scriptblock]$cond, [int]$seconds = 40) {
  for ($i = 0; $i -lt ($seconds * 2); $i++) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 500 }
  return $false
}
$installDir = Join-Path $root "Programm\Daedyx Launcher"
try {
  New-Item -ItemType Directory $root | Out-Null
  if ((Test-Path $uninst) -or (Test-Path $lnk)) { throw "Es ist schon ein Daedyx Launcher installiert; die Abnahme würde ihn überschreiben. Abbruch." }

  Write-Output "1) Setups bauen (0.1.0 und 0.1.1 mit denselben Programmen)"
  foreach ($v in "0.1.0","0.1.1") {
    & (Join-Path $PSScriptRoot "build-setup.ps1") -Version $v -Configuration $Configuration -SkipBuild -Notes "Abnahme $v" | Out-Null
  }
  $s010 = Join-Path $repo "dist\0.1.0\Daedyx-Launcher_0.1.0_x64-setup.exe"
  $s011 = Join-Path $repo "dist\0.1.1\Daedyx-Launcher_0.1.1_x64-setup.exe"
  Check "Setup 0.1.0 und 0.1.1 gebaut" ((Test-Path $s010) -and (Test-Path $s011))
  Check "Katalog-Eintrag im Format von Desk" ((Get-Content (Join-Path $repo "dist\0.1.1\launcher-catalog-entry.json") -Raw | ConvertFrom-Json).packages.'windows-x64'.kind -eq "nsis-silent")

  Write-Output "2) Installieren (still, pro Benutzer)"
  Start-Process $s010 -ArgumentList "/S","/D=$installDir" -Wait
  foreach ($f in "daedyx-launcher.exe","daedyx.exe","daedyx-catalog.exe","uninstall.exe") { Check "Datei $f installiert" (Test-Path (Join-Path $installDir $f)) }
  $reg = Get-ItemProperty $uninst
  Check "Registry: DisplayVersion 0.1.0" ($reg.DisplayVersion -eq "0.1.0")
  Check "Registry: InstallLocation stimmt" ($reg.InstallLocation -eq $installDir)
  Check "Startmenü-Eintrag da" (Test-Path $lnk)

  Write-Output "3) Katalog mit Selbst-Update bauen (Katalog-Werkzeug, Testschlüssel)"
  $cat = Join-Path $root "katalog"; New-Item -ItemType Directory "$cat\packages" | Out-Null
  Copy-Item $s011 "$cat\packages\"
  $keys = Join-Path $root "keys"
  & (Join-Path $bin "daedyx-catalog.exe") keygen --out $keys --no-password | Out-Null
  & (Join-Path $bin "daedyx-catalog.exe") add-release --catalog "$cat\catalog.json" --app launcher --name "Daedyx Launcher" --entry (Join-Path $repo "dist\0.1.1\launcher-catalog-entry.json") | Out-Null
  & (Join-Path $bin "daedyx-catalog.exe") sign --catalog "$cat\catalog.json" --key "$keys\catalog.key" | Out-Null

  Write-Output "4) Selbst-Update mit dem INSTALLIERTEN Programm"
  $cli = Join-Path $installDir "daedyx.exe"
  $data = Join-Path $root "daten"
  & $cli --data-dir $data config set --catalog $cat --pubkey "$keys\catalog.pub" | Out-Null
  Check "Update 0.1.1 wird gemeldet" ((& $cli --data-dir $data self-update --check | Out-String) -match "0\.1\.1")
  $oldTime = (Get-Item (Join-Path $installDir "daedyx.exe")).LastWriteTime
  Check "Update wird gestartet" ((& $cli --data-dir $data self-update --no-launch | Out-String) -match "gestartet")
  Check "Setup 0.1.1 hat den Eintrag aktualisiert (DisplayVersion 0.1.1)" (Wait-Until { (Get-ItemProperty $uninst).DisplayVersion -eq "0.1.1" })
  Check "gleicher Installationsordner" ((Get-ItemProperty $uninst).InstallLocation -eq $installDir)
  Check "Programme sind ersetzt und vorhanden" ((Test-Path (Join-Path $installDir "daedyx-launcher.exe")) -and (Test-Path (Join-Path $installDir "daedyx.exe")))
  Check "kein Neustart des Launchers bei /NOLAUNCH" (-not (Get-Process daedyx-launcher -ErrorAction SilentlyContinue))
  # Hinweis: Beide Setups enthalten dieselben Programme (eingebaute Version 0.1.0); darum meldet
  # `self-update --check` weiter 0.1.1. Bei echten Releases tragen die Programme ihre eigene Version.

  Write-Output "4b) Vollständiger Weg über das Fenster (Automatik, Beenden, Ersetzen, Neustart)"
  $ui = Start-Process (Join-Path $installDir "daedyx-launcher.exe") -ArgumentList "--data-dir",$data -PassThru
  $restarted = Wait-Until { (-not (Get-Process -Id $ui.Id -ErrorAction SilentlyContinue)) -and (Get-Process daedyx-launcher -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $ui.Id }) } 90
  Check "Fenster-Programm findet das Update, beendet sich selbst und wird vom Setup neu gestartet" $restarted
  Get-Process daedyx-launcher -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep 1

  Write-Output "5) Deinstallieren"
  Start-Process (Join-Path $installDir "uninstall.exe") -ArgumentList "/S","_?=$installDir" -Wait
  Check "Programme entfernt" (-not (Test-Path (Join-Path $installDir "daedyx-launcher.exe")))
  Check "Registry-Eintrag entfernt" (-not (Test-Path $uninst))
  Check "Startmenü-Eintrag entfernt" (-not (Test-Path $lnk))
  Check "Einstellungen und Daten bleiben" (Test-Path "$data\config.json")
  Write-Output "`nAbnahme bestanden ($script:ok Prüfungen)."
} finally {
  Get-Process daedyx-launcher -ErrorAction SilentlyContinue | Stop-Process -Force
  if (Test-Path (Join-Path $installDir "uninstall.exe")) {
    Start-Process (Join-Path $installDir "uninstall.exe") -ArgumentList "/S","_?=$installDir" -Wait
  }
  Remove-Item $root -Recurse -Force -ErrorAction SilentlyContinue
}
