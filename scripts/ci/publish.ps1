# Veröffentlicht eine gebaute Launcher-Version in das Repository daedyx/releases (läuft in der GitHub-Action):
#  1. Release-Datei (Setup) unter dem Tag launcher-v<Version> anlegen,
#  2. catalog.json um den Eintrag der App "launcher" ergänzen, mit dem Katalog-Schlüssel signieren und prüfen, committen, pushen.
# Reihenfolge: erst die Datei, dann der Katalog, damit der Katalog nie auf eine fehlende Datei zeigt.
# Den Katalog ändert auch die Pipeline von Desk: bei einem Push-Konflikt wird neu aufgesetzt (höchstens dreimal).
# Geheimnisse nur über Umgebungsvariablen: RELEASES_TOKEN (Schreibrecht auf daedyx/releases),
# DAEDYX_CATALOG_KEY (Inhalt der geheimen Schlüsseldatei), DAEDYX_KEY_PASSWORD. Nichts davon wird ausgegeben.
param(
  [Parameter(Mandatory)][string]$Version,
  [Parameter(Mandatory)][string]$OutDir,        # Ergebnis von build-setup.ps1: Setup und launcher-catalog-entry.json
  [Parameter(Mandatory)][string]$NotesFile,
  [Parameter(Mandatory)][string]$PublicKey,     # catalog.pub (öffentlich, liegt im Repository)
  [Parameter(Mandatory)][string]$CatalogTool,   # daedyx-catalog.exe
  [string]$ReleasesRepo = "daedyx/releases",
  [string]$ReleasesRemote,                      # nur für Proben: anderer Git-Remote (zum Beispiel ein lokaler Ordner)
  [switch]$SkipGhRelease                        # nur für Proben: keine Release-Datei bei GitHub anlegen
)
$ErrorActionPreference = "Stop"
$tag = "launcher-v$Version"
$work = Join-Path ([System.IO.Path]::GetTempPath()) "daedyx-launcher-releases-$PID"

function Fail($text) { Write-Error $text; exit 1 }
# Nur der Exit-Code zählt: Windows PowerShell 5.1 macht aus Warnungen auf stderr sonst einen Abbruch
function Invoke-GitCode {
  $old = $ErrorActionPreference; $ErrorActionPreference = "Continue"
  & git.exe @args 2>&1 | ForEach-Object { "$_" } | Write-Host
  $code = $LASTEXITCODE; $ErrorActionPreference = $old
  return $code
}
function Invoke-Git { if ((Invoke-GitCode @args) -ne 0) { throw "git $($args[0]) ist fehlgeschlagen" } }

foreach ($name in "DAEDYX_CATALOG_KEY", "DAEDYX_KEY_PASSWORD") {
  if (-not [Environment]::GetEnvironmentVariable($name)) { Fail "Das Secret $name fehlt." }
}
if (-not $ReleasesRemote -and -not $env:RELEASES_TOKEN) { Fail "Das Secret RELEASES_TOKEN fehlt." }
if ($Version -notmatch '^\d+\.\d+\.\d+$') { Fail "Ungültige Version: $Version" }
$setup = Get-ChildItem $OutDir -Filter "*_${Version}_x64-setup.exe" | Select-Object -First 1
if (-not $setup) { Fail "Kein Setup für Version $Version in $OutDir." }
if (-not (Test-Path (Join-Path $OutDir "launcher-catalog-entry.json"))) { Fail "launcher-catalog-entry.json fehlt in $OutDir." }
if (-not (Test-Path $PublicKey)) { Fail "Der öffentliche Katalog-Schlüssel fehlt: $PublicKey" }
# Nach Push-Location gelten relative Pfade nicht mehr: alle Pfad-Parameter jetzt absolut machen.
$PublicKey = (Resolve-Path $PublicKey).Path
$OutDir = (Resolve-Path $OutDir).Path
$NotesFile = (Resolve-Path $NotesFile).Path
$CatalogTool = (Resolve-Path $CatalogTool).Path

