# Installationsprobe des Launchers (U3, Review U3 F-U3-2). Nur mit Probe-Setups (`scripts\build-setup.ps1 -Probe`,
# Kennung „Daedyx Launcher Probe“, Merkmal `probe`); eine echte Installation „Daedyx Launcher“ wird nur gelesen.
# Ablauf ohne Netz:
#   0 Probe-Setup -From still in einen Wegwerf-Ordner installieren.
#   A Selbst-Update: Die installierte Kommandozeile startet das Setup -To über den gesperrten Griff mit
#     `/S /UPDATE /NOLAUNCH` (probe-self-update); Programm ersetzt, Ordner gleich, Version -To, kein Neustart.
#   B App über den Launcher: ein echtes NSIS-Setup (-AppSetup, z. B. das der Desk-Vorschau) mit `/S /D=` in den
#     Programmordner des Launchers (probe-install), dann `remove` über `uninstall.exe`.
#   C Deinstallation des Launchers (/S), alles weg.
# Den Nachweis braucht die Probe nicht (geprüft im Kern und in den Tests von launcher-core).
# Rückgabe 0 = alles in Ordnung, 1 = Fehler, 2 = Abbruch vor jedem Schreiben. Gibt nur OK/FEHLER und Zahlen aus.
# Beispiel: powershell -ExecutionPolicy Bypass -File scripts\installer-probe-launcher.ps1 -From 0.0.1 `
#   -Setup1 dist\0.0.1\Daedyx-Launcher-Probe_0.0.1_x64-setup.exe -To 0.0.2 `
#   -Setup2 dist\0.0.2\Daedyx-Launcher-Probe_0.0.2_x64-setup.exe `
#   -AppSetup ..\wt-desk-u3\installer\out\daedyx-desk-vorschau-0.0.1-setup.exe -AppProduct "Daedyx Desk Vorschau"
param(
    [Parameter(Mandatory = $true)][string]$From,
    [Parameter(Mandatory = $true)][string]$Setup1,
    [Parameter(Mandatory = $true)][string]$To,
    [Parameter(Mandatory = $true)][string]$Setup2,
    [Parameter(Mandatory = $true)][string]$AppSetup,
    [Parameter(Mandatory = $true)][string]$AppProduct,
    [string]$Dir = (Join-Path $env:TEMP "daedyx-launcher-probe-$PID"),
    [string]$Data = (Join-Path $env:TEMP "daedyx-launcher-probe-daten-$PID")
)
$product = "Daedyx Launcher Probe"
$uninst = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$product"
$dirKey = "HKCU:\Software\Daedyx\LauncherProbe"
$lnk = Join-Path ([Environment]::GetFolderPath("Programs")) "$product.lnk"
$appUninst = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppProduct"
$realUninst = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Daedyx Launcher"
$realDirKey = "HKCU:\Software\Daedyx\Launcher"
$locks = Join-Path $env:LOCALAPPDATA "daedyx\locks"
$script:failed = 0

