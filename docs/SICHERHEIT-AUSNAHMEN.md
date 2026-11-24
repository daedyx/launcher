# Sicherheits-Ausnahmen (daedyx-launcher)

Jede Ausnahme von einer Regel aus `docs/SICHERHEIT.md` steht hier: `#[allow(...)]` im Code, `ignore` in
`deny.toml`, `#[ignore]` an Tests, `unsafe`. Ohne Eintrag ist die Ausnahme nicht erlaubt. Dazu zählen auch
Lockerungen in `deny.toml`, `clippy.toml`, `.cargo/config.toml` und `.cargo/audit.toml`. Die CI sucht die Stellen
(`cargo xtask exceptions`) und vergleicht sie mit dieser Tabelle; eine Stelle ohne passende Zeile, eine Zeile ohne
Stelle oder ein abgelaufener Eintrag sind rot. Eine Datei mit zwei Stellen bekommt zwei Zeilen.

**Programmstarts:** Der Launcher installiert und startet Programme; das ist seine Aufgabe. Die Regel 2.7 sieht
dafür später ein Crate `daedyx-sys` vor (Gerüst M2). Bis dahin sind die Startstellen hier einzeln eingetragen.
Gemeinsam ist allen: festes Programm aus eigenen Pfaden (geprüft in `launch.rs`/`installer.rs`), keine Shell,
keine Fremdtexte in der Befehlszeile.

| Nr. | Datei | Regel | Grund | Entscheid von | Datum | Ablauf |
|---|---|---|---|---|---|---|
| A-001 | xtask/src/files.rs | clippy::disallowed_methods | Das Prüfwerkzeug startet `git` mit festen Argumenten, um die versionierten bzw. vorgemerkten Dateien zu lesen. Läuft nur auf dem Entwicklerrechner und in CI. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-11-23 |
| A-002 | crates/launcher-core/src/installer.rs | clippy::disallowed_methods | NSIS-Setup einer App still starten (`/S /D=<eigener Ordner>`), Paket vorher per SHA-256 und Größe geprüft. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-002 | crates/launcher-core/src/installer.rs | clippy::disallowed_methods | NSIS-Deinstaller `uninstall.exe` aus dem eigenen Installationsordner starten. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-003 | crates/launcher-core/src/launch.rs | clippy::disallowed_methods | Installierte App starten; Startprogramm aus `app.json` nur mit normalen Pfadteilen ohne `:` (Prüfung `launch_target`). | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-004 | crates/launcher-core/src/system.rs | clippy::disallowed_methods | Unabhängiger Start (Selbst-Update, Autostart) mit festen Argumenten. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-005 | crates/launcher-core/src/system_tests.rs | clippy::disallowed_methods | Nur Test: startet eine Kopie von `PING.EXE` als laufendes Programm im Testordner. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-11-23 |
| A-006 | crates/launcher-cli/src/bin/fake_installer.rs | clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::print_stdout, clippy::print_stderr, clippy::arithmetic_side_effects, clippy::exit, clippy::disallowed_methods (ganze Datei) | Test-Installer, nur mit Merkmal `test-helpers`, nicht Teil der Veröffentlichung. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-11-23 |
| A-007 | crates/launcher-cli/src/bin/demo_catalog.rs | clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::print_stdout, clippy::print_stderr, clippy::arithmetic_side_effects, clippy::exit, clippy::disallowed_methods (ganze Datei) | Demo-Katalog mit erfundenen Apps für die Sichtprüfung, nur mit Merkmal `test-helpers`. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-11-23 |
| A-008 | crates/launcher-cli/tests/e2e.rs | clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::print_stdout, clippy::print_stderr, clippy::arithmetic_side_effects, clippy::exit, clippy::disallowed_methods (ganze Datei) | Ende-zu-Ende-Test; Hilfsfunktionen außerhalb von `#[test]` gelten für Clippy nicht als Test. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-11-23 |
| A-009 | crates/launcher-cli/tests/catalog_tool.rs | clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::print_stdout, clippy::print_stderr, clippy::arithmetic_side_effects, clippy::exit, clippy::disallowed_methods (ganze Datei) | wie A-008 | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-11-23 |
| A-010 | crates/launcher-cli/src/main.rs | clippy::print_stdout, clippy::print_stderr | Kommandozeilen-Programm: Ausgabe auf der Konsole ist seine Aufgabe. Fehlertexte können eine Katalog-Adresse enthalten (öffentlich, kein Personenbezug). | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-011 | crates/launcher-cli/src/bin/daedyx_catalog/main.rs | clippy::print_stdout, clippy::print_stderr | Katalog-Werkzeug für Veröffentlichungen, Konsolen-Ausgabe ist seine Aufgabe. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-012 | crates/launcher-ui/src/lib.rs | clippy::arithmetic_side_effects, clippy::cast_possible_truncation, clippy::cast_sign_loss (ganze Oberfläche) | Rechnen mit Bildschirmmaßen, Farben und Pixeln aus festen Werten, keine Fremddaten. Umbau gehört zur gemeinsamen Oberfläche `daedyx-ui` (Gerüst M2), nicht zu diesem Auftrag („ohne Oberflächen-Umbau“). Die Absturz-Regeln gelten in der Oberfläche voll. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-013 | crates/launcher-ui/src/i18n.rs | clippy::expect_used | Eingebaute Sprachdateien; dass sie gültig sind, belegen die Tests für alle Sprachen. Ein Fehler wäre ein Programmierfehler vor jeder Veröffentlichung (SICHERHEIT 2.2, Ausnahme). | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-014 | crates/launcher-ui/src/bin/make_icon.rs | clippy::print_stdout, clippy::print_stderr | Entwicklerwerkzeug zum Erzeugen der Symboldatei, Konsolen-Ausgabe ist seine Aufgabe. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-11-23 |
| A-015 | xtask/src/main.rs | clippy::print_stdout, clippy::print_stderr | Das Prüfwerkzeug meldet Datei, Zeile und Regel auf der Konsole, nie den gefundenen Text. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-11-23 |
| A-016 | crates/launcher-cli/src/bin/daedyx_catalog/main.rs | clippy::too_many_arguments | Stil-Regel ohne Sicherheitswirkung, schon vor dem Rahmen vorhanden (Funktion `add` des Katalog-Werkzeugs). | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-017 | crates/launcher-ui/src/focus.rs | deprecated | Schon vor dem Rahmen vorhanden: veraltete egui-Schnittstelle für die Ebene des Fokusrings; Ersatz mit dem nächsten egui-Wechsel. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-06-30 |
| A-018 | .github/workflows/release.yml | scan:secret-im-workflow | Alter Release-Ablauf mit Signaturschlüssel, Passwort und Token als GitHub-Secrets (SICHERHEIT 8.1 Nr. 7). Wird mit dem Plan „ohne Secrets“ (Attestierung) ersetzt; bis dahin keine neuen Secrets. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-02-28 |
| A-019 | deny.toml | RUSTSEC-2026-0192 (ttf-parser nicht gepflegt) | Kommt über egui/winit (Schriften), keine bekannte Lücke, kein Ersatz verfügbar. Neu prüfen mit dem nächsten egui-Wechsel der Suite. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-02-28 |
| A-020 | deny.toml | RUSTSEC-2026-0370 (zvariant nicht gepflegt) | Nur Linux-Barrierefreiheit über egui/accesskit, keine bekannte Lücke. | wie A-019 | 2026-11-23 | 2027-02-28 |
| A-021 | deny.toml | RUSTSEC-2026-0371 (zbus_names nicht gepflegt) | wie A-020 | wie A-019 | 2026-11-23 | 2027-02-28 |
| A-022 | deny.toml | RUSTSEC-2026-0372 (zvariant_derive nicht gepflegt) | wie A-020 | wie A-019 | 2026-11-23 | 2027-02-28 |
| A-023 | deny.toml | licenses.exceptions=epaint_default_fonts:OFL-1.1+Ubuntu-font-1.0 (Schrift-Lizenzen) | Die eingebauten egui-Schriften stehen unter Schrift-Lizenzen; sie erlauben Weitergabe mit dem Programm und sind mit MIT verträglich. Gilt nur für dieses eine Crate. | Bau-Chat, Ja des Betreuers 2026-11-24 | 2026-11-23 | 2027-11-23 |
| A-024 | deny.toml | bans.allow-wildcard-paths=true | Eigene Crates im Workspace hängen nur per Pfad voneinander ab (`publish = false`); cargo-deny würde diese Pfad-Abhängigkeiten sonst als Platzhalter-Version melden. Fremde Crates bleiben von `wildcards = "deny"` erfasst. Bisher nicht gezählt, erst der neue Zähler (Review M-2) macht die Lockerung sichtbar. | Bau-Chat Prüflücken, Ja des Betreuers ausstehend | 2026-11-24 | 2027-06-30 |

