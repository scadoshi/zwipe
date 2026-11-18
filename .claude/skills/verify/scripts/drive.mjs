// Drives a page in headless Chrome over the DevTools protocol, one step at a time, and prints
// what each labeled step reads. Same steps, same widths, so a before and an after compare.
//
//   node drive.mjs <url> <width> <steps.json>      (H=<height>, default 900)
//
// Under 900 wide it emulates a phone (touch, no hover, coarse pointer); at 900 and over, a desktop.
//
// Steps run in order, one object each:
//   {"eval": "js", "label": "name"}   runs js in the page (awaits a promise); prints it if labeled
//   {"click": "css selector"}         a real mouse click at the element's center
//   {"type": "text"}                  types into the focused field
//   {"key": "u"} / {"key": "Enter"}   a key press; add "shift": true for Shift (sends "U")
//   {"nav": "url"}                    loads a url, then waits 6s (or "after": ms)
//   {"wait": ms}                      pauses
//   {"shot": "path.png"}              saves a screenshot
// Any step takes "after": ms to pause once it's done.
//
// Env: CHROME (the browser binary), PORT (the DevTools port, default 9340), H (height).
import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const [, , url, width, stepsFile] = process.argv;
if (!url || !width || !stepsFile) {
  console.error("usage: node drive.mjs <url> <width> <steps.json>");
  process.exit(2);
}
const steps = JSON.parse(readFileSync(stepsFile, "utf8"));
const chromePath =
  process.env.CHROME ??
  (process.platform === "darwin"
    ? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    : "google-chrome");
const port = process.env.PORT ?? "9340";
const profile = mkdtempSync(join(tmpdir(), "zwipe-verify-"));
const chrome = spawn(
  chromePath,
  ["--headless=new", "--no-first-run", `--user-data-dir=${profile}`, `--remote-debugging-port=${port}`, "about:blank"],
  { stdio: "ignore" },
);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let targets;
for (let i = 0; i < 50 && !targets; i++) {
  try {
    targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
  } catch {
    await sleep(200);
  }
}
if (!targets) {
  console.error(`no Chrome on port ${port}; set CHROME to the browser binary`);
  chrome.kill();
  process.exit(1);
}
const ws = new WebSocket(targets.find((t) => t.type === "page").webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener("open", r));
let id = 0;
const pending = new Map();
ws.addEventListener("message", (e) => {
  const m = JSON.parse(e.data);
  if (pending.has(m.id)) {
    pending.get(m.id)(m);
    pending.delete(m.id);
  }
});
const send = (method, params = {}) =>
  new Promise((r) => {
    const i = ++id;
    pending.set(i, r);
    ws.send(JSON.stringify({ id: i, method, params }));
  });
const evaluate = async (expression) => {
  const r = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  return r.result?.result?.value ?? r.result?.exceptionDetails?.exception?.description ?? r.error;
};

// Under 900 wide it's a phone: touch, no hover and a coarse pointer, so the web app's
// mouse-only rules (`hover: hover`) stop applying, as on a real phone's browser.
const phone = Number(width) < 900;
await send("Emulation.setDeviceMetricsOverride", {
  width: Number(width),
  height: Number(process.env.H ?? 900),
  deviceScaleFactor: phone ? 3 : 1,
  mobile: phone,
});
await send("Emulation.setTouchEmulationEnabled", { enabled: phone, maxTouchPoints: phone ? 5 : 0 });
await send("Emulation.setEmulatedMedia", {
  features: phone
    ? [{ name: "hover", value: "none" }, { name: "pointer", value: "coarse" }]
    : [{ name: "hover", value: "hover" }, { name: "pointer", value: "fine" }],
});
await send("Page.navigate", { url });
await sleep(7000);

const named = { Enter: 13, Escape: 27, Tab: 9, Backspace: 8, ArrowUp: 38, ArrowDown: 40, ArrowLeft: 37, ArrowRight: 39 };
for (const s of steps) {
  if (s.eval !== undefined) {
    const value = await evaluate(s.eval);
    if (s.label) console.log(`${s.label} =>`, JSON.stringify(value));
  } else if (s.click) {
    const at = await evaluate(
      `(()=>{const e=document.querySelector(${JSON.stringify(s.click)}); if(!e) return null; e.scrollIntoView({block:'center'}); const b=e.getBoundingClientRect(); return [b.left+b.width/2, b.top+b.height/2]})()`,
    );
    if (!Array.isArray(at)) console.log("click target missing:", s.click);
    else
      for (const type of ["mouseMoved", "mousePressed", "mouseReleased"])
        await send("Input.dispatchMouseEvent", { type, x: at[0], y: at[1], button: "left", clickCount: 1 });
  } else if (s.type !== undefined) {
    await send("Input.insertText", { text: s.type });
  } else if (s.key) {
    // With Shift a letter is its capital, as the app reads it from the event's key.
    const k = s.shift && s.key.length === 1 ? s.key.toUpperCase() : s.key;
    const vk = named[k] ?? k.toUpperCase().charCodeAt(0);
    const base = { key: k, code: k.length > 1 ? k : `Key${k.toUpperCase()}`, windowsVirtualKeyCode: vk, modifiers: s.shift ? 8 : 0 };
    const text = k === "Enter" ? "\r" : k.length === 1 ? k : undefined;
    await send("Input.dispatchKeyEvent", { type: text ? "keyDown" : "rawKeyDown", ...base, ...(text ? { text } : {}) });
    await send("Input.dispatchKeyEvent", { type: "keyUp", ...base });
  } else if (s.nav) {
    await send("Page.navigate", { url: s.nav });
    await sleep(s.after ?? 6000);
    continue;
  } else if (s.wait) {
    await sleep(s.wait);
  } else if (s.shot) {
    const shot = await send("Page.captureScreenshot", { format: "png" });
    writeFileSync(s.shot, Buffer.from(shot.result.data, "base64"));
  }
  await sleep(s.after ?? (s.key || s.click ? 400 : 0));
}
ws.close();
const exited = new Promise((r) => chrome.once("exit", r));
chrome.kill();
await exited;
rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
