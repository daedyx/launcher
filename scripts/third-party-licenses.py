"""Aufruf: python scripts/third-party-licenses.py (ohne Netz). Erneuert die Tabelle „Rust-Bibliotheken“ in THIRD-PARTY-LICENSES.md aus `cargo metadata --offline` (alle Pakete
im Lock außer den eigenen Workspace-Crates). Aufruf im Repo-Ordner."""
import json
import re
import subprocess

meta = json.loads(subprocess.run(
    ["cargo", "metadata", "--offline", "--locked", "--format-version", "1"],
    capture_output=True, check=True, text=True, encoding="utf-8").stdout)
own = set(meta["workspace_members"])
rows = []
for p in meta["packages"]:
    if p["id"] in own:
        continue
    lic = p.get("license") or "siehe Paket"
    src = p.get("repository") or p.get("homepage") or f"https://crates.io/crates/{p['name']}"
    if (p.get("source") or "").startswith("git+https://github.com/daedyx/"):
        src = p["source"][4:].split("?")[0]
    rows.append((p["name"], p["version"], lic, src))
rows.sort(key=lambda r: (r[0], [int(x) if x.isdigit() else x for x in re.split(r"[.+-]", r[1])]))
table = "| Paket | Version | Lizenz | Quelle |\n|---|---|---|---|\n" + "".join(f"| {a} | {b} | {c} | {d} |\n" for a, b, c, d in rows)
path = "THIRD-PARTY-LICENSES.md"
s = open(path, encoding="utf-8").read()
start = s.index("## Rust-Bibliotheken (")
end = s.index("## Symbole und Schriften")
s = s[:start] + f"## Rust-Bibliotheken ({len(rows)})\n\n" + table + "\n" + s[end:]
open(path, "w", encoding="utf-8").write(s)
print(len(rows), "Pakete")
