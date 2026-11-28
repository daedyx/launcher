# Daedyx Launcher veröffentlichen — was Björn selbst tun muss

Stand: 2026-11-27, Auftrag R1 (Zweig `launcher-r1`), nach dem Muster von Tasks (T7v). Der Workflow
`.github/workflows/release.yml` ist ein **Entwurf**: Er ist nie gelaufen, und es ist nichts veröffentlicht. Er braucht
**keine Secrets, keine Schlüssel und keine Passwörter**. Signiert wird über einen Nachweis, den GitHub selbst ausstellt
(Sigstore). Der Launcher prüft beim Selbst-Update genau diesen Nachweis: Workflow `.github/workflows/release.yml`, Zweig
`main`, Repository `daedyx/launcher` mit festen Kennungen (`crates/launcher-core/src/apps.rs`).

Die alte Anleitung `docs/RELEASE-CI.md` (Katalog-Schlüssel, `RELEASES_TOKEN`, Katalog in `daedyx/releases`) gilt für
neue Releases nicht mehr. Für den Launcher legst du **keinen** Schlüssel und **kein** Token an.

## Wichtig für deinen installierten Launcher 0.1.0
Launcher 0.1.0 sucht Updates nur im alten, signierten Katalog (`daedyx/releases`). Der neue Workflow trägt dort nichts
mehr ein. Dein installierter 0.1.0 findet 0.2.0 deshalb **nicht von selbst**. Du installierst 0.2.0 **einmal von Hand**:
Setup `daedyx-launcher-0.2.0-setup.exe` aus dem neuen Release laden und drüber installieren (plan-ohne-secrets Weg B;
optional vorher `gh attestation verify <Datei> --repo daedyx/launcher`). Ab 0.2.0 kommen alle Launcher-Updates über
den Nachweis von selbst. Der alte Weg A (0.2.0 ein letztes Mal mit den alten Secrets in den Katalog) ist im Workflow
nicht mehr vorgesehen; die alten Skripte (`scripts/ci/publish.ps1`, `fetch-catalog-tool.ps1`, `set-version.ps1`,
`test-publish.ps1`) liegen nur noch für diesen Fall bereit und werden mit der Abschaltung des Katalogs gelöscht.

## Offene Punkte vor dem ersten Lauf
Solange L4 und L5 nicht belegt sind, kann der erste Lauf abbrechen, ohne etwas zu veröffentlichen.

| Punkt | Stand |
|---|---|
| L1 | **Geschlossen** (2026-11-27, nach Björns Ja zum Nachschlagen): `actions/attest-build-provenance` v4.2.2 = `4d101475d8b20a2381f78447822ac1eab6504dd8` (Release vom 2026-08-06). Sie ruft `actions/attest` selbst per Commit-ID auf: `508db95dd578ae2727ebd6217d5ba78e4fbda05d` (v4.2.1). Ob diese Fassung Rekor-v1-Einträge schreibt, wie die App sie verlangt, ist **ungeprüft**; das zeigt erst die Selbstprüfung im ersten Lauf (sie bricht sonst vor dem Veröffentlichen ab). Der Wächter-Test lehnt den Platzhalter aus Nullen jetzt ab (H-T7v-3). |
| L4 | NSIS auf dem GitHub-Läufer: Der Bau-Job läuft seit `fix-release-2` auf `windows-2022`. Laut dessen Runner-Image-Readme ist dort NSIS 3.10 vorinstalliert; `windows-2025` hat es nicht mehr (Release-Lauf 2 brach mit „makensis fehlt auf dem Läufer“ ab). Der erste Schritt des Bau-Jobs gibt `makensis /VERSION` aus und bricht ohne NSIS ab, vor jeder Freigabe. Ein Nachinstall ist nicht eingebaut (Entscheid des Masters). Die übrigen Jobs bleiben auf `windows-2025`. Belegt wird das erst im nächsten Release-Lauf. |
| L5 | Der Prüfschritt `gh attestation verify` (Schalter `--source-ref` und `--deny-self-hosted-runners`) ist **ungeprüft**. |
| L5b | Tag-Regel: Erlaubt nur dir eine Regel, Tags `v*` anzulegen, scheitert `gh release create` womöglich erst beim Tag, nach dem Hochladen. Dann kann ein **Entwurfs-Release** liegen bleiben (**ungeprüft**). Beim ersten Lauf ansehen und nur einen solchen Entwurf entfernen. |
| L6 | **Geschlossen** (2026-11-27): vier Jobs, jeder nur mit den Rechten für seine Aufgabe (Review T7v, N-T7v-4 Weg 1 und M-T7v-3). „bauen“ und „pruefen“ nur mit Leserecht, „bezeugen“ mit Freigabe und Kennung für den Nachweis, „veroeffentlichen“ nur mit `contents: write`. Programme aus dem Bau (das Prüfprogramm) laufen nur in „pruefen“, ohne Kennung und ohne Schreibrecht; die Jobs mit Rechten führen nichts aus der Übergabe aus. Übergabe mit `actions/upload-artifact` v7.0.2 = `cf430e030ddbb5b0abf93d22962f4752f3646cd9` (2026-10-07) und `actions/download-artifact` v8.0.2 = `9000827ccba6bdab643e8b6fd33ac0654aef8333` (2026-10-07), Prüfsummen aus dem Bau-Job werden gegengeprüft. |