function Check($name, $ok) { if ($ok) { "OK      $name" } else { "FEHLER  $name"; $script:failed++ } }
function Run($file, $argLine, $ms = 120000) {
    $p = Start-Process $file -ArgumentList $argLine -PassThru -WindowStyle Hidden
    if ($p.WaitForExit($ms)) { return $p.ExitCode }
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    return "haengt"
}
function Wait-Until($ms, [scriptblock]$cond) {
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $ms) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 250 }
    return [bool](& $cond)
}
function Version { (Get-ItemProperty $uninst -ErrorAction SilentlyContinue).DisplayVersion }
function Procs { @(Get-Process "daedyx-launcher", "daedyx" -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "$Dir\*" }) }
function RealState {
    $d = (Get-ItemProperty $realDirKey -ErrorAction SilentlyContinue).InstallDir
    "u=$(Test-Path $realUninst)|d=$d"
}
function Invoke-Cli($argLine) { Run "$Dir\daedyx.exe" "--data-dir `"$Data`" --json $argLine" }

# Schutz (vor jedem Schreiben): Probe-Kennung und App nirgends vorhanden; der echte Launcher wird nur gelesen.
if ((Test-Path $uninst) -or (Test-Path $dirKey) -or (Test-Path $lnk) -or (Test-Path $Dir) -or (Test-Path $Data) -or (Test-Path $appUninst)) {
    "Abbruch: Probe-Launcher oder App sind schon installiert (nichts verändert)."; exit 2
}
if (-not (Test-Path $Setup1) -or -not (Test-Path $Setup2) -or -not (Test-Path $AppSetup) -or $From -eq $To) {
    "Abbruch: Setups fehlen oder gleiche Version."; exit 2
}
$realBefore = RealState
$locksBefore = Test-Path $locks

Check "0 Probe-Launcher $From still installieren (/S /D=)" ((Run $Setup1 "/S /D=$Dir") -eq 0)
Check "0 Uninstall-Eintrag Version $From, Ordner $Dir" ((Version) -eq $From -and (Get-ItemProperty $uninst).InstallLocation -eq $Dir)
Check "0 Programme da, kein Katalog-Werkzeug" ((Test-Path "$Dir\daedyx.exe") -and (Test-Path "$Dir\daedyx-launcher.exe") -and -not (Test-Path "$Dir\daedyx-catalog.exe"))
$hash1 = (Get-FileHash "$Dir\daedyx.exe").Hash

Check "A Selbst-Update gestartet (probe-self-update)" ((Invoke-Cli "probe-self-update `"$((Resolve-Path $Setup2).Path)`"") -eq 0)
Check "A Setup ersetzt das Programm (Version $To)" (Wait-Until 60000 { (Version) -eq $To -and (Get-FileHash "$Dir\daedyx.exe").Hash -ne $hash1 })
Check "A Ordner gleich" ((Get-ItemProperty $uninst).InstallLocation -eq $Dir)
Start-Sleep -Seconds 2
Check "A kein Neustart (/NOLAUNCH)" ((Procs).Count -eq 0)
Check "A Setup lag im eigenen Ordner des Launchers (downloads\daedyx-launcher)" (@(Get-ChildItem "$Data\downloads\daedyx-launcher" -File -ErrorAction SilentlyContinue).Count -eq 1)

Check "B App über den Launcher installieren (probe-install, /S /D=)" ((Invoke-Cli "probe-install desk `"$((Resolve-Path $AppSetup).Path)`"") -eq 0)
Check "B App im Programmordner des Launchers, app.json da" (Test-Path "$Data\apps\desk\app.json")
Check "B Zustand merkt die App" ((Get-Content "$Data\installed.json" -Raw) -match '"desk"')
Check "B Uninstall-Eintrag der App da" (Test-Path $appUninst)
Check "B keine Kopie des Setups liegt herum" (@(Get-ChildItem "$Data\downloads\daedyx-desk" -File -ErrorAction SilentlyContinue).Count -eq 0)
Check "B App entfernen (remove, uninstall.exe)" ((Invoke-Cli "remove desk") -eq 0)
Start-Sleep -Seconds 2
Check "B App-Ordner und Uninstall-Eintrag der App weg" (-not (Test-Path "$Data\apps\desk") -and -not (Test-Path $appUninst))

Check "C Deinstallation des Probe-Launchers (/S)" ((Run "$Dir\uninstall.exe" "/S") -eq 0)
Check "C Ordner entfernt" (Wait-Until 15000 { -not (Test-Path "$Dir\daedyx.exe") })
Check "C Uninstall-Eintrag, Merkschlüssel, Verknüpfung entfernt" (-not ((Test-Path $uninst) -or (Test-Path $dirKey) -or (Test-Path $lnk)))
Check "C keine Prozesse" ((Procs).Count -eq 0)
Check "C echter Launcher unverändert (nur gelesen)" ((RealState) -eq $realBefore)
Remove-Item $Dir, $Data -Recurse -Force -ErrorAction SilentlyContinue
# Den gemeinsamen Sperrordner nur wegräumen, wenn die Probe ihn angelegt hat und er leer ist.
if (-not $locksBefore -and (Test-Path $locks) -and -not (Get-ChildItem $locks -Force)) {
    Remove-Item $locks -Force
}
Check "C keine Sperrdatei übrig" (-not (Test-Path $locks) -or -not (Get-ChildItem $locks -Force))
if ($script:failed -eq 0) { "ERGEBNIS: alles in Ordnung"; exit 0 } else { "ERGEBNIS: $($script:failed) Fehler"; exit 1 }
