# Abnahme von L3 mit dem ECHTEN Windows-Setup von Daedyx Desk (Testschlüssel, Temp-Ordner).
# Installiert und entfernt Desk nur in Temp-Ordnern (das Setup trägt dabei kurz einen Uninstall-
# Eintrag in HKCU ein und entfernt ihn wieder). Desk selbst wird NIE gestartet: Es würde Daten des
# Benutzers übernehmen. „Läuft“ wird mit einem harmlosen Hilfsprogramm im Installationsordner geprüft.
param(
  [Parameter(Mandatory)][string]$Setup,
  [string]$Version = "0.1.0",
  [string]$Target = "D:\Programmierung\daedyx-launcher\target\debug"
)
$ErrorActionPreference = "Stop"
$daedyx = Join-Path $Target "daedyx.exe"
$tool   = Join-Path $Target "daedyx-catalog.exe"
$root   = Join-Path $env:TEMP "daedyx-l3-$PID"
$regKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Daedyx Desk"
$script:ok = 0
function Check($name, [bool]$cond) { if (-not $cond) { throw "FEHLER: $name" } ; $script:ok++; Write-Output "  ok  $name" }
function Dx {
  # Fehlerausgabe des Programms soll Text sein, kein PowerShell-Abbruch
  $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
  $r = & $daedyx --data-dir $data @args 2>&1 | Out-String
  $ErrorActionPreference = $old
  $r
}