# Anmeldung für git nur über einen Kopfzeilen-Parameter, das Token steht nie in der Adresse oder in .git/config
$remote = $ReleasesRemote
$gitAuth = @()
if (-not $remote) {
  $remote = "https://github.com/$ReleasesRepo.git"
  $basic = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("x-access-token:$($env:RELEASES_TOKEN)"))
  $gitAuth = @("-c", "http.extraheader=AUTHORIZATION: basic $basic")
}
function Clone-Releases {
  if (Test-Path $work) { Remove-Item -Recurse -Force $work }
  Invoke-Git @gitAuth clone --quiet $remote $work
  Push-Location $work
  Invoke-Git config user.name "daedyx release bot"
  Invoke-Git config user.email "release-bot@users.noreply.github.com"
  if ((Invoke-GitCode rev-parse --verify HEAD) -ne 0) {
    # leeres Repository: Hauptzweig anlegen, damit Tag und Katalog ein Ziel haben
    Invoke-Git checkout --quiet -b main
    Set-Content (Join-Path $work "README.md") "# Daedyx Releases`n`nSetup-Dateien (Release-Dateien) und der signierte Katalog (catalog.json) der Daedyx-Apps." -Encoding utf8
    Invoke-Git add README.md
    Invoke-Git commit --quiet -m "chore: Repository anlegen"
    Invoke-Git @gitAuth push --quiet origin main
  }
  Pop-Location
}

# 1. Prüfen: Version ist höher als jede bisherige des Launchers
Clone-Releases
$catalogPath = Join-Path $work "catalog.json"
if (Test-Path $catalogPath) {
  try { $cat = [IO.File]::ReadAllText($catalogPath).TrimStart([char]0xFEFF) | ConvertFrom-Json } catch { Fail "Der bisherige Katalog ist beschädigt; es wird nichts veröffentlicht." }
  $app = @($cat.apps) | Where-Object { $_.id -eq "launcher" } | Select-Object -First 1
  foreach ($r in @($app.releases)) {
    if ([version]$Version -le [version]$r.version) { Fail "Version $Version muss höher sein als die schon veröffentlichte $($r.version)." }
  }
}

# 2. Release-Datei hochladen
if (-not $SkipGhRelease) {
  $env:GH_TOKEN = $env:RELEASES_TOKEN
  & gh release view $tag --repo $ReleasesRepo *> $null
  if ($LASTEXITCODE -eq 0) { Fail "Das Release $tag gibt es schon." }
  & gh release create $tag $setup.FullName --repo $ReleasesRepo --target main --title "Daedyx Launcher $Version" --notes-file $NotesFile
  if ($LASTEXITCODE -ne 0) { Fail "Das Release $tag konnte nicht angelegt werden." }
}

# 3. Katalog ergänzen, signieren, prüfen, committen (bei einem gleichzeitigen Push höchstens dreimal neu)
$pushed = $false
for ($try = 1; $try -le 3 -and -not $pushed; $try++) {
  if ($try -gt 1) { Clone-Releases }
  Push-Location $work
  Copy-Item $PublicKey (Join-Path $work "catalog.pub") -Force
  & $CatalogTool add-release --catalog catalog.json --app launcher --entry (Join-Path $OutDir "launcher-catalog-entry.json") `
    --name "Daedyx Launcher" --description "Apps der Daedyx-Suite installieren und aktualisieren" --author "Daedyx"
  if ($LASTEXITCODE -ne 0) { Pop-Location; Fail "add-release ist fehlgeschlagen." }
  & $CatalogTool sign --catalog catalog.json --key-env DAEDYX_CATALOG_KEY --public catalog.pub
  if ($LASTEXITCODE -ne 0) { Pop-Location; Fail "Das Signieren ist fehlgeschlagen." }
  & $CatalogTool verify --catalog catalog.json --public catalog.pub
  if ($LASTEXITCODE -ne 0) { Pop-Location; Fail "Die Gegenprobe der Signatur ist fehlgeschlagen." }
  # Signierte Dateien dürfen nie umgewandelt werden (Zeilenenden), sonst passt die Signatur nicht mehr.
  # Die Datei gehört beiden Pipelines (Desk fügt eigene Zeilen hinzu): nur anlegen, nie überschreiben.
  $ga = Join-Path $work ".gitattributes"
  if (-not (Test-Path $ga)) {
    Set-Content $ga "catalog.json -text`ncatalog.json.minisig -text`ncatalog.pub -text" -Encoding ascii
  }
  Invoke-Git add .gitattributes catalog.json catalog.json.minisig catalog.pub
  Invoke-Git commit --quiet -m "release: Daedyx Launcher $Version"
  if ((Invoke-GitCode @gitAuth push --quiet origin main) -eq 0) { $pushed = $true } else { Write-Output "Push abgelehnt (Versuch $try), neu aufsetzen." }
  Pop-Location
}
if (-not $pushed) { Fail "Der Katalog konnte nicht gepusht werden. Die Release-Datei $tag ist schon da; den Katalog-Eintrag von Hand nachtragen." }

Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
Write-Output "Veröffentlicht: Daedyx Launcher $Version ($tag), Katalog aktualisiert."
exit 0
