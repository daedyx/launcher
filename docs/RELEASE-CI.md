# Den Launcher per Knopfdruck veröffentlichen (GitHub Actions)

Stand: 2026-11-17. Ziel: auf „Run workflow“ drücken, eine neue Launcher-Version erscheint, installierte Launcher holen sie von selbst (Selbst-Update). Keine PowerShell im Alltag.

## Was die Pipeline macht
1. Trägt die Version in `Cargo.toml` ein (der Launcher kennt seine Version nur von dort; das Selbst-Update vergleicht sie mit dem Katalog).
2. Führt Clippy und alle Tests aus (bei einem Fehlschlag ein zweiter Versuch, gegen zeitabhängige Tests).
3. Baut das Setup (`scripts/build-setup.ps1 -RequireRealKey`, bricht ab, solange in `crates/launcher-core/assets/catalog.pub` ein Platzhalter steht) und den Katalog-Eintrag `launcher-catalog-entry.json`.
4. Legt das Setup als Release-Datei in `daedyx/releases` ab (Tag `launcher-v<Version>`, Datei `Daedyx-Launcher_<Version>_x64-setup.exe`).
5. Ergänzt in `catalog.json` die App `launcher`, signiert mit dem Katalog-Schlüssel, prüft die Signatur, committet und pusht. Der Katalog wird auch von der Pipeline von Desk geändert: bei einem Push-Konflikt wird bis zu dreimal neu aufgesetzt.
6. Schreibt `Cargo.toml` und `Cargo.lock` mit der neuen Version zurück in den Quellcode.

Reihenfolge: erst die Release-Datei, dann der Katalog, damit der Katalog nie auf eine fehlende Datei zeigt. Das Katalog-Werkzeug lädt die Pipeline aus dem öffentlichen Release `tools` in `daedyx/releases` und prüft dessen SHA-256.

Feste Adressen (öffentlich, ohne Anmeldung):
- Katalog: `https://raw.githubusercontent.com/daedyx/releases/main/catalog.json` (daneben `catalog.json.minisig`, `catalog.pub`)
- Setup: `https://github.com/daedyx/releases/releases/download/launcher-v<Version>/Daedyx-Launcher_<Version>_x64-setup.exe`

## Einmalig einrichten (du, im Browser)
Geheimnisse gehören **nur** in GitHub-Secrets, nie in den Chat, nie in Git. Dieselben Werte wie bei Desk.

Repository `daedyx/launcher` → **Settings** → **Secrets and variables** → **Actions** → **New repository secret**. Je Secret ein Eintrag, Name genau so:

| Name | Was du einfügst |
|---|---|
| `DAEDYX_CATALOG_KEY` | der ganze Inhalt der Datei `catalog.key` (im Editor öffnen, alles kopieren) |
| `DAEDYX_KEY_PASSWORD` | das Passwort des Katalog-Schlüssels |
| `RELEASES_TOKEN` | Fine-grained Token, Resource owner `daedyx`, nur Repository `daedyx/releases`, Berechtigung **Contents: Read and write**, Ablauf 1 Jahr (Schritt-für-Schritt wie in der Anleitung von Desk: GitHub → Profilbild → Settings → Developer settings → Personal access tokens → Fine-grained tokens) |

Das Token ist dasselbe wie bei Desk; es muss aber auch im Repository `daedyx/launcher` als Secret stehen (Secrets gelten je Repository).

Der öffentliche Katalog-Schlüssel ist schon im Programm eingebaut (`crates/launcher-core/assets/catalog.pub`) und liegt öffentlich neben dem Katalog.

## Eine Version veröffentlichen
1. `github.com/daedyx/launcher` → **Actions** → links **Release** → rechts **Run workflow**.
2. **Version** eintragen (`0.1.0` für die allererste; danach immer höher als die letzte), **Änderungsnotiz** eintragen, **Run workflow**. Die Notiz erscheint im Launcher beim Update.
3. Warten (etwa 10 bis 20 Minuten, das erste Mal länger). Grün = fertig. Rot = Protokoll des roten Schritts öffnen (Tabelle unten).
4. Installierte Launcher holen das Update beim nächsten Start oder in der regelmäßigen Prüfung selbst.

Alternativ: einen Tag `v0.2.0` pushen, dann startet dieselbe Pipeline (Notiz: „Daedyx Launcher 0.2.0“; die Versionsdateien werden dann nicht zurückgeschrieben, `Cargo.toml` im Quellcode bitte von Hand nachziehen).

Der **erste echte Lauf macht das Setup öffentlich** (Release-Datei und Katalogeintrag in einem öffentlichen Repository). Deshalb nur starten, wenn das gewollt ist.

