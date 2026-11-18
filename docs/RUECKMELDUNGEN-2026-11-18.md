# Rückmeldungen 2026-11-18 (FB Launcher)

Art: Fehler / Wunsch / Frage. Größe: klein (hier erledigt) / Paket (an den Master).

## Eigentest mit Demo-Katalog (Claude, 2026-11-18)
Demo-Katalog mit erfundenen Apps und Testschlüssel, Launcher im echten Fenster, Bedienung per UI-Automation. Gesehen und in Ordnung: Liste, Installieren mit Fortschritt, Update 1.0.0 auf 1.1.0, Updates-Seite mit Zahl, Einstellungsseite, Dunkelmodus (Apps, Updates, Einstellungen), Hinweis bei nicht passender Signatur (nichts verändert). Zweiter Durchgang: Kachelansicht, „…“-Menü, Entfernen mit Rückfrage (Abbrechen ändert nichts, Entfernen löscht den Programmordner, grüner Hinweis) in Ordnung. Dritter Durchgang: Englisch (Haupt-, Updates- und Einstellungsseite komplett übersetzt, nur die App-Texte aus dem Katalog bleiben deutsch) und Tastatur (Tab springt durch Seitenleiste und Knöpfe, Enter löst aus) in Ordnung.

| Nr | Art | Beobachtung | Bewertung | Status |
|----|-----|-------------|-----------|--------|
| E1 | Fehler | Beim Download (Installieren oder Aktualisieren) läuft die Zeile über den Fensterrand: Text „Läd…“ ist abgeschnitten, bei der Zeile darunter ist der Knopf „Installieren“ nach rechts verschoben und abgeschnitten. | klein, sichtbar | erledigt: kürzerer Text („Lädt … 12 %“), schmalerer Balken, Test misst die Zeilenbreite |
| E2 | Fehler | Nach abgelehntem Katalog (Signatur) steht auf der Updates-Seite weiter „Alles ist auf dem neuesten Stand.“ neben dem roten Hinweis. Irreführend, richtig wäre „Stand unbekannt“. | klein | erledigt: bei abgelehntem Katalog entfällt der Satz, Test |
| E3 | Wunsch | In der Update-Zeile steht nur „V1.0.0“ (installiert), die neue Version fehlt. Besser „1.0.0 → 1.1.0“. | klein | erledigt: „V1.0.0 auf V1.1.0“ (de) / „to“ (en), Test; der Pfeil fehlt in der Schrift |
| E5 | Wunsch | Kachelansicht: Kacheln einer Reihe sind unterschiedlich hoch, wenn eine Beschreibung umbricht (Notizbuch höher als Zeichenbrett). | klein, Schönheit | erledigt: Höhe je Reihe aus dem letzten Bild, Knopfzeile unten bündig, Test, im Fenster gesehen |
| E6 | Fehler | Kachelansicht mit Bildlaufleiste: die rechte Kachel wird am Rand abgeschnitten, die Leiste liegt über dem Rahmen. | klein | erledigt: 10 px Rand rechts im Raster, im Fenster gesehen |
| E7 | Fehler | Der Download-Fortschritt ragte in Englisch („Loading … 21 %“) weiter über den Fensterrand als in Deutsch; mein Fix aus E1 reichte nicht. | klein, sichtbar | erledigt: Fortschritt in der Listenzeile in fester Breite (Balken oben, Text darunter, notfalls gekürzt), Test prüft Deutsch und Englisch, im Fenster gesehen |
| E8 | Fehler | Tastaturfokus kaum sichtbar: ein fokussierter Knopf (zum Beispiel „Install“) unterscheidet sich im hellen Modus kaum von den anderen, nur „Aktualisieren“ wird blassgrau. Ursache vermutlich `selection.stroke` in `theme.rs` (weiß). Barrierefreiheit. | klein bis mittel | offen: Rahmenfarbe ändern, ohne die Textmarkierung zu beeinträchtigen, mit Test |
| E9 | Wunsch | Beim Download wird die Listenzeile etwa 28 px höher (Balken über Text). Funktioniert, wirkt aber springend. | kosmetisch | offen, niedrig |
| E4 | Frage | Fenstertitelleiste bleibt im Dunkelmodus hell (gehört Windows). Bekannt, kein Fehler im Launcher. | kein Handlungsbedarf | geklärt |

## Rückmeldungen von Björn
(noch keine)
