# Daedyx Launcher

Installiert und aktualisiert die Apps der Daedyx-Suite aus einem signierten Katalog. Rust. Windows zuerst, dann macOS.

**Stand:** Meilenstein L1 (Kern und Kommandozeile) ist fertig, noch ohne Oberfläche. Siehe `docs/STAND.md`.

## Ausprobieren
```powershell
cargo test --workspace --features launcher-cli/test-helpers
```
Die Tests legen einen Katalog mit Test-Apps an, signieren ihn mit einem Testschlüssel und führen Installation, Update und Entfernen mit der echten Kommandozeile aus.

Die Kommandozeile (`daedyx`) braucht einen Katalog (Ordner oder Adresse) und den öffentlichen Schlüssel:

```powershell
daedyx --catalog <Ordner-oder-Adresse> --pubkey <pub.key> list
daedyx --catalog <...> --pubkey <...> install desk
daedyx --catalog <...> --pubkey <...> update
daedyx --catalog <...> --pubkey <...> remove desk
```
Mit `--json` kommt die Ausgabe maschinenlesbar.

## Lizenz
Noch nicht festgelegt.
