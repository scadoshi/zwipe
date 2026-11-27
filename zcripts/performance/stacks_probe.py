"""stacks_probe.py ORIGIN ACTION VARIANT: the deck page's Stacks at 390 with a mocked 100-card deck and the CPU throttled 4x; runs ACTION (regroup, switch, raise, flip) and prints frame times and render counts. VARIANT: base, nosettle, replace (settle moves without composite add), nomargin (tap reveal without the margin transition)."""
import base64, copy, json, os, re, select, sys, time, uuid
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
os.environ.setdefault("PROFILE", __import__("tempfile").mkdtemp(prefix="stacks-probe-"))
from cdplib import Pipe

origin, action, variant = sys.argv[1], sys.argv[2], sys.argv[3]
screen, W = "deck", 390
theme = "gruvbox"; shot = screen
tag = f"{screen}-{W}"
H = 900 if W >= 700 else 844
far = "2030-01-01T00:00:00Z"
CORS = [{"name": "Access-Control-Allow-Origin", "value": origin}, {"name": "Access-Control-Allow-Credentials", "value": "true"},
        {"name": "Access-Control-Allow-Headers", "value": "content-type, authorization"}, {"name": "Access-Control-Allow-Methods", "value": "GET, POST, PUT, PATCH, DELETE, OPTIONS"}]
JSONH = CORS + [{"name": "Content-Type", "value": "application/json"}]
b64 = lambda o: base64.b64encode(json.dumps(o).encode()).decode()
HINTS = {k: True for k in "first_login add_deck_cards swipe_select remove_deck_cards create_deck edit_deck deck_list first_deck deck_cards profile config filter import export oracle_tag_dictionary otag_examples commander_maybeboard suggested_commanders activity universes_beyond_exceptions".split()}
def session(tok):
    return {"user": {"id": str(uuid.UUID(int=7)), "username": "layouttest", "email": "layouttest@example.com", "email_verified_at": far, "hints_shown": HINTS},
            "access_token": {"value": tok, "expires_at": far}, "refresh_token": {"value": "r", "expires_at": far}, "settings": {"theme": theme, "dark_mode": True}}
prof = json.load(open(os.path.join(HERE, "fixtures", "decks.json")))[2]
ENTRIES = json.load(open(os.path.join(HERE, "fixtures", "shared.json")))["entries"]
for e in ENTRIES:
    p = e["card"]["scryfall_data"].setdefault("prices", {})
    p.update(usd="0.25", eur=None, tix=None, usd_foil=None, usd_etched=None)
CMD = copy.deepcopy(ENTRIES[0]["card"]); ENTRIES = ENTRIES[1:]
CMD["scryfall_data"]["prices"].update(usd="123.79")
def printing(n, usd, set_name):
    c = copy.deepcopy(CMD); sd = c["scryfall_data"]
    sd["id"] = str(uuid.UUID(int=0xed6a0 + n)); sd["set_name"] = set_name; sd["collector_number"] = str(n)
    if isinstance(c.get("card_profile"), dict):
        c["card_profile"]["scryfall_data_id"] = sd["id"]
    sd["prices"].update(usd=usd, usd_foil=None if usd else "112.50")
    def bump(o):
        if isinstance(o, dict):
            return {k: (f"{v}{'&' if '?' in v else '?'}v={n}" if isinstance(v, str) and v.startswith("http") else bump(v)) for k, v in o.items()}
        if isinstance(o, list):
            return [bump(x) for x in o]
        return o
    for k in ("image_uris", "card_faces"):
        if sd.get(k):
            sd[k] = bump(sd[k])
    return c