## Was du selbst tust, Schritt für Schritt
Jeder Schritt ist ein eigener Klick-Weg. Mach ihn erst, wenn du ausdrücklich Ja dazu gesagt hast. Die Menünamen bei
GitHub können leicht anders heißen (**ungeprüft**, Stand der Vorbereitung).

### Schritt 1: Die Repositories öffentlich stellen
Der Nachweis von GitHub funktioniert ohne Bezahl-Tarif nur in öffentlichen Repositories. Ohne Lese-Token muss der Bau
außerdem den Kern frei abrufen können. Erst nach der Verlaufsprüfung und deinem Ja.
1. Öffne auf GitHub das Repository `daedyx/core`.
2. Klicke auf **Settings**.
3. Scrolle ganz nach unten zu **Danger Zone**.
4. Klicke auf **Change visibility** und dann auf **Change to public**.
5. Bestätige, wie GitHub es verlangt.
6. Mach dasselbe für `daedyx/launcher`.

### Schritt 2: Dem Workflow nur Leserechte als Standard geben
1. Im Repository `daedyx/launcher`: **Settings**, dann **Actions**, dann **General**.
2. Unter **Workflow permissions** wählst du **Read repository contents and packages permissions**.
3. Wenn es den Haken **Require actions to be pinned to a full-length commit SHA** gibt: setzen.
4. Klicke auf **Save**.

Der Release-Workflow bekommt die Rechte zum Veröffentlichen nur für sich selbst, nur für diesen Lauf.

### Schritt 3: Das Environment „release“ mit deiner Freigabe anlegen
So startet nichts, bevor du es erlaubst.
1. Im Repository `daedyx/launcher`: **Settings**, dann links **Environments**.
2. Klicke auf **New environment**.
3. Name: `release` (klein geschrieben, genau so). Klicke auf **Configure environment**.
4. Setze den Haken bei **Required reviewers** und trage dich selbst ein.
5. Lass **Prevent self-review** aus, sonst kannst du deinen eigenen Lauf nicht freigeben.
6. Bei **Deployment branches and tags** wählst du **Selected branches and tags** und fügst `main` hinzu.
7. Lege hier **keine** Secrets und **keine** Variablen an.
8. Klicke auf **Save protection rules**.

### Schritt 4 (empfohlen): Unveränderliche Releases einschalten
Dann lassen sich Tag und Dateien eines Releases nachträglich nicht mehr austauschen.
1. Im Repository `daedyx/launcher`: **Settings**, dann **General**.
2. Suche den Abschnitt **Releases** und schalte **Release immutability** ein (Name **ungeprüft**).

Hast du eine Regel, die nur dir erlaubt, Tags `v*` anzulegen: Der Workflow legt den Tag `v<Version>` selbst an.
Die Regel muss das zulassen, sonst scheitert der letzte Schritt (**ungeprüft**, beim ersten Lauf ansehen).

### Schritt 5: Version anheben lassen
Ein Bau-Chat hebt die Version in `Cargo.toml` (Abschnitt `[workspace.package]`) an und bringt das auf `main`. Für
den ersten Lauf steht dort schon `0.2.0`. Der Workflow schreibt nichts mehr in den Quellcode zurück. Er prüft nur, dass deine Eingabe dort steht.