## Erste Installation (für dich)
1. `https://github.com/daedyx/releases/releases` öffnen, beim Release `launcher-v0.1.0` die Datei `Daedyx-Launcher_0.1.0_x64-setup.exe` laden.
2. Doppelklick. Windows zeigt **„Der Computer wurde durch Windows geschützt“** (SmartScreen), weil das Programm nicht mit einem Zertifikat signiert ist: **Weitere Informationen** → **Trotzdem ausführen**. Das Setup braucht keine Administratorrechte und installiert nach `%LOCALAPPDATA%\Programs\Daedyx Launcher`, mit Startmenü-Eintrag.
3. Der Launcher startet, holt den Katalog von der festen Adresse und zeigt die Apps. Ab jetzt aktualisiert er sich selbst; in den Einstellungen steht „Launcher automatisch aktualisieren“ (Voreinstellung an) und optional „Beim Anmelden starten“.

Keine Selbst-Update-Schleife: Das Setup trägt dieselbe Version wie `Cargo.toml` (das Skript bricht sonst ab), und der Launcher bietet ein Update nur an, wenn die Katalog-Version **höher** ist.

## Wenn etwas schiefgeht
| Meldung im Protokoll | Ursache und Lösung |
|---|---|
| „Diese Secrets fehlen im Repository: …“ | Das genannte Secret eintragen (siehe oben). |
| „Ungültige Version“ | Format `x.y.z`, zum Beispiel `0.2.0`. |
| „In …\catalog.pub steht noch kein echter öffentlicher Schlüssel“ | Der Platzhalter ist noch eingebaut: echten öffentlichen Schlüssel einchecken. |
| „Version … muss höher sein als die schon veröffentlichte …“ | Höhere Versionsnummer wählen. |
| „Das Release launcher-v… gibt es schon“ | Version war schon veröffentlicht oder ein früherer Lauf ist nach dem Hochladen abgebrochen; neue Nummer nehmen und das halbe Release in `daedyx/releases` löschen. |
| Schritt „Katalog-Werkzeug laden“ rot | Das Release `tools` fehlt oder die Prüfsumme stimmt nicht. |
| `403` oder „Resource not accessible“ beim Veröffentlichen | Token ohne Schreibzugriff auf `daedyx/releases` oder abgelaufen; auch im Repository `daedyx/launcher` eintragen. |
| Signieren schlägt fehl | Falsches Passwort in `DAEDYX_KEY_PASSWORD` oder nur ein Teil von `catalog.key` kopiert. |
| „Die Tests sind fehlgeschlagen“ | Protokoll des Schritts „Prüfen“ lesen; nur im Notfall „Tests überspringen“ anhaken. |
| „Der Katalog konnte nicht gepusht werden“ | Gleichzeitig lief die Pipeline von Desk dreimal in Folge dazwischen. Die Release-Datei ist schon da: Bescheid sagen, der Katalog-Eintrag wird von Hand nachgetragen. Den Lauf **nicht** wiederholen. |

## Probe ohne Netz
`scripts\ci\test-publish.ps1 -CatalogTool target\release\daedyx-catalog.exe -LauncherCli target\release\daedyx.exe` spielt `publish.ps1` gegen ein lokales Git-Repository mit Wegwerf-Schlüsseln durch: leeres Repository, zwei Versionen, doppelte und niedrigere Version werden abgelehnt, ein vorhandener Eintrag von Desk und dessen `.gitattributes` bleiben erhalten, der echte Launcher liest das Ergebnis.

## Nicht geprüft
Ein echter Lauf bei GitHub (Windows-Runner: NSIS-Bereitstellung, Laufzeit, Tests ohne Grafikkarte) steht aus; geprüft sind Skripte und Katalogweg lokal. Das Setup ist nicht mit einem Zertifikat signiert (SmartScreen-Hinweis).

## Brücke 0.2.0 (U3, 2026-11-26)

Ab 0.2.0 liest der Launcher keinen Katalog mehr, sondern prüft je App `entry.json` mit GitHub-Nachweis (Kern `daedyx-update`).
- Der eigene Crate `daedyx-update` mit dem Werkzeug `daedyx-catalog` ist gelöscht (Entscheid F1).
- Die Veröffentlichung von 0.2.0 läuft ein letztes Mal über diesen alten Ablauf, damit installierte 0.1.0 die Brücke im alten Katalog finden.
- `release.yml` holt dafür das Werkzeug aus `daedyx/releases` (`scripts/ci/fetch-catalog-tool.ps1`) und prüft mit `crates/launcher-core/assets/catalog.pub`.
- Die Trockenprobe `scripts\ci\test-publish.ps1 -CatalogTool <daedyx-catalog.exe>` prüft das Ergebnis mit `daedyx-catalog verify`, nicht mehr mit der Kommandozeile des Launchers.
- Ab 0.3.0 gilt der neue Weg: `entry.json` und Attestierung im öffentlichen Repo, Workflow `release.yml` auf `refs/heads/main` (plan-ohne-secrets 2, Schritt 8).
