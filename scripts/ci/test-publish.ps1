# Trockenprobe von publish.ps1 ohne Netz: lokales Git-Repository als Gegenstelle, Wegwerf-Schlüssel, Attrappen-Setup.
# Prüft: leeres Repository wird angelegt, Katalog wird signiert und ist gültig, zweite Version kommt dazu,
# dieselbe und eine niedrigere Version werden abgelehnt, ein vorhandener Eintrag von Desk und eine fremde
# .gitattributes bleiben unberührt.
# Die GitHub-Release-Datei (gh) wird hier ausgelassen.
# Aufruf: scripts\ci\test-publish.ps1 -CatalogTool <daedyx-catalog.exe> -LauncherCli <daedyx.exe>
param([Parameter(Mandatory)][string]$CatalogTool, [Parameter(Mandatory)][string]$LauncherCli)
$ErrorActionPreference = "Continue"  # Erwartete Fehler (abgelehnte Versionen) dürfen die Probe nicht abbrechen; geprüft wird über Exit-Codes und throw
$CatalogTool = (Resolve-Path $CatalogTool).Path
$LauncherCli = (Resolve-Path $LauncherCli).Path
$t = Join-Path ([System.IO.Path]::GetTempPath()) "daedyx-launcher-publish-test-$PID"
New-Item -ItemType Directory -Force $t | Out-Null
$oldKey = $env:DAEDYX_CATALOG_KEY; $oldPw = $env:DAEDYX_KEY_PASSWORD
try {
  # Wegwerf-Schlüssel (nur für diese Probe)
  & $CatalogTool keygen --out (Join-Path $t "keys") --no-password | Out-Null
  $env:DAEDYX_CATALOG_KEY = Get-Content (Join-Path $t "keys\catalog.key") -Raw
  $env:DAEDYX_KEY_PASSWORD = "unbenutzt-in-der-probe"
  $pub = Join-Path $t "keys\catalog.pub"
  if (-not (Test-Path $pub)) { $pub = Get-ChildItem (Join-Path $t "keys") -Filter "*.pub" -Recurse | Select-Object -First 1 -ExpandProperty FullName }

  # leere Gegenstelle
  $remote = Join-Path $t "remote.git"
  git init --quiet --bare --initial-branch=main $remote
  if ($LASTEXITCODE -ne 0) { throw "git init" }

  function New-Out($version) {
    $out = Join-Path $t "out$version"
    New-Item -ItemType Directory -Force $out | Out-Null
    $name = "Daedyx-Launcher_${version}_x64-setup.exe"
    $bytes = [byte[]](1..200)
    [IO.File]::WriteAllBytes((Join-Path $out $name), $bytes)
    $hash = (Get-FileHash (Join-Path $out $name) -Algorithm SHA256).Hash.ToLower()
    $entry = [ordered]@{
      version = $version; date = "2026-11-17"; notes = "Probe $version"
      packages = [ordered]@{ "windows-x64" = [ordered]@{
        url = "https://github.com/daedyx/releases/releases/download/launcher-v$version/$name"
        sha256 = $hash; size = $bytes.Length; kind = "nsis-silent" } }
    }
    [IO.File]::WriteAllText((Join-Path $out "launcher-catalog-entry.json"), ($entry | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
    $notes = Join-Path $t "notes$version.txt"; Set-Content $notes "Probe $version" -Encoding utf8
    return @{ Out = $out; Notes = $notes }
  }

  function Publish($version) {
    $o = New-Out $version
    $code = 0
    try {
      & (Join-Path $PSScriptRoot "publish.ps1") -Version $version -OutDir $o.Out -NotesFile $o.Notes -PublicKey $pub `
        -CatalogTool $CatalogTool -ReleasesRemote $remote -SkipGhRelease *>&1 | Out-Host
      $code = [int]$LASTEXITCODE
    } catch { Write-Output "(erwarteter Abbruch: $($_.Exception.Message))"; $code = 1 }
    return $code
  }

  if ((Publish "0.1.0") -ne 0) { throw "erste Veröffentlichung fehlgeschlagen" }

  # Die Desk-Pipeline hat inzwischen einen Eintrag und eigene .gitattributes-Zeilen hinzugefügt
  $desk = Join-Path $t "desk-clone"
  git clone --quiet $remote $desk 2>$null
  Push-Location $desk
  git config user.name probe; git config user.email probe@users.noreply.github.com
  $deskEntry = Join-Path $t "desk-entry.json"
  $deskEntryObj = [ordered]@{ version = "0.5.0"; date = "2026-11-17"; notes = "Desk"; packages = [ordered]@{ "windows-x64" = [ordered]@{
    url = "https://github.com/daedyx/releases/releases/download/desk-v0.5.0/Daedyx.Desk_0.5.0_x64-setup.exe"; sha256 = ("a" * 64); size = 10; kind = "nsis-silent" } } }
  [IO.File]::WriteAllText($deskEntry, ($deskEntryObj | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
  & $CatalogTool add-release --catalog catalog.json --app desk --entry $deskEntry --name "Daedyx Desk" --registry-key "Daedyx Desk" --description "x" --author "Daedyx" | Out-Null
  & $CatalogTool sign --catalog catalog.json --key-env DAEDYX_CATALOG_KEY --public catalog.pub | Out-Null
  Set-Content .gitattributes "catalog.json -text`ncatalog.json.minisig -text`ncatalog.pub -text`ndesk/latest.json -text" -Encoding ascii
  git add -A; git commit --quiet -m "release: Desk 0.5.0"; git push --quiet origin main 2>$null
  if ($LASTEXITCODE -ne 0) { throw "Desk-Stand konnte nicht gepusht werden" }
  Pop-Location

  if ((Publish "0.1.1") -ne 0) { throw "zweite Veröffentlichung fehlgeschlagen" }
  if ((Publish "0.1.1") -eq 0) { throw "dieselbe Version wurde nicht abgelehnt" }
  if ((Publish "0.1.0") -eq 0) { throw "eine niedrigere Version wurde nicht abgelehnt" }

  # Ergebnis prüfen: Katalog aus der Gegenstelle mit dem echten Launcher-CLI lesen (prüft Signatur und Schema)
  $check = Join-Path $t "check"
  git clone --quiet $remote $check 2>$null
  foreach ($f in "catalog.json", "catalog.json.minisig", "catalog.pub", "README.md", ".gitattributes") {
    if (-not (Test-Path (Join-Path $check $f))) { throw "Datei fehlt im Ergebnis: $f" }
  }
  if ((Get-Content (Join-Path $check ".gitattributes") -Raw) -notmatch "desk/latest.json") { throw ".gitattributes von Desk wurde überschrieben" }
  $list = & $LauncherCli --catalog $check --pubkey (Join-Path $check "catalog.pub") --data-dir (Join-Path $t "ldata") list
  Write-Output $list
  if (($list -join " ") -notmatch "Daedyx Desk") { throw "Eintrag von Desk ist verloren gegangen" }
  $cat = Get-Content (Join-Path $check "catalog.json") -Raw | ConvertFrom-Json
  $l = $cat.apps | Where-Object { $_.id -eq "launcher" }
  if ((($l.releases | ForEach-Object { $_.version }) -join ",") -ne "0.1.0,0.1.1") { throw "Versionen des Launchers im Katalog: $(($l.releases | ForEach-Object { $_.version }) -join ',')" }
  if ($l.releases[1].packages.'windows-x64'.url -ne "https://github.com/daedyx/releases/releases/download/launcher-v0.1.1/Daedyx-Launcher_0.1.1_x64-setup.exe") { throw "Paketadresse falsch" }
  if (-not ($cat.apps | Where-Object { $_.id -eq "desk" })) { throw "desk fehlt im Katalog" }
  Write-Output "Trockenprobe bestanden."
} finally {
  $env:DAEDYX_CATALOG_KEY = $oldKey; $env:DAEDYX_KEY_PASSWORD = $oldPw
  Remove-Item -Recurse -Force $t -ErrorAction SilentlyContinue
}
