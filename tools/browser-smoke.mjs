// Exercise the production WASM in Chrome through its DevTools protocol.
// Node 22+ and Chrome/Chromium are the only dependencies; no npm install is needed.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { mkdtemp, readFile, rm, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve, extname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("../examples/showcase/dist/", import.meta.url)));
const output = resolve(process.argv[2] ?? fileURLToPath(new URL("../target/browser-smoke/", import.meta.url)));
await readFile(join(root, "index.html")); // Fail early if Trunk has not run.
await mkdir(output, { recursive: true });
const profile = await mkdtemp(join(tmpdir(), "faseela-browser-"));
const server = createServer(async (request, response) => {
  try {
    const name = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    const path = resolve(root, `.${name === "/" ? "/index.html" : name}`);
    if (!path.startsWith(root + sep) && path !== join(root, "index.html")) {
      response.writeHead(403).end();
      return;
    }
    const data = await readFile(path);
    response.setHeader("Content-Type", ({ ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm" })[extname(path)] ?? "application/octet-stream");
    response.end(data);
  } catch {
    response.writeHead(404).end();
  }
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const address = `http://127.0.0.1:${server.address().port}`;
const chrome = spawn(process.env.CHROME ?? "google-chrome", [
  "--headless=new", "--no-sandbox", "--disable-dev-shm-usage", "--no-first-run",
  "--no-default-browser-check", "--enable-unsafe-swiftshader", "--use-angle=swiftshader",
  "--remote-debugging-port=0", `--user-data-dir=${profile}`, "about:blank",
], { stdio: ["ignore", "ignore", "pipe"] });
let socket;
try {
  const endpoint = await new Promise((resolve, reject) => {
    let log = "";
    const timeout = setTimeout(() => reject(new Error("Chrome startup timed out")), 15000);
    chrome.once("error", (error) => { clearTimeout(timeout); reject(error); });
    chrome.stderr.on("data", (chunk) => {
      log += chunk;
      const match = log.match(/DevTools listening on (ws:\/\/\S+)/);
      if (match) { clearTimeout(timeout); resolve(match[1]); }
    });
    chrome.once("exit", (code) => { clearTimeout(timeout); reject(new Error(`Chrome exited (${code}): ${log}`)); });
  });
  socket = new WebSocket(endpoint);
  await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
  let nextId = 0;
  const pending = new Map();
  const errors = [];
  socket.onmessage = ({ data }) => {
    const message = JSON.parse(data);
    if (message.id) {
      const request = pending.get(message.id);
      pending.delete(message.id);
      clearTimeout(request.timeout);
      if (message.error) request.reject(new Error(JSON.stringify(message.error)));
      else request.resolve(message.result);
    }
    if (message.method === "Runtime.exceptionThrown") errors.push(message.params.exceptionDetails);
    if (message.method === "Runtime.consoleAPICalled" && message.params.type === "error") errors.push(message.params.args);
  };
  function send(method, params = {}, sessionId) {
    const id = ++nextId;
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => { pending.delete(id); reject(new Error(`${method} timed out`)); }, 30000);
      pending.set(id, { resolve, reject, timeout });
      socket.send(JSON.stringify({ id, method, params, sessionId }));
    });
  }
  const { targetId } = await send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
  const call = (method, params) => send(method, params, sessionId);
  await call("Page.enable");
  await call("Runtime.enable");
  const evaluate = async (expression) => {
    const result = await call("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
    assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails));
    return result.result.value;
  };
  // Await actual painted frames, not a fixed machine-dependent screenshot delay.
  const settle = () => evaluate("new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => requestAnimationFrame(resolve))))");
  async function click(x, y) {
    await call("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
    await call("Input.dispatchMouseEvent", { type: "mousePressed", x, y, button: "left", clickCount: 1 });
    await settle();
    await call("Input.dispatchMouseEvent", { type: "mouseReleased", x, y, button: "left", clickCount: 1 });
    await settle();
  }
  async function key(key, code, windowsVirtualKeyCode) {
    await call("Input.dispatchKeyEvent", { type: "keyDown", key, code, windowsVirtualKeyCode });
    await call("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode });
    await settle();
  }
  async function resize(width, height) {
    await call("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile: false });
    await settle();
  }
  const captures = [];
  async function screenshot(name, width, height) {
    const geometry = await evaluate(`(() => { const canvas = document.querySelector('canvas'); const rect = canvas.getBoundingClientRect(); return { width: innerWidth, height: innerHeight, canvasWidth: canvas.width, canvasHeight: canvas.height, left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom, scrollWidth: document.documentElement.scrollWidth }; })()`);
    assert.equal(geometry.width, width);
    assert.equal(geometry.height, height);
    assert.equal(geometry.canvasWidth, width);
    assert.equal(geometry.canvasHeight, height);
    assert.equal(geometry.left, 0);
    assert.equal(geometry.top, 0);
    assert.equal(geometry.right, width);
    assert.equal(geometry.bottom, height);
    assert(geometry.scrollWidth <= width);
    const { data } = await call("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
    await writeFile(join(output, name + ".png"), Buffer.from(data, "base64"));
    captures.push({ name, ...geometry });
  }
  await resize(1280, 800);
  await call("Page.addScriptToEvaluateOnNewDocument", { source: "window.faseelaStarted = new Promise(resolve => addEventListener('TrunkApplicationStarted', resolve, {once: true}));" });
  await call("Page.navigate", { url: address });
  await evaluate("window.faseelaStarted");
  await settle();
  await screenshot("desktop-overview", 1280, 800);

  // Desktop sidebar entries are fixed-height Small buttons (26 + 8 spacing + 2 gap).
  // Screenshots make these explicit coordinates reviewable if showcase navigation changes.
  await click(90, 188); // Buttons
  await screenshot("desktop-buttons", 1280, 800);
  await click(90, 224); // Forms
  await screenshot("desktop-forms", 1280, 800);
  await click(90, 368); // Appearance
  await screenshot("desktop-appearance", 1280, 800);
  await click(90, 440); // RTL & Arabic
  await screenshot("desktop-arabic", 1280, 800);

  const viewports = [[320,568],[360,640],[390,844],[680,800],[768,1024],[1280,800],[1920,1080]];
  for (const [width, height] of viewports) {
    await resize(width, height);
    await screenshot(`viewport-${width}x${height}`, width, height);
    console.log(`Captured ${width}x${height}`);
  }
  await resize(360, 640);
  await click(310, 30); // Open mobile section disclosure.
  await screenshot("mobile-navigation", 360, 640);
  await key("Escape", "Escape", 27);
  await screenshot("mobile-navigation-closed", 360, 640);

  if (process.env.FASEELA_BROWSER_MATRIX === "1") {
    for (const [preset, presetX] of [["dark", 322], ["midnight", 417], ["forest", 495], ["light", 559]]) {
      for (const rtl of [false, true]) {
        await resize(1280, 800);
        await call("Page.navigate", { url: address });
        await evaluate("window.faseelaStarted");
        await settle();
        await click(90, 368); // Appearance
        await click(presetX, 180);
        if (rtl) {
          await click(90, 440);
          await click(350, 178); // RTL button; subsequent section navigation moves to right sidebar.
        }
        for (let section = 0; section < 10; section++) {
          await resize(1280, 800);
          await click(rtl ? 1160 : 90, 116 + section * 36);
          for (const [width, height] of viewports) {
            await resize(width, height);
            await screenshot(`${preset}-${rtl ? "rtl" : "ltr"}-section-${section}-${width}x${height}`, width, height);
          }
        }
        console.log(`Captured all sections and viewports: ${preset} ${rtl ? "RTL" : "LTR"}`);
      }
    }
  }
  assert.equal(errors.length, 0, JSON.stringify(errors, null, 2));
  await writeFile(join(output, "results.json"), JSON.stringify({ captures, errors }, null, 2));
  console.log(`${captures.length} production-browser captures; no JavaScript/WASM errors. ${output}`);
} finally {
  socket?.close();
  const exited = new Promise((resolve) => chrome.once("exit", resolve));
  if (chrome.exitCode === null) { chrome.kill(); await exited; }
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
  await rm(profile, { recursive: true, force: true });
}
