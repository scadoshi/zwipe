"""Drives headless Chrome over --remote-debugging-pipe: open a URL, run JS steps, screenshot."""
import base64, json, os, subprocess, sys, time

# CHROME overrides; otherwise the Linux headless shell, then the macOS Chrome app.
CHROME = os.environ.get("CHROME") or next(
    (p for p in (
        os.path.expanduser("~/.cache/ms-playwright/chromium_headless_shell-1248/chrome-headless-shell-linux64/chrome-headless-shell"),
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ) if os.path.exists(p)),
    "chromium",
)

class Pipe:
    def __init__(self, width, height):
        r_in, w_in = os.pipe()   # we write -> chrome fd 3
        r_out, w_out = os.pipe() # chrome fd 4 -> we read
        def pre():
            os.dup2(r_in, 3); os.dup2(w_out, 4)
        self.proc = subprocess.Popen([CHROME, "--remote-debugging-pipe", "--no-sandbox", "--headless=new", "--hide-scrollbars",
            f"--window-size={width},{height}", "--user-data-dir=" + os.environ.get("PROFILE", "/tmp/webc-profile"), "about:blank"],
            pass_fds=(3, 4), preexec_fn=pre, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        os.close(r_in); os.close(w_out)
        self.w = os.fdopen(w_in, "wb", buffering=0); self.r = os.fdopen(r_out, "rb", buffering=0)
        self.buf = b""; self.id = 0; self.session = None; self.events = []
    def send(self, method, params=None, session=True):
        self.id += 1
        msg = {"id": self.id, "method": method, "params": params or {}}
        if session and self.session: msg["sessionId"] = self.session
        self.w.write(json.dumps(msg).encode() + b"\0")
        while True:
            m = self.read()
            if m.get("id") == self.id:
                if "error" in m: raise RuntimeError(m["error"])
                return m.get("result", {})
            self.events.append(m)
    def read(self):
        while b"\0" not in self.buf:
            chunk = self.r.read(65536)
            if not chunk: raise EOFError
            self.buf += chunk
        raw, self.buf = self.buf.split(b"\0", 1)
        return json.loads(raw)

def main():
    url, width, height, steps_file, shot_prefix = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4], sys.argv[5]
    p = Pipe(width, height)
    t = p.send("Target.createTarget", {"url": "about:blank"}, session=False)
    a = p.send("Target.attachToTarget", {"targetId": t["targetId"], "flatten": True}, session=False)
    p.session = a["sessionId"]
    p.send("Runtime.enable"); p.send("Page.enable")
    p.send("Emulation.setDeviceMetricsOverride", {"width": width, "height": height, "deviceScaleFactor": 1, "mobile": False})
    p.send("Page.navigate", {"url": url})
    steps = json.load(open(steps_file))
    n = 0
    for step in steps:
        kind = step[0]
        if kind == "wait": time.sleep(step[1])
        elif kind == "js":
            r = p.send("Runtime.evaluate", {"expression": step[1], "awaitPromise": True, "returnByValue": True})
            print("js:", json.dumps(r.get("result", {}).get("value"))[:600])
        elif kind == "type":
            for ch in step[1]:
                p.send("Input.insertText", {"text": ch})
        elif kind == "key":
            for t_ in ("keyDown", "keyUp"):
                p.send("Input.dispatchKeyEvent", {"type": t_, "key": step[1], "code": step[1], "windowsVirtualKeyCode": 13 if step[1] == "Enter" else 0})
        elif kind == "shot":
            n += 1
            data = p.send("Page.captureScreenshot", {"format": "png"})["data"]
            path = f"{shot_prefix}{step[1]}.png"
            open(path, "wb").write(base64.b64decode(data)); print("shot:", path)
    for m in p.events:
        if m.get("method") == "Runtime.consoleAPICalled":
            args = " ".join(str(x.get("value", x.get("description", ""))) for x in m["params"]["args"])
            if any(w in args.lower() for w in ("error", "warn", "panic")): print("console:", args[:300])
        if m.get("method") == "Runtime.exceptionThrown":
            print("exception:", json.dumps(m["params"])[:400])
    p.proc.kill()

main()