PRINTINGS = [CMD, printing(1, "80.00", "Innistrad Remastered"), printing(3, None, "Judge Gift Cards 2021")]
BY_ID = {c["scryfall_data"]["id"]: c for c in PRINTINGS}
prof["commander_id"] = CMD["scryfall_data"]["id"]
DID = prof["id"]
TYPES = ["Creature \u2014 Elf Druid", "Instant", "Sorcery", "Artifact", "Enchantment", "Land", "Creature \u2014 Goblin"]
COLORS = [["G"], ["R"], ["U"], ["B"], ["W"], [], ["G", "U"]]
def deal100(base):
    out = []
    for i in range(100):
        e = copy.deepcopy(base[i % len(base)])
        sd = e["card"]["scryfall_data"]
        sd["id"] = str(uuid.UUID(int=0xc0de0000 + i)); sd["name"] = f"{sd['name']} {i}"
        sd["type_line"] = TYPES[i % len(TYPES)]; sd["colors"] = COLORS[(i // 3) % len(COLORS)]; sd["color_identity"] = sd["colors"]
        if isinstance(sd.get("image_uris"), dict):
            sd["image_uris"] = {k: (v + ("&" if "?" in v else "?") + f"n={i}") if isinstance(v, str) else v for k, v in sd["image_uris"].items()}
        if isinstance(e["card"].get("card_profile"), dict):
            e["card"]["card_profile"]["scryfall_data_id"] = sd["id"]
        e["deck_card"]["scryfall_data_id"] = sd["id"]
        out.append(e)
    return out
ENTRIES = deal100(ENTRIES)
_dfc = json.load(open(os.path.join(HERE, "fixtures", "cards54.json")))["dfc"]
ENTRIES[50] = {"card": _dfc, "deck_card": dict(ENTRIES[50]["deck_card"], scryfall_data_id=_dfc["scryfall_data"]["id"])}
DECKS = json.load(open(os.path.join(HERE, "fixtures", "decks.json")))
C54 = json.load(open(os.path.join(HERE, "fixtures", "cards54.json")))
COUNTS = {"W": 412, "U": 268, "B": 151, "R": 377, "G": 520}
SUMMARY = {"deck_count": len(DECKS), "color_map": {"colors": [{"color": c, "cards": n} for c, n in COUNTS.items()], "colorless_cards": 96, "pairs": [{"colors": [a, b], "decks": 1} for a, b in [("W","U"),("W","B"),("W","R"),("W","G"),("U","B"),("U","R"),("U","G"),("B","R"),("B","G"),("R","G")]]}}
WRITES = []
HITS = {}

p = Pipe(W, H)
t = p.send("Target.createTarget", {"url": "about:blank"}, session=False)
a = p.send("Target.attachToTarget", {"targetId": t["targetId"], "flatten": True}, session=False)
p.session = a["sessionId"]; p.send("Runtime.enable"); p.send("Page.enable")
p.send("Emulation.setDeviceMetricsOverride", {"width": W, "height": H, "deviceScaleFactor": 1, "mobile": W < 700})
p.send("Fetch.enable", {"patterns": [{"urlPattern": "*api.zwipe.net/api/*"}]})
ERRS = []
def handle(m):
    if m.get("method") == "Runtime.exceptionThrown":
        ERRS.append(json.dumps(m["params"]["exceptionDetails"].get("exception", {}).get("description", ""))[:300]); return
    if m.get("method") == "Runtime.consoleAPICalled" and m["params"]["type"] in ("error", "warning"):
        ERRS.append(" ".join(str(a.get("value", a.get("description", "")))[:300] for a in m["params"]["args"])); return
    if m.get("method") != "Fetch.requestPaused":
        return
    req = m["params"]["request"]; rid = m["params"]["requestId"]
    path = re.sub(r"^https?://[^/]+", "", req["url"]).split("?")[0]
    HITS[req["method"] + " " + re.sub(r"[0-9a-f]{8}-[0-9a-f-]{27}", "<id>", path)] = HITS.get(req["method"] + " " + re.sub(r"[0-9a-f]{8}-[0-9a-f-]{27}", "<id>", path), 0) + 1
    ok = lambda body: p.send("Fetch.fulfillRequest", {"requestId": rid, "responseCode": 200, "responseHeaders": JSONH, "body": body})
    if req["method"] == "OPTIONS":
        p.send("Fetch.fulfillRequest", {"requestId": rid, "responseCode": 204, "responseHeaders": CORS}); return
    if req["method"] == "POST" and path == f"/api/deck/{DID}/card/search":
        if os.environ.get("HOLD"):
            return
        ok(b64({"cards": [C54["creature"], C54["dfc"]], "synergy_applied": False})); return
    if path.endswith("/auth/cookie/refresh"):
        ok(b64(session("aaa.bbb.ccc"))); return
    if req["method"] == "PATCH" and path == f"/api/deck/{DID}":
        # Answered here, never sent on: the profile with the commander the request names.
        body = json.loads(req.get("postData") or "{}")
        WRITES.append("PATCH (mocked) " + json.dumps(body))
        new = body.get("commander_id")
        if isinstance(new, dict):
            new = next((v for v in new.values() if isinstance(v, str)), None)
        if isinstance(new, str):
            prof["commander_id"] = new
        ok(b64(prof)); return
    if req["method"] != "GET":
        WRITES.append("FAILED " + req["method"] + " " + path)
        p.send("Fetch.failRequest", {"requestId": rid, "errorReason": "Failed"}); return
    if req["method"] != "GET":
        WRITES.append("FAILED " + req["method"] + " " + path)
        p.send("Fetch.failRequest", {"requestId": rid, "errorReason": "Failed"}); return
    if path == f"/api/deck/{DID}" and os.environ.get("HOLD"):
        return
    if path == "/api/deck":
        ok(b64(DECKS)); return
    if path == "/api/deck/playstyle":
        ok(b64(SUMMARY)); return
    if path == f"/api/deck/{DID}":
        cz = [BY_ID.get(prof["commander_id"], CMD)]
        ok(b64({"deck_profile": prof, "entries": ENTRIES, "warnings": [], "command_zone_cards": cz}))
    elif path == f"/api/deck/profile/{DID}":
        ok(b64(prof))
    elif path.endswith("/printings"):
        ok(b64(PRINTINGS))
    elif path == f"/api/deck/{DID}/tokens":
        ok(b64([]))
    elif re.match(r"^/api/card/[0-9a-f-]+$", path):
        ok(b64(BY_ID.get(path.rsplit("/", 1)[1], CMD)))
    elif path.startswith("/api/card/featured") or path.startswith("/api/changelog") or path.startswith("/api/client") or path.startswith("/api/card/ub-franchises") or path.startswith("/api/metrics/public"):
        p.send("Fetch.continueRequest", {"requestId": rid})
    else:
        ok(b64([]))
def pump(secs):
    end = time.time() + secs
    while True:
        while p.events:
            handle(p.events.pop(0))
        left = end - time.time()
        if left <= 0:
            return
        if (p.buf and b"\0" in p.buf) or select.select([p.r], [], [], left)[0]:
            handle(p.read())
def js(e):
    r = p.send("Runtime.evaluate", {"expression": e, "returnByValue": True}).get("result", {}).get("value")
    pump(0); return r
def tap(selector):
    js("(()=>{const b=[...document.querySelectorAll('" + selector + "')].find(b=>b.offsetParent);if(b)b.scrollIntoView({block:'center'})})()"); pump(0.8)
    r = js("(()=>{const b=[...document.querySelectorAll('" + selector + "')].find(b=>b.offsetParent);if(!b)return null;const r=b.getBoundingClientRect();return [r.left+r.width/2,r.top+r.height/2]})()")
    if not r:
        return False
    for t in ("mouseMoved", "mousePressed", "mouseReleased"):
        p.send("Input.dispatchMouseEvent", {"type": t, "x": r[0], "y": r[1], "button": "left", "clickCount": 1})
    pump(1.2)
    return True
def key(k, code, vk):
    p.send("Input.dispatchKeyEvent", {"type": "keyDown", "key": k, "code": code, "windowsVirtualKeyCode": vk, "text": k if len(k) == 1 else ""})
    p.send("Input.dispatchKeyEvent", {"type": "keyUp", "key": k, "code": code, "windowsVirtualKeyCode": vk})
    pump(0.8)
def gotit():
    js("[...document.querySelectorAll('button')].filter(b=>b.offsetParent && b.textContent.trim()==='Got it').forEach(b=>b.click())"); pump(1)
RENDERS = []
_orig_handle = handle
def handle(m):
    if m.get("method") == "Runtime.consoleAPICalled":
        text = " ".join(str(a.get("value", "")) for a in m["params"]["args"])
        hit = re.search(r"R:([A-Za-z]+)", text)
        if hit:
            RENDERS.append(hit.group(1)); return
    _orig_handle(m)
p.send("Emulation.setCPUThrottlingRate", {"rate": 1 if action in ("frames", "rframes") else 4})
if variant in ("nosettle", "replace", "timed", "noghost", "hoist", "hoistshallow"):
    p.send("Page.addScriptToEvaluateOnNewDocument", {"source": "window.__zwSettle = true;"})
p.send("Page.navigate", {"url": origin + "/privacy"}); pump(8)
js("localStorage.setItem('zwiper-web.session', " + json.dumps(json.dumps(session("stored.without.token"))) + ")")
js("localStorage.setItem('zwiper-web.deck-cards-view', '" + ("list" if action == "switch" else "stacks") + "')")
js("localStorage.setItem('zwiper-web.theme', " + json.dumps(json.dumps({"name": theme, "is_dark": True, "server": {"name": theme, "is_dark": True}})) + ")")
p.send("Page.navigate", {"url": f"{origin}/decks/{DID}"}); pump(25)
gotit()
js("[...document.querySelectorAll('button')].find(b=>b.getClientRects().length && b.textContent.trim()==='Cards')?.click()"); pump(8)
gotit(); pump(4)
if variant in ("replace", "timed", "noghost", "hoist", "hoistshallow"):
    src = open(os.path.join(HERE, "..", "..", "components", "src", "settle.rs")).read()
    sj = src[src.index('const SETTLE_JS: &str = r"') + len('const SETTLE_JS: &str = r"'):src.index('";\n\n/// The script with its numbers')]
    sj = sj.replace("__MS__", "250").replace("__WATCH__", "900").replace("__JUMP__", "40").replace("__KEYS__", '["u", "U", "m", "s", "S", "-", "+"]').replace("__SKIP__", ".deck-fold-all, .web-deck-tags-fold")
    sj = sj.replace("if (window.__zwSettle) return;\n  window.__zwSettle = true;", "")
    if variant == "replace":
        sj = sj.replace(", { ...timing, composite: 'add' }", ", timing")
    if variant == "noghost":
        sj = sj.replace("const ghost = (old) => {", "const ghost = (old) => { return;")
    if variant in ("hoist", "hoistshallow"):
        sj = sj.replace("const ease = () => getComputedStyle(document.documentElement).getPropertyValue('--ease-enter').trim() || 'ease-out';", "let EASE = null; const ease = () => EASE || (EASE = getComputedStyle(document.documentElement).getPropertyValue('--ease-enter').trim() || 'ease-out');")
        assert "let EASE = null" in sj
    if variant == "hoistshallow":
        sj = sj.replace("const copy = old.el.cloneNode(true);", "const copy = old.el.cloneNode(false);")
    if variant == "timed":
        sj = sj.replace("    if (!prev) return;\n    const now = snapshot(touched);", "    if (!prev) return;\n    const t0 = performance.now();\n    const now = snapshot(touched);\n    const t1 = performance.now();")
        sj = sj.replace("    // Only what was measured can have left", "    const t2 = performance.now();\n    // Only what was measured can have left")
        sj = sj.replace("    for (const [key, cur] of now) prev.set(key, cur);\n  };", "    const t3 = performance.now();\n    (window.__st = window.__st || []).push({ snap: Math.round(t1 - t0), moves: Math.round(t2 - t1), ghosts: Math.round(t3 - t2), measured: now.size, gone: gone.length, ghostNodes: gone.reduce((n, k) => n + (prev.has(k) ? 0 : 0), 0) });\n    for (const [key, cur] of now) prev.set(key, cur);\n  };")
        sj = sj.replace("    document.body.appendChild(copy);", "    window.__gn = (window.__gn || 0) + copy.querySelectorAll('*').length + 1; window.__gi = (window.__gi || 0) + copy.querySelectorAll('img').length;\n    document.body.appendChild(copy);")
        assert sj.count("performance.now()") >= 4, "timers in"
    js(sj)
if variant == "nomargin":
    js("(()=>{const s=document.createElement('style');s.textContent='.card-stack.tap .card-stack-slot{transition:transform var(--card-stack-move) var(--card-stack-ease)!important}';document.head.appendChild(s)})()")
print(action, variant, "page:", js("({cards:document.querySelectorAll('.card-stack-card').length, stacks:document.querySelectorAll('.card-stack').length, tap:document.querySelectorAll('.card-stack.tap').length, list:document.querySelectorAll('.card-row').length})"))
REC = "(()=>{window.__f=[];let last=performance.now();const end=last+__MS__;const tick=t=>{window.__f.push(t-last);last=t;if(t<end)requestAnimationFrame(tick)};requestAnimationFrame(tick);window.__lt=[];try{new PerformanceObserver(l=>l.getEntries().forEach(e=>window.__lt.push(e.duration))).observe({type:'longtask'})}catch(e){}})()"
def record(ms):
    js(REC.replace("__MS__", str(ms)))
METRICS = {}
def metrics():
    got = p.send("Performance.getMetrics").get("metrics", [])
    return {m["name"]: m["value"] for m in got if m["name"] in ("LayoutCount", "RecalcStyleCount", "LayoutDuration", "RecalcStyleDuration", "ScriptDuration", "TaskDuration")}
def report(label):
    if METRICS:
        now = metrics()
        d = {k: (round((now[k] - METRICS[k]) * 1000) if "Duration" in k else int(now[k] - METRICS[k])) for k in now}
        print(action, variant, label, "work:", d)
    f = js("window.__f") or []
    lt = js("window.__lt") or []
    anims = js("document.getAnimations().length")
    f = [x for x in f if x > 0]
    worst = sorted(f)[-3:] if f else []
    over = len([x for x in f if x > 50])
    from collections import Counter
    print(action, variant, label, f"frames={len(f)} worst={[round(x) for x in worst]} over50ms={over} sum_over16={round(sum(max(0, x - 16.7) for x in f))}ms longtasks={[round(x) for x in lt][:6]} animations_now={anims} renders={dict(Counter(RENDERS))}")
pump(2); RENDERS.clear()
if action == "regroup":
    record(2000)
    js("[...document.querySelectorAll('button')].find(b=>b.getClientRects().length && b.textContent.trim()==='Color')?.click()"); pump(3)
    report("group by Color")
    if variant == "timed":
        print(action, variant, "settle checks:", js("window.__st"), "ghost nodes:", js("window.__gn"), "ghost imgs:", js("window.__gi"))
elif action == "frames":
    outdir = os.environ.get("FRAMES", ".")
    js("window.scrollTo(0, 0); document.querySelector('.deck-stacks')?.scrollIntoView({block:'start'})"); pump(1)
    js("[...document.querySelectorAll('button')].find(b=>b.getClientRects().length && b.textContent.trim()==='Color')?.click()")
    t0 = time.time()
    for i in range(10):
        d = p.send("Page.captureScreenshot", {"format": "png"})
        open(os.path.join(outdir, f"regroup-{variant}-{i:02d}.png"), "wb").write(base64.b64decode(d["data"]))
    print(action, variant, "10 frames over", round((time.time() - t0) * 1000), "ms")
elif action == "rframes":
    outdir = os.environ.get("FRAMES", ".")
    OPTION = {"floatoff": ".card-stack.tap .floatable.floating{animation-play-state:paused!important}", "marginoff": ".card-stack.tap .card-stack-slot{transition:transform var(--card-stack-move) var(--card-stack-ease)!important}"}.get(variant)
    if OPTION:
        js("(()=>{const s=document.createElement('style');s.textContent=" + json.dumps(OPTION) + ";document.head.appendChild(s)})()")
    big = "[...document.querySelectorAll('.card-stack')].sort((a,b)=>b.children.length-a.children.length)[0]"
    js("(()=>{const s=" + big + ";s.querySelectorAll('.card-stack-card')[2].scrollIntoView({block:'start'})})()"); pump(1.5)
    n = int(os.environ.get("N", "12"))
    js("(()=>{const s=" + big + ";s.querySelectorAll('.card-stack-card')[2].click()})()")
    t0 = time.time()
    for i in range(n):
        d = p.send("Page.captureScreenshot", {"format": "png"})
        open(os.path.join(outdir, f"raise-{variant}-{i:02d}.png"), "wb").write(base64.b64decode(d["data"]))
        if os.environ.get("GAP"):
            pump(float(os.environ["GAP"]))
    print(action, variant, n, "frames over", round((time.time() - t0) * 1000), "ms")
elif action == "switch":
    record(2500)
    js("[...document.querySelectorAll('button')].find(b=>b.getClientRects().length && b.textContent.trim()==='Stacks')?.click()"); pump(3.5)
    report("List to Stacks")
elif action == "raise":
    js("(()=>{const s=[...document.querySelectorAll('.card-stack')].sort((a,b)=>b.children.length-a.children.length)[0];const c=s&&s.querySelectorAll('.card-stack-card');if(c&&c[2])c[2].scrollIntoView({block:'center'})})()"); pump(1.5); RENDERS.clear()
    p.send("Performance.enable"); METRICS.update(metrics())
    record(1200)
    js("(()=>{const s=[...document.querySelectorAll('.card-stack')].sort((a,b)=>b.children.length-a.children.length)[0];s.querySelectorAll('.card-stack-card')[2].click()})()"); pump(2)
    report("raise one card")
    print(action, variant, "state:", js("({revealed:document.querySelectorAll('.card-stack-slot.revealed').length, floating:document.querySelectorAll('.card-stack-card.floating').length, untucked:[...[...document.querySelectorAll('.card-stack')].sort((a,b)=>b.children.length-a.children.length)[0].querySelectorAll('.card-stack-slot')].map(e=>e.classList.contains('tucked')?1:0).join(''), drawer:document.querySelectorAll('.bottom-sheet.show').length})"))
    RENDERS.clear(); METRICS.update(metrics()); record(1200)
    js("(()=>{const s=[...document.querySelectorAll('.card-stack')].sort((a,b)=>b.children.length-a.children.length)[0];s.querySelectorAll('.card-stack-card')[4].click()})()"); pump(2)
    report("raise another (first drops)")
    METRICS.update(metrics()); pump(3)
    report("3s with a card up")
elif action == "flip":
    print(action, variant, "dfc found:", js("(()=>{const c=[...document.querySelectorAll('.card-stack-card')].find(b=>/Delver|Insectile/.test(b.getAttribute('aria-label')||''));if(c){c.scrollIntoView({block:'center'});c.click();}return !!c})()")); pump(2)
    js("(()=>{const c=[...document.querySelectorAll('.card-stack-card')].find(b=>/Delver|Insectile/.test(b.getAttribute('aria-label')||''));if(c)c.click()})()"); pump(4)
    RENDERS.clear(); record(1500)
    FACE = "(()=>{const d=document.querySelector('.bottom-sheet.show');return d?[d.querySelector('.sheet-title')?.textContent, [...d.querySelectorAll('[data-face]')].map(e=>e.getAttribute('data-face')).slice(0,2)]:null})()"
    print(action, variant, "drawer before:", js(FACE))
    js("(()=>{const b=document.querySelector('.bottom-sheet.show [data-action=flip], .bottom-sheet.show [aria-label=Flip]');if(b)b.click();return !!b})()"); pump(2.5)
    print(action, variant, "drawer after:", js(FACE))
    report("flip in the drawer")
print(action, variant, "errors:", ERRS[:3], "writes:", WRITES[:3])
p.proc.kill()
