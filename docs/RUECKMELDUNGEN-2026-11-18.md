# Rückmeldungen 2026-11-18 (FB Launcher)

Art: Fehler / Wunsch / Frage. Größe: klein (hier erledigt) / Paket (an den Master).

## Eigentest mit Demo-Katalog (Claude, 2026-11-18)
Demo-Katalog mit erfundenen Apps und Testschlüssel, Launcher im echten Fenster, Bedienung per UI-Automation. Gesehen und in Ordnung: Liste, Installieren mit Fortschritt, Update 1.0.0 auf 1.1.0, Updates-Seite mit Zahl, Einstellungsseite, Dunkelmodus (Apps, Updates, Einstellungen), Hinweis bei nicht passender Signatur (nichts verändert). Zweiter Durchgang: Kachelansicht, „…“-Menü, Entfernen mit Rückfrage (Abbrechen ändert nichts, Entfernen löscht den Programmordner, grüner Hinweis) in Ordnung. Nicht gesehen: Englisch, Tastatur.

| Nr | Art | Beobachtung | Bewertung | Status |
|----|-----|-------------|-----------|--------|
| E1 | Fehler | Beim Download (Installieren oder Aktualisieren) läuft die Zeile über den Fensterrand: Text „Läd…“ ist abgeschnitten, bei der Zeile darunter ist der Knopf „Installieren“ nach rechts verschoben und abgeschnitten. | klein, sichtbar | erledigt: kürzerer Text („Lädt … 12 %“), schmalerer Balken, Test misst die Zeilenbreite |
| E2 | Fehler | Nach abgelehntem Katalog (Signatur) steht auf der Updates-Seite weiter „Alles ist auf dem neuesten Stand.“ neben dem roten Hinweis. Irreführend, richtig wäre „Stand unbekannt“. | klein | erledigt: bei abgelehntem Katalog entfällt der Satz, Test |
| E3 | Wunsch | In der Update-Zeile steht nur „V1.0.0“ (installiert), die neue Version fehlt. Besser „1.0.0 → 1.1.0“. | klein | erledigt: „V1.0.0 auf V1.1.0“ (de) / „to“ (en), Test; der Pfeil fehlt in der Schrift |
| E5 | Wunsch | Kachelansicht: Kacheln einer Reihe sind unterschiedlich hoch, wenn eine Beschreibung umbricht (Notizbuch höher als Zeichenbrett). | klein, Schönheit | offen |
| E6 | Fehler | Kachelansicht mit Bildlaufleiste: die rechte Kachel wird am Rand abgeschnitten, die Leiste liegt über dem Rahmen. | klein | offen |
| E4 | Frage | Fenstertitelleiste bleibt im Dunkelmodus hell (gehört Windows). Bekannt, kein Fehler im Launcher. | kein Handlungsbedarf | geklärt |

## Rückmeldungen von Björn
(noch keine)