try {
  New-Item -ItemType Directory $root | Out-Null
  $data = Join-Path $root "data"
  Write-Output "1) Katalog mit dem Katalog-Werkzeug bauen (Testschlüssel ohne Passwort)"
  $cat = Join-Path $root "catalog"; New-Item -ItemType Directory "$cat\packages" | Out-Null
  $pkg = "packages\Daedyx.Desk_${Version}_x64-setup.exe"
  Copy-Item $Setup (Join-Path $cat $pkg)
  $hash = (Get-FileHash (Join-Path $cat $pkg) -Algorithm SHA256).Hash.ToLower()
  $size = (Get-Item (Join-Path $cat $pkg)).Length
  $entry = Join-Path $root "catalog-entry.json"
  (@{ version = $Version; date = (Get-Date -Format "yyyy-MM-dd"); notes = "Abnahme"
      packages = @{ "windows-x64" = @{ url = ($pkg -replace "\\","/"); sha256 = $hash; size = $size; kind = "nsis-silent" } } } | ConvertTo-Json -Depth 5) | Set-Content $entry -Encoding utf8
  $keys = Join-Path $root "keys"
  & $tool keygen --out $keys --no-password | Out-Null
  & $tool add-release --catalog "$cat\catalog.json" --app desk --entry $entry --name "Daedyx Desk" --description "Mail, Kalender, Aufgaben" --registry-key "Daedyx Desk" --author "Daedyx" | Out-Null
  & $tool sign --catalog "$cat\catalog.json" --key "$keys\catalog.key" | Out-Null
  Check "Katalog signiert und gepruft" ((& $tool verify --catalog "$cat\catalog.json" --public "$keys\catalog.pub" 2>&1 | Out-String) -match "Signatur")

  Write-Output "2) Einstellung dauerhaft speichern, danach ohne Parameter arbeiten"
  Dx config set --catalog $cat --pubkey "$keys\catalog.pub" | Out-Null
  Check "config.json geschrieben" (Test-Path "$data\config.json")
  $list = Dx --json list | ConvertFrom-Json
  Check "desk im Katalog, nicht installiert" ($list[0].id -eq "desk" -and -not $list[0].installed)

  Write-Output "3) Installieren mit dem echten Setup (vom Launcher)"
  $out = Dx install desk
  Check "Installation gemeldet" ($out -match "installiert")
  $dir = Join-Path $data "apps\desk"
  $app = Get-Content "$dir\app.json" -Raw | ConvertFrom-Json
  Check "app.json mit Version $Version" ($app.version -eq $Version)
  Check "Startprogramm vorhanden ($($app.exe))" (Test-Path (Join-Path $dir $app.exe))
  $reg = Get-ItemProperty $regKey
  Check "Registry: DisplayVersion $Version" ($reg.DisplayVersion -eq $Version)
  Check "Registry: InstallLocation = Installationsordner" (($reg.InstallLocation.Trim('"')) -eq $dir)
  $list = Dx --json list | ConvertFrom-Json
  Check "Status: installiert $Version, laeuft nicht" ($list[0].installed -eq $Version -and $list[0].running -eq $false)

  Write-Output "4) Laeuft-Pruefung: Hilfsprogramm im Installationsordner"
  Copy-Item (Join-Path $env:SystemRoot "System32\PING.EXE") "$dir\hilfsprogramm.exe"
  $helper = Start-Process "$dir\hilfsprogramm.exe" -ArgumentList "-n","60","127.0.0.1" -PassThru -WindowStyle Hidden
  Start-Sleep 1
  $list = Dx --json list | ConvertFrom-Json
  Check "Status: laeuft" ($list[0].running -eq $true)
  & $tool add-release --catalog "$cat\catalog.json" --app desk --entry $entry --replace | Out-Null
  # neue Version 9.9.9 im Katalog, damit ein Update ansteht (das Paket wird dabei nicht geladen)
  $e2 = Get-Content $entry -Raw | ConvertFrom-Json; $e2.version = "9.9.9"
  $e2 | ConvertTo-Json -Depth 5 | Set-Content "$root\entry2.json" -Encoding utf8
  & $tool add-release --catalog "$cat\catalog.json" --app desk --entry "$root\entry2.json" | Out-Null
  & $tool sign --catalog "$cat\catalog.json" --key "$keys\catalog.key" | Out-Null
  $before = (Get-Item "$dir\app.json").LastWriteTime
  $out = Dx update desk
  Check "Update wird abgelehnt (laeuft)" ($out -match "ft gerade")
  Check "nichts geladen, nichts verandert" ((-not (Test-Path "$data\downloads") -or (Get-ChildItem "$data\downloads").Count -eq 0) -and (Get-Item "$dir\app.json").LastWriteTime -eq $before)
  $out = Dx remove desk
  Check "Entfernen wird abgelehnt (laeuft)" ($out -match "ft gerade")
  Stop-Process -Id $helper.Id -Force; Start-Sleep 1
  Remove-Item "$dir\hilfsprogramm.exe" -Force
  $list = Dx --json list | ConvertFrom-Json
  Check "Status: laeuft nicht mehr, Update 9.9.9 verfugbar" ($list[0].running -eq $false -and $list[0].update_available -eq $true)

  Write-Output "5) Entfernen durch den Launcher"
  $out = Dx remove desk
  Check "Entfernt" ($out -match "entfernt")
  Check "app.json weg" (-not (Test-Path "$dir\app.json"))
  Check "Registry-Eintrag weg" (-not (Test-Path $regKey))

  Write-Output "6) Von Hand installierte Version wird uebernommen"
  $manual = Join-Path $root "manuell\Daedyx Desk"
  Start-Process $Setup -ArgumentList "/S","/D=$manual" -Wait
  Check "Handinstallation hat app.json" (Test-Path "$manual\app.json")
  Remove-Item $data -Recurse -Force   # frischer Launcher-Zustand, nur Katalog-Einstellung neu
  Dx config set --catalog $cat --pubkey "$keys\catalog.pub" | Out-Null
  $list = Dx --json list | ConvertFrom-Json
  Check "Launcher erkennt die Handinstallation ($Version)" ($list[0].installed -eq $Version)
  $state = Get-Content "$data\installed.json" -Raw | ConvertFrom-Json
  Check "Installationsordner aus der Registry gemerkt" ($state.apps.desk.install_dir -eq $manual)
  $out = Dx remove desk
  Check "Entfernen ausserhalb des Programmordners wird abgelehnt" ($out -match "erhalb des Programmordners")
  Start-Process "$manual\uninstall.exe" -ArgumentList "/S","_?=$manual" -Wait
  Check "Handinstallation entfernt, Registry sauber" (-not (Test-Path "$manual\app.json") -and -not (Test-Path $regKey))
  Write-Output "`nAbnahme bestanden ($script:ok Prufungen)."
} finally {
  Get-Process hilfsprogramm -ErrorAction SilentlyContinue | Stop-Process -Force
  foreach ($d in @("$data\apps\desk", "$root\manuell\Daedyx Desk")) {
    if (Test-Path "$d\uninstall.exe") { Start-Process "$d\uninstall.exe" -ArgumentList "/S","_?=$d" -Wait }
  }
  Remove-Item $root -Recurse -Force -ErrorAction SilentlyContinue
}
