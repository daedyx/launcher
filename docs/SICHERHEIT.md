# Sicherheit (daedyx-launcher)

Gekürzte Fassung des verbindlichen Sicherheitsrahmens der Daedyx-Suite für dieses Repository. Die Mutter-Fassung
liegt bei Daedyx Desk (`docs/SICHERHEIT.md`); bei Widerspruch gilt sie. Stand: 2026-11-23.

Grundsatz Björn: Keine Lücken, keine persönlichen Daten. Fehlerklassen werden durch Bauweise unmöglich gemacht,
alles Übrige vor jedem Merge automatisch gestoppt. Eine Garantie gibt es nicht; ehrlich bleiben.

## Kurzfassung (wortgleich in jeder CLAUDE.md)
- **Kein `unsafe`** (Workspace-Lint `forbid`). Ausnahme nur im Crate `daedyx-sys`, mit `// SAFETY:` und Eintrag in `docs/SICHERHEIT-AUSNAHMEN.md`.
- **Keine Panik im Normalpfad:** kein `unwrap`/`expect`/`panic!`/Index `a[i]`/Text-Schnitt; Clippy verbietet es (Tests ausgenommen). Panik-Texte ohne Daten.
- **Jede Eingabe von außen hat Grenzen** (Größe, Tiefe/Anzahl, Zeit) aus dem Modul `limits`. Jeder Parser hat ein Fuzz-Ziel und proptest. Missbrauchstests zuerst.
- **Netz nur über `daedyx-net`/`NetGate`** (nur https, Zeit- und Größengrenze, Zweck). Keine Telemetrie. Fremdbilder aus, Links nur nach Bestätigung, Anhänge nie öffnen.
- **Keine Shell, keine Prozesse mit Fremddaten;** Prozesse nur über `daedyx-sys`. Schreiben nur in den eigenen Datenordner oder an einen im Dialog gewählten Pfad (`UserChosenPath`).
- **Daten verschlüsselt** (SQLCipher), Schlüssel und Passwörter nur im Betriebssystem-Schlüsselbund, im Programm nur als `Secret` (kein Debug, zeroize). Keine Klartext-Kopien.
- **Keine Protokolle mit Inhalten, Namen, Adressen, Pfaden.** Kein println/eprintln/dbg. Diagnose nur als Codes und Zahlen.
- **Keine echten Daten** in Code, Tests, Commits, CI-Ausgaben, Bildschirmfotos, Chats. Testdaten nur aus `daedyx-fixtures` (example.com, .test, 192.0.2.x). Bildschirmfotos nur über `DAEDYX_SCREENSHOT` im Demo-Modus.
- **Keine Secrets** in Repos und Workflows; Veröffentlichung nur über Attestierung (`plan-ohne-secrets`). Neue Abhängigkeit nur mit Eintrag in `docs/ABHAENGIGKEITEN.md`, Version mindestens 14 Tage alt.
- **Nie eine Prüfung abschalten, um grün zu werden** (kein `#[allow]`, `ignore`, `#[ignore]`, `--no-verify`, `skip_tests` ohne Ausnahme-Eintrag und Björns Ja). Sicherheitsdateien (deny.toml, clippy.toml, [lints], Workflows, xtask scan) nur mit Björns Ja ändern.
- **Vor dem Bau:** Abschnitt im `docs/BEDROHUNGSMODELL.md`. **Vor dem Merge:** `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --locked`, `cargo deny check`, `cargo audit`, `cargo xtask scan`, dazu Sicherheits-Review durch einen anderen Chat. Im Abschlussbericht jede Prüfung mit Ergebnis nennen oder „ungeprüft“ schreiben.

## Was der Launcher von außen bekommt
- **Katalog** (`apps.json` und Signatur) aus einem Ordner oder über https: erst minisign-Signatur, dann lesen;
  höchstens 5 MB (`source.rs`).
- **Pakete** (Setups): Größe und SHA-256 aus dem signierten Katalog, erst danach wird installiert.
- **`app.json`** im Installationsordner einer App, **Registry-Einträge** der Deinstallation, eigene Dateien
  (`config.json`, `ui-settings.json`, Zustand) im Datenordner, **Befehlszeilen-Argumente**.

Der Launcher installiert und startet Programme; das ist seine Aufgabe. Jede Startstelle ist in
`docs/SICHERHEIT-AUSNAHMEN.md` eingetragen (feste Programme aus eigenen Pfaden, keine Shell).

