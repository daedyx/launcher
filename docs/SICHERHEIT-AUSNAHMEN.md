# Sicherheits-Ausnahmen (daedyx-launcher)

Jede Ausnahme von einer Regel aus `docs/SICHERHEIT.md` steht hier: `#[allow(...)]` im Code, `ignore` in
`deny.toml`, `#[ignore]` an Tests, `unsafe`. Ohne Eintrag ist die Ausnahme nicht erlaubt. Die CI zählt die
Stellen im Code (`cargo xtask exceptions`) und vergleicht sie mit dieser Tabelle; mehr oder weniger Stellen als
Einträge oder ein abgelaufener Eintrag sind rot. Eine Datei mit zwei Stellen bekommt zwei Zeilen.

**Programmstarts:** Der Launcher installiert und startet Programme; das ist seine Aufgabe. Die Regel 2.7 sieht
dafür später ein Crate `daedyx-sys` vor (Gerüst M2). Bis dahin sind die Startstellen hier einzeln eingetragen.
Gemeinsam ist allen: festes Programm aus eigenen Pfaden (geprüft in `launch.rs`/`installer.rs`), keine Shell,
keine Fremdtexte in der Befehlszeile.

| Nr. | Datei | Regel | Grund | Entscheid von | Datum | Ablauf |
|---|---|---|---|---|---|---|
| A-001 | xtask/src/files.rs | clippy::disallowed_methods | Das Prüfwerkzeug startet `git` mit festen Argumenten, um die versionierten bzw. vorgemerkten Dateien zu lesen. Läuft nur auf dem Entwicklerrechner und in CI. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-11-23 |
| A-002 | crates/launcher-core/src/installer.rs | clippy::disallowed_methods | NSIS-Setup einer App still starten (`/S /D=<eigener Ordner>`), Paket vorher per SHA-256 und Größe geprüft. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-002 | crates/launcher-core/src/installer.rs | clippy::disallowed_methods | NSIS-Deinstaller `uninstall.exe` aus dem eigenen Installationsordner starten. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-003 | crates/launcher-core/src/launch.rs | clippy::disallowed_methods | Installierte App starten; Startprogramm aus `app.json` nur mit normalen Pfadteilen ohne `:` (Prüfung `launch_target`). | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-004 | crates/launcher-core/src/system.rs | clippy::disallowed_methods | Unabhängiger Start (Selbst-Update, Autostart) mit festen Argumenten. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-005 | crates/launcher-core/src/system_tests.rs | clippy::disallowed_methods | Nur Test: startet eine Kopie von `PING.EXE` als laufendes Programm im Testordner. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-11-23 |
| A-006 | crates/launcher-cli/src/bin/fake_installer.rs | Absturz- und Ausgabe-Regeln (Datei) | Test-Installer, nur mit Merkmal `test-helpers`, nicht Teil der Veröffentlichung. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-11-23 |
| A-007 | crates/launcher-cli/src/bin/demo_catalog.rs | Absturz- und Ausgabe-Regeln (Datei) | Demo-Katalog mit erfundenen Apps für die Sichtprüfung, nur mit Merkmal `test-helpers`. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-11-23 |
| A-008 | crates/launcher-cli/tests/e2e.rs | Absturz- und Ausgabe-Regeln (Datei) | Ende-zu-Ende-Test; Hilfsfunktionen außerhalb von `#[test]` gelten für Clippy nicht als Test. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-11-23 |
| A-009 | crates/launcher-cli/tests/catalog_tool.rs | Absturz- und Ausgabe-Regeln (Datei) | wie A-008 | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-11-23 |
| A-010 | crates/launcher-cli/src/main.rs | clippy::print_stdout, clippy::print_stderr | Kommandozeilen-Programm: Ausgabe auf der Konsole ist seine Aufgabe. Fehlertexte können eine Katalog-Adresse enthalten (öffentlich, kein Personenbezug). | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-011 | crates/launcher-cli/src/bin/daedyx_catalog/main.rs | clippy::print_stdout, clippy::print_stderr | Katalog-Werkzeug für Veröffentlichungen, Konsolen-Ausgabe ist seine Aufgabe. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-012 | crates/launcher-ui/src/lib.rs | arithmetic_side_effects, cast_possible_truncation, cast_sign_loss (ganze Oberfläche) | Rechnen mit Bildschirmmaßen, Farben und Pixeln aus festen Werten, keine Fremddaten. Umbau gehört zur gemeinsamen Oberfläche `daedyx-ui` (Gerüst M2), nicht zu diesem Auftrag („ohne Oberflächen-Umbau“). Die Absturz-Regeln gelten in der Oberfläche voll. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-013 | crates/launcher-ui/src/i18n.rs | clippy::expect_used | Eingebaute Sprachdateien; dass sie gültig sind, belegen die Tests für alle Sprachen. Ein Fehler wäre ein Programmierfehler vor jeder Veröffentlichung (SICHERHEIT 2.2, Ausnahme). | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-014 | crates/launcher-ui/src/bin/make_icon.rs | clippy::print_stdout, clippy::print_stderr | Entwicklerwerkzeug zum Erzeugen der Symboldatei, Konsolen-Ausgabe ist seine Aufgabe. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-11-23 |
| A-015 | xtask/src/main.rs | clippy::print_stdout, clippy::print_stderr | Das Prüfwerkzeug meldet Datei, Zeile und Regel auf der Konsole, nie den gefundenen Text. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-11-23 |
| A-016 | crates/launcher-cli/src/bin/daedyx_catalog/main.rs | clippy::too_many_arguments | Stil-Regel ohne Sicherheitswirkung, schon vor dem Rahmen vorhanden (Funktion `add` des Katalog-Werkzeugs). | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-017 | crates/launcher-ui/src/focus.rs | deprecated | Schon vor dem Rahmen vorhanden: veraltete egui-Schnittstelle für die Ebene des Fokusrings; Ersatz mit dem nächsten egui-Wechsel. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-06-30 |
| A-018 | .github/workflows/release.yml | scan:secret-im-workflow | Alter Release-Ablauf mit Signaturschlüssel, Passwort und Token als GitHub-Secrets (SICHERHEIT 8.1 Nr. 7). Wird mit dem Plan „ohne Secrets“ (Attestierung) ersetzt; bis dahin keine neuen Secrets. | Bau-Chat, Björns Ja ausstehend | 2026-11-23 | 2027-02-28 |

Eine Zeile mit Regel `scan:<regel>` erlaubt genau diesen Fund des Scans in genau dieser Datei bis zum
Ablaufdatum; sie zählt nicht als Code-Stelle. Gedacht nur für Altlasten, deren Ablösung beschlossen ist.

Format für `cargo xtask exceptions`: Spalte 2 ist der Pfad (genau wie im Repo), Spalte 7 ein Datum `JJJJ-MM-TT`.
