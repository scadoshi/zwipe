"""Render every theme in themes.css as a card: bg, text, accents, status, palette.

usage: gen.py <themes.css> <palette.json|-> <out.html> [dark|light|all]
palette.json: {"gruvbox-dark": ["#..", ...6], ...}; "-" means no palette data yet.
"""
import json, re, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from colors import contrast, dist
css, pal_path, out, mode = sys.argv[1], sys.argv[2], sys.argv[3], (sys.argv[4] if len(sys.argv) > 4 else "all")
src = open(css).read()
_j = json.load(open(pal_path)) if pal_path != "-" else {"palette":{},"notes":{}}
pal, notes = _j["palette"], _j.get("notes", {})
blocks = re.findall(r'\.theme-([a-z0-9-]+?)\s*\{([^}]*)\}', src)
themes = []
for name, body in blocks:
    v = dict(re.findall(r'--([a-z-]+):\s*([^;]+);', body))
    if mode != "all" and not name.endswith(mode):
        continue
    themes.append((name, v))
themes.sort(key=lambda t: (t[0].rsplit('-', 1)[0], t[0]))

def chip(label, color, fg):
    return f'<span class="chip" style="color:{color};border-color:color-mix(in srgb,{color} 35%,transparent);background:color-mix(in srgb,{color} 12%,transparent)">{label}</span>'

cards = []
for name, v in themes:
    bg = v.get("bg-primary", "#000"); tx = v.get("text-primary", "#fff"); mu = v.get("text-muted", tx); bd = v.get("border-secondary", mu)
    acc = [v.get(f"accent-{k}", "#f0f") for k in ("primary", "secondary", "tertiary")]
    st = [v.get(f"color-{k}", "#f0f") for k in ("success", "warning", "error")]
    p = pal.get(name) or [v.get(f"palette-{i}") for i in range(1, 7)]
    p = [c if c else None for c in p]
    flags = []
    def cls(i, c):
        k = ""
        if contrast(c, bg) < 3: k += " lowc"
        if any(dist(c, s_) < 10 for s_ in st): k += " nearstatus"
        return k
    sw = "".join(f'<div class="sw{cls(i,c)}" style="background:{c}"><b>{i+1}</b></div>' if c else f'<div class="sw empty" style="border-color:{mu}"><b>{i+1}</b></div>' for i, c in enumerate(p))
    pairs = [(i, j, dist(p[i], p[j])) for i in range(6) for j in range(i+1, 6) if p[i] and p[j]]
    worst = min(pairs, key=lambda t: t[2]) if pairs else None
    if worst and worst[2] < 15: flags.append(f"pair {worst[0]+1}/{worst[1]+1} dE {worst[2]:.0f}")
    if any(contrast(c, bg) < 3 for c in p if c): flags.append("contrast<3")
    if any(any(dist(c, s_) < 10 for s_ in st) for c in p if c): flags.append("near status")
    note = notes.get(name, "")
    flagtxt = (" · ".join(flags)) if flags else "ok"
    print(f"{name:28s} {note:18s} {flagtxt}")
    tags = "".join(chip(f"tag {i+1}", c, tx) for i, c in enumerate(p) if c)
    cards.append(f'''<section class="card" style="background:{bg};color:{tx};border-color:{bd}">
  <header><span class="eyebrow" style="color:{acc[2]}">{name.rsplit('-',1)[1]}</span><h3 style="color:{acc[0]}">{name.rsplit('-',1)[0]}</h3></header>
  <p style="color:{tx}">Body text in text-primary. <span style="color:{mu}">Muted text.</span> <a style="color:{acc[0]}">A link.</a> <span class="sel" style="color:{acc[1]};border-color:{acc[1]}">selected</span></p>
  <div class="row"><span class="lbl" style="color:{mu}">accents</span>{''.join(f'<div class="sw" style="background:{c}"></div>' for c in acc)}</div>
  <div class="row"><span class="lbl" style="color:{mu}">status</span>{''.join(f'<div class="sw" style="background:{c}"></div>' for c in st)}</div>
  <div class="row"><span class="lbl" style="color:{mu}">palette</span>{sw}</div>
  <div class="row tags">{tags}</div>
  <div class="foot" style="color:{mu}">{note} · {flagtxt}</div>
</section>''')
html = f'''<!doctype html><meta charset="utf-8"><title>Theme swatches</title>
<style>
body{{margin:0;padding:16px;background:#111;font-family:"JetBrains Mono",ui-monospace,monospace;font-size:12px}}
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:12px}}
.card{{border:1px solid;border-radius:10px;padding:12px 14px}}
header{{display:flex;flex-direction:column;gap:2px;margin-bottom:6px}}
.eyebrow{{font-size:10px;text-transform:uppercase;letter-spacing:.08em}}
h3{{margin:0;font-size:15px}}
p{{margin:6px 0 8px;line-height:1.5}}
a{{text-decoration:underline}}
.sel{{border:1px solid;border-radius:6px;padding:0 5px}}
.row{{display:flex;align-items:center;gap:5px;margin:4px 0}}
.lbl{{width:52px;font-size:10px}}
.sw{{width:26px;height:22px;border-radius:5px;position:relative}}
.sw b{{position:absolute;right:3px;bottom:1px;font-size:9px;color:rgba(0,0,0,.55);font-weight:400}}
.sw.empty{{border:1px dashed;background:transparent}}
.sw.lowc{{outline:2px solid #ff2d2d;outline-offset:1px}}
.sw.nearstatus::after{{content:"s";position:absolute;left:3px;top:1px;font-size:9px;color:rgba(0,0,0,.7)}}
.foot{{font-size:9px;margin-top:6px}}
.tags{{flex-wrap:wrap;margin-top:6px}}
.chip{{display:inline-block;border:1px solid;border-radius:999px;padding:1px 8px;font-size:10px}}
</style>
<div class="grid">{''.join(cards)}</div>'''
open(out, "w").write(html)
print(len(themes), "themes rendered")
