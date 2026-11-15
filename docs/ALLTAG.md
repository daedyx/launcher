# Alltag mit dem Launcher: Katalog anlegen, veröffentlichen, aktualisieren

Stand: 2026-11-14. Alles in der **eigenen PowerShell** von Björn. Passwörter tippt Björn selbst, sie stehen nie in Befehlen, Git oder Chat. Voraussetzung: `cargo build --release` im Ordner `D:\Programmierung\daedyx-launcher` (Programme liegen dann in `target\release`).

## Einmalig: Katalog-Schlüssel erzeugen
```powershell
cd D:\Programmierung\daedyx-launcher
.\target\release\daedyx-catalog.exe keygen --out "$env:USERPROFILE\.daedyx\catalog"
```
Das Programm fragt zweimal nach einem Passwort (verdeckte Eingabe). Ergebnis: `catalog.key` (**geheim**, an zwei Orten sichern, nie in Git oder Chat) und `catalog.pub` (öffentlich). Geht `catalog.key` verloren, lässt sich kein Katalog mehr signieren. Dieser Schlüssel ist ein **anderer** als der Update-Schlüssel von Desk.

## Einmalig: Launcher auf den Katalog zeigen
Den Ordner, in dem der Katalog liegt (lokaler Ordner oder später `https://…`), und den öffentlichen Schlüssel eintragen. Entweder in der Oberfläche (Zahnrad → Katalog) oder so:
```powershell
.\target\release\daedyx.exe config set --catalog "D:\Katalog" --pubkey "$env:USERPROFILE\.daedyx\catalog\catalog.pub"
```
Der Befehl prüft zuerst (Ordner da? Schlüssel gültig?) und speichert dann dauerhaft.

## Pro Desk-Version
1. Im Werkbank-Ordner `scripts\release.ps1 …` ausführen (siehe `RELEASE.md` dort). Ergebnis: Setup und `catalog-entry.json` in `dist-release\<Version>\`.
2. Das Setup **unverändert** in den Katalogordner legen (zum Beispiel `D:\Katalog\packages\`). Prüfsumme und Größe im `catalog-entry.json` gelten genau für diese Datei.
3. Release in den Katalog einfügen (beim **ersten Mal** mit den Angaben zur App):
```powershell
.\target\release\daedyx-catalog.exe add-release --catalog "D:\Katalog\catalog.json" --app desk --entry "D:\Programmierung\Werkbank\dist-release\0.2.0\catalog-entry.json" --package-url "packages/Daedyx.Desk_0.2.0_x64-setup.exe" --name "Daedyx Desk" --registry-key "Daedyx Desk" --description "Mail, Kalender, Aufgaben" --license "…" --author "Daedyx"
```
`--package-url` ersetzt die Adresse im Eintrag (`release.ps1` erlaubt dort nur `https://`; ein lokaler Katalogordner braucht eine **relative** Adresse zum Katalogordner). Bei einem Katalog im Netz entfällt `--package-url`, dann gilt die `https://`-Adresse aus dem Eintrag. Ab der zweiten Version genügt `--catalog … --app desk --entry … --package-url …`.
4. Signieren (fragt das Passwort ab):
```powershell
.\target\release\daedyx-catalog.exe sign --catalog "D:\Katalog\catalog.json" --key "$env:USERPROFILE\.daedyx\catalog\catalog.key"
```
5. Prüfen (optional): `daedyx-catalog verify --catalog "D:\Katalog\catalog.json" --public "$env:USERPROFILE\.daedyx\catalog\catalog.pub"`
6. Im Launcher auf **Aktualisieren** drücken. Desk steht als „Update verfügbar“ da; läuft Desk gerade, wartet der Launcher, bis Desk geschlossen ist.

## Wichtig
- **Reihenfolge:** Nach jedem `add-release` ist die alte Signatur ungültig (das Programm löscht sie); erst danach neu signieren.
- **Schon installiertes Desk:** Der Launcher findet eine von Hand installierte Desk-Version über den Uninstall-Eintrag in der Registry und aktualisiert sie an Ort und Stelle. Dafür muss der Katalogeintrag `--registry-key "Daedyx Desk"` haben.
- **Sperre:** Während der Launcher installiert, liegt `%LOCALAPPDATA%\Daedyx\Launcher\locks\desk.lock`. Ein eigener Updater von Desk soll in dieser Zeit nicht starten.

## Seit L4: Updates von selbst (öffentliches Repository `daedyx/releases`)
- **Öffentlicher Schlüssel einbauen (einmalig):** den Inhalt von `catalog.pub` nach `crates/launcher-core/assets/catalog.pub` kopieren (ersetzt den Platzhalter), einchecken, Launcher neu bauen. Danach braucht niemand mehr `config set`; die Einstellungen bleiben nur als Überschreibung.
- **Eingebaute Adresse:** `https://raw.githubusercontent.com/daedyx/releases/main` (Katalog `catalog.json` samt Signatur und Pakete als Release-Dateien).
- **Pipeline (ohne Rückfrage signieren):** `DAEDYX_CATALOG_KEY` = Inhalt der Schlüsseldatei (GitHub-Secret direkt als Umgebungsvariable), `DAEDYX_KEY_PASSWORD` = Passwort (als geheime Werte der Pipeline, nie in Git), dann `daedyx-catalog sign --catalog catalog.json --key-env DAEDYX_CATALOG_KEY --public catalog.pub`.
- **Launcher-Release bauen:** `scripts\build-setup.ps1` (mit `DAEDYX_REQUIRE_REAL_KEY=1` bricht es beim Platzhalter-Schlüssel ab); `launcher-catalog-entry.json` per `add-release` als App `launcher` in den Katalog.
