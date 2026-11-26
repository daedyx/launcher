# Daedyx Launcher

Installiert und aktualisiert die Apps der Daedyx-Suite. Rust. Windows zuerst, dann macOS.

Seit 0.2.0 (U3) prüft der Launcher jedes Update mit der gemeinsamen Update-Bibliothek `daedyx-update` (Kern
`daedyx/core`): Eintrag `entry.json` und GitHub-Nachweis aus dem Release der App, dazu Größe und SHA-256 des Setups.
Die Quellen stehen fest im Programm (`crates/launcher-core/src/apps.rs`); es gibt keinen Katalog, keinen Schlüssel
und nichts einzustellen. Siehe `docs/STAND.md`.

## Ausprobieren
```powershell
cargo test --workspace --all-features
```
Die Tests laufen ohne Netz gegen einen gefälschten Server und eine Attrappe für den Nachweis.

Die Kommandozeile (`daedyx`):

```powershell
daedyx list
daedyx install desk
daedyx update
daedyx remove desk
daedyx self-update --check
```
Mit `--json` kommt die Ausgabe maschinenlesbar, mit `--data-dir <Ordner>` ein anderer Datenordner.

## Lizenz
MIT (siehe `LICENSE`), Fremdbestandteile in `THIRD-PARTY-LICENSES.md` (`python scripts/third-party-licenses.py`).
