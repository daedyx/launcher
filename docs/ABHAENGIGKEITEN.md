# Abhängigkeiten (daedyx-launcher)

Jede direkte Abhängigkeit mit Zweck, Lizenz und Prüfung (SICHERHEIT 4.1). Neue Versionen erst, wenn sie
mindestens 14 Tage alt sind (Ausnahme: Sicherheits-Updates, dann Änderungen lesen). Lizenz und Versionen
unten stammen aus `cargo metadata --locked` (2026-11-23), nicht aus dem Gedächtnis. „Bestand“ heißt: vor dem
Sicherheitsrahmen aufgenommen, Zweck und Pflegezustand noch nicht einzeln nachgetragen.

| Crate (Version im Lockfile) | Benutzt von (Art) | Zweck | Lizenz | Bauskript/Makro |
|---|---|---|---|---|
| daedyx-update (Kern `daedyx/core`, Tag `daedyx-update-v0.1.0`; in der Korrekturrunde U3 per `rev` auf Zweig `paket-u3-net` mit Merkmal `net`, bis zum nächsten Kern-Tag) | launcher-core | Eintrag, GitHub-Nachweis (Sigstore), Version, geprüfter Download, Sperre, Zustand; dieselbe Bibliothek wie in Tasks und Desk (U3). Bringt `attestation-verify` =0.1.0 und `x509-cert` 0.3.1 mit (Kern-Ausnahmen U-A1, U-A2) | MIT | – |
| clap 4.6.7 | launcher-cli, launcher-ui | Befehlszeilen-Argumente | MIT OR Apache-2.0 | derive-Makro |
| dirs 7.0.0 | launcher-core | Standard-Ordner des Systems | MIT OR Apache-2.0 | – |
| eframe 0.37.0 | launcher-ui | Fenster und Oberfläche (egui) | MIT OR Apache-2.0 | – |
| egui_kittest 0.37.0 | launcher-ui (Test) | Oberflächen-Tests | MIT OR Apache-2.0 | – |
| serde 1.0.229 | alle | Typen zu/aus JSON | MIT OR Apache-2.0 | `build.rs`, derive-Makro |
| serde_json 1.0.154 | alle | JSON lesen und schreiben | MIT OR Apache-2.0 | `build.rs` |
| sha2 0.11.1 | launcher-core | Zweite Prüfsumme über den gesperrten Griff (`held.rs`, Review U3 M-U3-1); in Tests Prüfsummen der erfundenen Setups | MIT OR Apache-2.0 | – |
| sys-locale 0.3.2 | launcher-ui | Systemsprache | MIT OR Apache-2.0 | – |
| sysinfo 0.40.3 | launcher-core | Läuft eine App noch? (Prozessliste) | MIT | – |
| tempfile 3.27.0 | Tests | Temporäre Ordner in Tests | MIT OR Apache-2.0 | – |
| thiserror 2.0.21 | launcher-core | Fehlertypen | MIT OR Apache-2.0 | `build.rs`, derive-Makro |
| ureq 3.5.2 (indirekt über `daedyx-update` mit Merkmal `net`) | launcher-core (`remote.rs`) | `entry.json`, Nachweis und Setups über https laden. Seit der Korrekturrunde U3 (F-U3-1) kommt das Netz aus dem Kern (`daedyx_update::net`), dieselbe Datei wie in `daedyx-updater`; die Abschrift `net.rs` im Launcher ist weg | MIT OR Apache-2.0 | – |
| winreg 0.56.0 | launcher-core, launcher-ui (Windows) | Deinstallations-Einträge, Autostart | MIT | – |
| xtask (eigenes) | – | Prüfwerkzeug, nur Standardbibliothek | MIT | – |

## Änderungen
- 2026-11-26 (U3, Zweig `launcher-u3`): Der eigene Crate `daedyx-update` mit Katalog, Katalog-Werkzeug und
  minisign-Prüfung ist gelöscht (Entscheid F1); an seine Stelle tritt die gemeinsame Bibliothek aus dem Kern.
  Entfallen: `minisign`, `minisign-verify`, `rpassword`, `semver`, `hex` als direkte Abhängigkeiten. `ureq` ohne
  gzip. Neue Pakete der Nachweis-Kette kommen über den Kern; sie stehen schon in den Locks von Tasks und Desk.
- 2026-11-23: `url` (indirekt über egui-winit/webbrowser) von 2.5.8 auf 2.5.9 angehoben, Sicherheits-Update
  RUSTSEC-2026-0592; die Version ist erst 3 Tage alt (Ausnahme der Alter-Regel für Sicherheits-Updates).
  Gelesen: Änderungsliste 2.5.8…2.5.9 im Repository servo/rust-url (Fehlerbehebungen, blob:-Herkunft, keine
  neuen Abhängigkeiten außer form_urlencoded 1.2.3 aus demselben Repository).