## Was in diesem Repository erzwungen ist
| Regel | Wie | Wo |
|---|---|---|
| Kein `unsafe` | `unsafe_code = "forbid"` (kompiliert nicht) | `Cargo.toml` `[workspace.lints]` |
| Keine Panik, keine Ausgaben | Clippy `deny`: unwrap, expect, panic, Index, Text-Schnitt, println/eprintln/dbg, exit u. a.; in Tests erlaubt | `Cargo.toml`, `clippy.toml` |
| Rechnen ohne Überlauf | Clippy `arithmetic_side_effects`/Umwandlungen, in CI rot; Oberfläche ausgenommen (A-012) | `Cargo.toml` |
| Prozesse, Temp, Netz nur bewusst | `disallowed-methods`, jede Stelle eingetragen | `clippy.toml` |
| Lieferkette | `cargo deny` (Lücken, Lizenzen, Quellen, Verbote) und `cargo audit` | `deny.toml`, `.cargo/audit.toml` |
| Keine Geheimnisse, keine echten Daten | `cargo xtask scan` (Pre-Commit nur Vorgemerktes, CI alles) | `xtask/`, `.githooks/pre-commit` |
| Ausnahmen nur mit Eintrag und Ablauf | `cargo xtask exceptions` | `docs/SICHERHEIT-AUSNAHMEN.md` |
| Alles bei jedem Push und Pull Request | GitHub Actions, keine Secrets, nur Lesen | `.github/workflows/ci.yml` |

Was der Scan meldet und wie die lokale Wortliste funktioniert, steht in `xtask/src/patterns.rs` und
`xtask/src/wordlist.rs` (gleiches Werkzeug wie in `daedyx/core`).
Gelesen wird jede Datei als Text: UTF-16 mit BOM (LE und BE) als UTF-16, sonst UTF-8 mit Ersatzzeichen für
ungültige Bytes (z. B. Windows-1252). Als binär gelten nur Bild- und Schrift-Endungen (`png`, `jpg`, `jpeg`, `gif`,
`bmp`, `webp`, `ico`, `icns`, `ttf`, `otf`, `woff`, `woff2`); jede andere Datei mit NUL-Byte meldet
`unbekannte-binaerdatei` und wird trotzdem zeilenweise geprüft.

`cargo xtask exceptions` sucht Ausnahmen und Lockerungen und vergleicht Datei **und Regel** mit
`docs/SICHERHEIT-AUSNAHMEN.md` (Einzelheiten dort): `allow`/`expect` auch in `cfg_attr`, mit Leerzeichen und über
mehrere Zeilen, `#[ignore]`, `unsafe`; in `deny.toml` jede Abweichung von der festgehaltenen Grundliste (u. a.
`yanked`, `unmaintained`, `unknown-git`, `allow-git`, neue Lizenzen, `skip`/`skip-tree`); Lockerungen in
`clippy.toml`, `.cargo/config.toml` und `.cargo/audit.toml`. Jedes Crate muss `[lints] workspace = true` haben,
und eine Ausnahme läuft höchstens 12 Monate.

## Einmal je Klon
```
git config core.hooksPath .githooks
```

## Prüfbefehle (vor jedem Merge, Ergebnis im Bericht nennen)
```
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo deny --locked check advisories bans licenses sources
cargo audit --deny warnings
cargo xtask scan
cargo xtask exceptions
```

## Meldungen von außen
Siehe `SECURITY.md`. Reaktionszeiten: Eingang binnen 7 Tagen; kritisch 7, hoch 30, mittel 90 Tage, niedrig mit
dem nächsten Release. Gleiche Fristen für neue RustSec-Meldungen zu unseren Abhängigkeiten.

## Bekannte Lücken und Offenes (ehrlich, nicht in diesem Paket behoben)
- **Download ohne Obergrenze während des Ladens** (`source.rs`, `copy_with_progress`): Die Größe wird erst nach
  dem Laden geprüft; ein feindlicher Server kann die Platte füllen. Behebung: Abbruch bei Katalog-Größe + 1 Byte.
- **Keine im Code gesetzte Zeitgrenze** für Netzabrufe (`ureq`, Voreinstellung ungeprüft).
- **Paket-Adressen dürfen auf jeden https-Rechner zeigen** (durch Signatur und SHA-256 entschärft).
- **Alter Release-Ablauf** `release.yml` nutzt Secrets und Actions ohne Commit-ID; wird mit dem Plan
  „ohne Secrets“ ersetzt (Scan-Ausnahme A-018 bis 2027-02-28).
- Fehlertexte der Kommandozeile reichen Fremdtexte weiter (`e.to_string()`, kann die Katalog-Adresse enthalten).
- Fuzz-Ziel und proptest für den Katalog (`daedyx-update/src/catalog.rs`) fehlen noch.
- `rust-toolchain.toml`, `gitleaks`, `zizmor`, Dependabot, CODEOWNERS, `overflow-checks` im Release-Profil:
  offen (Herunterladen bzw. Entscheidung nur mit Björns Ja).
- Repository-Einstellungen (Private vulnerability reporting, SHA-Pflicht für Actions, Branch-Schutz): nur Björn.