### Schritt 6: Den Lauf starten
1. Im Repository `daedyx/launcher`: **Actions**, links **Release**.
2. Klicke auf **Run workflow**.
3. Bei **Branch** muss `main` stehen.
4. Trage die **Version** ein, zum Beispiel `0.2.0`.
5. Trage eine kurze **Änderungsnotiz** ein: eine Zeile, ohne Links, ohne echte Namen. Tippe sie am besten selbst;
   kopierter Text kann unsichtbare Zeichen enthalten, dann bricht der Lauf mit einer Meldung ab.
6. Tippe die Version bei **Bestätigung** noch einmal ein.
7. Klicke auf den grünen Knopf **Run workflow**.

### Schritt 7: Die Freigabe klicken
Der Lauf testet und baut zuerst, ohne Schreibrechte (ungefähr eine Stunde, **ungeprüft**). Danach wartet er auf dich.
1. Öffne den Lauf, der jetzt oben in der Liste steht. Er wartet mit dem Hinweis „Waiting for review“.
2. Klicke auf **Review deployments**.
3. Setze den Haken bei **release** und klicke auf **Approve and deploy**.

Danach laufen drei Jobs nacheinander, ohne weiteren Klick:
1. **Bezeugen:** Der Lauf prüft, dass die gebauten Dateien unverändert angekommen sind, und lässt GitHub Eintrag und
   Setup bezeugen.
2. **Prüfen:** Er prüft das Ergebnis genau so, wie jede installierte App es prüfen wird (Prüfweg des Kerns mit
   eingebauter Vertrauenswurzel), dazu noch mit `gh attestation verify`. Dieser Job hat keine Schreibrechte.
3. **Veröffentlichen:** Erst wenn beides passt, legt er das Release `v<Version>` an.

Schon vor dem Bau bricht er ab, wenn die Version nicht neuer ist als das bisherige neueste Release.

Deine Freigabe sitzt vor dem Bezeugen, weil dort die Kennung für den Nachweis entsteht (Empfehlung aus Review T7v).
Willst du lieber erst kurz vor dem Veröffentlichen klicken, sag es einem Bau-Chat; dann wandert `environment: release`
zum Job „veroeffentlichen“.

### Schritt 8: Nachsehen
1. Im Repository `daedyx/launcher`: rechts **Releases**.
2. Das neue Release `v<Version>` muss als **Latest** markiert sein und genau drei Dateien haben:
   `daedyx-launcher-<Version>-setup.exe`, `entry.json` und `provenance.sigstore.json`.
3. Wenn etwas fehlt oder rot ist: nichts löschen, einem Bau-Chat Bescheid sagen.

## Zwei Regeln für immer
- **Releases nie löschen, nur durch ein neueres ersetzen.** Wer ein Release gesehen hat, bekommt sonst dauerhaft
  „Quelle liefert ältere Version, bitte melden“ (Review U1-R, H-U1-3).
- **Kein Secret im Release-Weg.** Braucht ein neuer Schritt doch eines, entscheidest du das vorher schriftlich
  (plan-ohne-secrets (f)). Die CI (`ci.yml`, Vorprüfung) meldet in allen Workflows jedes Secret als Fehler.

## Hinweise für Bau-Chats
- Die Kennung im Eintrag (`app`) ist `daedyx-launcher` (`apps::LAUNCHER.update_app`). Das Setup im Release heißt
  `daedyx-launcher-<Version>-setup.exe`; `build-setup.ps1 -SkipBuild` baut es aus den Programmen des Bau-Schritts, **ohne**
  `-Probe` und ohne Merkmal `probe` (Wächter `release_tests.rs`).
- `entry.json` schreibt `scripts/ci/make-entry.ps1` (gleiche Datei wie in Tasks, im Workflow mit Windows PowerShell 5.1,
  wie im Test). Vor dem Veröffentlichen prüft `crates/launcher-core/examples/release_selbstpruefung.rs` die drei Dateien
  mit dem echten Prüfer des Launchers (`release_check::verify` mit `apps::LAUNCHER`).
- Die Wächter `release_tests.rs`, `release_jobs_tests.rs`, `release_flow_tests.rs`, `release_guard_tests.rs`
  (launcher-core) prüfen Pfad, Zweig, Repository, Auslöser, keine Secrets, gepinnte Aktionen, Rechte je Job und schicken
  ein erfundenes Release durch Suche und Download des Kerns.
- `release_check.rs` ist eine Abschrift aus Tasks (bis auf die Quelle). Gehört beim nächsten Kern-Tag in den Kern.