Eine Zeile mit Regel `scan:<regel>` erlaubt genau diesen Fund des Scans in genau dieser Datei bis zum
Ablaufdatum; sie zählt nicht als Code-Stelle. Gedacht nur für Altlasten, deren Ablösung beschlossen ist.

Format für `cargo xtask exceptions`: Spalte 2 ist der Pfad (genau wie im Repo), Spalte 3 die Regel, Spalte 6 das
Eintragsdatum und Spalte 7 der Ablauf, beide `JJJJ-MM-TT`. Eine Datei mit zwei Ausnahmen bekommt zwei Zeilen.

Verglichen werden **Datei und Regel**, nicht nur die Anzahl. Die Regel ist der Text vor einer Erläuterung in Klammern
` (…)`; Groß-/Kleinschreibung, Leerzeichen und Backticks zählen nicht, mehrere Regeln eines Attributs stehen mit Komma
in einer Zeile (Reihenfolge egal). Mögliche Regeln:
- Rust: die Lints eines `allow`/`expect` (auch in `cfg_attr`, über mehrere Zeilen), z. B. `clippy::print_stdout,
  clippy::print_stderr`; `ignore`; `unsafe`.
- `deny.toml`: die Kennung `RUSTSEC-…` einer Meldung; `advisories.ignore=<crate>`;
  `licenses.exceptions=<crate>:<Lizenz>+<Lizenz>`; jede Abweichung von der Grundliste im Werkzeug als
  `<abschnitt>.<schlüssel>=<wert>`, z. B. `advisories.yanked=warn`, `sources.allow-git=<adresse>`,
  `licenses.allow=<lizenz>`, `bans.skip=<crate>`, `bans.skip-tree=<crate>`, `bans.deny-fehlt=<crate>`.
- `clippy.toml`: `<schlüssel>=<wert>` außerhalb der Grundliste, `disallowed-methods-fehlt=<pfad>`.
- `.cargo/config.toml`: `build.rustflags` u. ä., `alias.<name>`; `.cargo/audit.toml`: alles außer `ignore`;
  `Cargo.toml`: zusätzliche `workspace.lints.<werkzeug>.<lint>=allow`.

Ohne Ausnahme-Weg rot: ein Crate-Manifest ohne `[lints] workspace = true` oder mit eigenen Lint-Stufen, Workspace-Lints
schwächer als die Grundliste, eine zweite `clippy.toml`. Der Ablauf liegt höchstens 12 Monate nach dem Eintragsdatum,
das Eintragsdatum nicht in der Zukunft.
