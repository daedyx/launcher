# Abhängigkeiten (daedyx-launcher)

Jede direkte Abhängigkeit mit Zweck, Lizenz und Prüfung (SICHERHEIT 4.1). Neue Versionen erst, wenn sie
mindestens 14 Tage alt sind (Ausnahme: Sicherheits-Updates, dann Änderungen lesen). Lizenz und Versionen
unten stammen aus `cargo metadata --locked` (2026-11-23), nicht aus dem Gedächtnis. „Bestand“ heißt: vor dem
Sicherheitsrahmen aufgenommen, Zweck und Pflegezustand noch nicht einzeln nachgetragen.

| Crate (Version im Lockfile) | Benutzt von (Art) | Zweck | Lizenz | Bauskript/Makro |
|---|---|---|---|---|
| clap 4.6.7 | launcher-cli, launcher-ui | Befehlszeilen-Argumente | MIT OR Apache-2.0 | derive-Makro |
| dirs 7.0.0 | launcher-core | Standard-Ordner des Systems | MIT OR Apache-2.0 | – |
| eframe 0.37.0 | launcher-ui | Fenster und Oberfläche (egui) | MIT OR Apache-2.0 | – |
| egui_kittest 0.37.0 | launcher-ui (Test) | Oberflächen-Tests | MIT OR Apache-2.0 | – |
| hex 0.4.3 | daedyx-update | Prüfsummen als Text | MIT OR Apache-2.0 | – |
| minisign 0.10.0 | launcher-cli; Tests | Katalog signieren (Werkzeug), Testschlüssel | MIT | – |
| minisign-verify 0.3.0 | daedyx-update | Signatur des Katalogs prüfen | MIT | – |
| rpassword 7.5.5 | launcher-cli | Passwort verdeckt abfragen (Katalog-Werkzeug) | Apache-2.0 | – |
| semver 1.0.28 | daedyx-update | Versionen vergleichen | MIT OR Apache-2.0 | – |
| serde 1.0.229 | alle | Typen zu/aus JSON | MIT OR Apache-2.0 | `build.rs`, derive-Makro |
| serde_json 1.0.154 | alle | JSON lesen und schreiben | MIT OR Apache-2.0 | `build.rs` |
| sha2 0.11.1 | daedyx-update | SHA-256 der Pakete | MIT OR Apache-2.0 | – |
| sys-locale 0.3.2 | launcher-ui | Systemsprache | MIT OR Apache-2.0 | – |
| sysinfo 0.40.3 | launcher-core | Läuft eine App noch? (Prozessliste) | MIT | – |
| tempfile 3.27.0 | Tests | Temporäre Ordner in Tests | MIT OR Apache-2.0 | – |
| thiserror 2.0.21 | daedyx-update, launcher-core | Fehlertypen | MIT OR Apache-2.0 | `build.rs`, derive-Makro |
| ureq 3.5.2 | launcher-core | Katalog und Pakete über https laden | MIT OR Apache-2.0 | – |
| winreg 0.56.0 | launcher-core, launcher-ui (Windows) | Deinstallations-Einträge, Autostart | MIT | – |
| xtask (eigenes) | – | Prüfwerkzeug, nur Standardbibliothek | MIT | – |

## Änderungen
- 2026-11-23: `url` (indirekt über egui-winit/webbrowser) von 2.5.8 auf 2.5.9 angehoben, Sicherheits-Update
  RUSTSEC-2026-0592; die Version ist erst 3 Tage alt (Ausnahme der Alter-Regel für Sicherheits-Updates).
  Gelesen: Änderungsliste 2.5.8…2.5.9 im Repository servo/rust-url (Fehlerbehebungen, blob:-Herkunft, keine
  neuen Abhängigkeiten außer form_urlencoded 1.2.3 aus demselben Repository).
