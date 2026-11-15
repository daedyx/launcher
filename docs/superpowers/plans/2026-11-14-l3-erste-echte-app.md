# L3 Erste echte App (Daedyx Desk) — Detailplan

> Auftrag des Master-Chats (Björn will so schnell wie möglich nur noch über Launcher und Installer arbeiten). Test zuerst, kleine Commits, `docs/STAND.md` nachziehen, pushen, dem Master berichten.

**Ziel:** Daedyx Desk lässt sich mit dem echten Windows-Setup aus einem lokalen, signierten Katalogordner installieren, aktualisieren und starten; der Launcher kennt die installierte Version aus Registry und `app.json`, erkennt „läuft“, aktualisiert nie eine laufende App, und Katalogquelle und Schlüssel sind dauerhaft einstellbar.

## Was die Probe mit dem echten Setup ergeben hat (2026-11-14)
`Daedyx Desk_0.1.0_x64-setup.exe` mit `/S /D=<Ordner>`:
- Dateien: `daedyx-desk.exe`, `uninstall.exe`, **`app.json`** = `{"version": "0.1.0", "exe": "daedyx-desk.exe"}` (Hook `installer-hooks.nsh`, läuft bei jeder Installation, auch beim Selbst-Update).
- Registry (pro Benutzer): `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Daedyx Desk` mit `DisplayName`, `DisplayVersion` (`0.1.0`), `InstallLocation` (**in Anführungszeichen**), `UninstallString`, `Publisher`.
- Deinstallation `uninstall.exe /S _?=<Ordner>` entfernt `app.json` und den Uninstall-Schlüssel; zurück bleibt nur der leere Schlüssel `HKCU\Software\daedyx`.

## Entscheidungen
1. **Versionsquelle:** erst `app.json` im Installationsordner (Quelle der Wahrheit, vom Setup bei jeder Installation geschrieben), sonst Registry `DisplayVersion`. Den **Installationsordner** liefert der Zustand des Launchers, sonst die Registry (`InstallLocation`), sonst `<Datenordner>\apps\<id>`. So erkennt der Launcher auch eine von Hand installierte Desk-Version („übernehmen“) und aktualisiert sie an Ort und Stelle.
2. **Katalogformat:** neues optionales Feld je App: `registryKey` (Name des Uninstall-Schlüssels, bei Desk `Daedyx Desk`). Die Release-Einträge von `release.ps1` (`catalog-entry.json`) bleiben unverändert. Schema bleibt 1.
3. **Läuft:** Ein Prozess zählt als laufend, wenn sein Programmpfad im Installationsordner der App liegt (Prozessliste über `sysinfo`). Der Kern lehnt Update und Entfernen einer laufenden App mit `AppRunning` ab, ohne etwas zu laden. Die Oberfläche zeigt „LÄUFT“ (alle paar Sekunden aktualisiert) und fragt vor dem Update nach.
4. **Update-Sperre:** bleibt die Sperrdatei je App `<Datenordner>\locks\<id>.lock` (Anlegen schlägt fehl, wenn sie existiert; nach einer Stunde gilt sie als übrig geblieben). Sie ist der Vertrag mit Desk-Selbstupdate (Chat D1): Existiert die Datei, startet Desk kein eigenes Update.
5. **Dauerhafte Einstellungen:** `config.json` im Datenordner (`catalog`, `pubkey`). Vorrang: Startparameter/Umgebung vor `config.json`. Die Einstellungsseite bearbeitet beides, prüft vor dem Speichern (Quelle erlaubt? Schlüsseldatei lesbar und gültig?) und startet den Arbeiter neu.
6. **Katalog-Werkzeug `daedyx-catalog`** (zweites Programm in `launcher-cli`, auch in der Veröffentlichung): `keygen` (Schlüsselpaar, Passwort tippt Björn selbst, zweimal), `add-release` (fügt `catalog-entry.json` in den Eintrag der App in `catalog.json` ein, legt den Eintrag mit `--name` usw. an), `sign` (schreibt `catalog.json.minisig`, fragt das Passwort ab). Testschlüssel ohne Passwort nur mit `--no-password`.
7. **Gestartet wird Desk in den Tests nie wirklich:** Desk übernimmt beim ersten Start Daten und legt Sicherungen an (Datenordner des Benutzers). Der Start wird mit einem Test-Programm geprüft, der Weg zum echten `daedyx-desk.exe` nur aufgelöst.

## Aufgaben
1. `daedyx-update`: Feld `registryKey` (Test: optional, wird gelesen).
2. Kern: Trait `SystemInfo` (`uninstall_entry`, `running`), echte Umsetzung (`winreg`, `sysinfo`) mit Tests gegen einen Test-Schlüssel und einen echten Hilfsprozess, Test-Attrappe in `test_support`; `Launcher.system`.
3. Kern: Installationsort und Version aus Zustand/Registry/`app.json` („übernehmen“), `AppStatus.running`, `AppRunning` bei Update/Entfernen, `Job::CheckRunning` und `Event::Running`.
4. Kern: `config.rs` (laden, speichern, prüfen); Kommandozeile liest Quelle und Schlüssel aus Startparametern oder `config.json`; `daedyx config show|set`.
5. `daedyx-catalog` mit Ende-zu-Ende-Tests (keygen → add-release → sign → `daedyx list`).
6. Oberfläche: Etikett „LÄUFT“, Rückfrage vor dem Update, Einstellungsseite mit Eingabefeldern (Katalogquelle, Schlüsseldatei), Neustart des Arbeiters; Texte de/en.
7. Abnahme mit dem **echten Desk-Setup** in einem Temp-Datenordner (Katalog per `daedyx-catalog`, Installation, Status, Entfernen), `STAND.md`, Push, Bericht.
