// Exercise the production WASM and wait for the actual eframe app in Chrome.
// Node 24+ and Chrome are the only runtime dependencies; no npm install is needed.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { mkdtemp, readFile, rm, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve, extname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";

const STARTUP_TIMEOUT_MS = Number(process.env.FASEELA_BROWSER_STARTUP_TIMEOUT_MS ?? 45000);
const COMMAND_TIMEOUT_MS = Number(process.env.FASEELA_BROWSER_COMMAND_TIMEOUT_MS ?? 20000);
assert(Number.isSafeInteger(STARTUP_TIMEOUT_MS) && STARTUP_TIMEOUT_MS > 0,
  "FASEELA_BROWSER_STARTUP_TIMEOUT_MS must be a positive integer");
assert(Number.isSafeInteger(COMMAND_TIMEOUT_MS) && COMMAND_TIMEOUT_MS > 0,
  "FASEELA_BROWSER_COMMAND_TIMEOUT_MS must be a positive integer");

const root = resolve(fileURLToPath(new URL("../examples/showcase/dist/", import.meta.url)));
const output = resolve(process.argv[2] ?? fileURLToPath(new URL("../target/browser-smoke/", import.meta.url)));
await readFile(join(root, "index.html")); // Fail early if Trunk has not run.
await mkdir(output, { recursive: true });

const profile = await mkdtemp(join(tmpdir(), "faseela-browser-"));
const httpFailures = [];
const contentTypes = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".wasm": "application/wasm",
  ".css": "text/css; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".json": "application/json",
  ".woff": "font/woff",
  ".woff2": "font/woff2",
};

const server = createServer(async (request, response) => {
  try {
    if (request.method !== "GET" && request.method !== "HEAD") {
      response.writeHead(405).end();
      return;
    }

    const pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    const path = resolve(root, "." + (pathname === "/" ? "/index.html" : pathname));
    if (!path.startsWith(root + sep) && path !== join(root, "index.html")) {
      httpFailures.push({ url: request.url, status: 403, reason: "path escaped showcase dist" });
      response.writeHead(403).end();
      return;
    }

    const data = await readFile(path);
    response.setHeader("Content-Type", contentTypes[extname(path)] ?? "application/octet-stream");
    response.setHeader("X-Content-Type-Options", "nosniff");
    response.writeHead(200);
    response.end(request.method === "HEAD" ? undefined : data);
  } catch (error) {
    const status = error.code === "ENOENT" ? 404 : 500;
    httpFailures.push({ url: request.url, status, error: String(error) });
    response.writeHead(status).end();
  }
});

await new Promise((resolve, reject) => {
  server.once("error", reject);
  server.listen(0, "127.0.0.1", resolve);
});
const address = "http://127.0.0.1:" + server.address().port;

const chrome = spawn(process.env.CHROME ?? "google-chrome", [
  "--headless=new", "--no-sandbox", "--disable-dev-shm-usage", "--no-first-run",
  "--no-default-browser-check", "--enable-unsafe-swiftshader", "--use-angle=swiftshader",
  "--remote-debugging-port=0", "--user-data-dir=" + profile, "about:blank",
], { stdio: ["ignore", "ignore", "pipe"] });

let socket;
let chromeLog = "";
chrome.stderr.on("data", (chunk) => {
  chromeLog = (chromeLog + chunk.toString()).slice(-16000);
});

try {
  const endpoint = await new Promise((resolve, reject) => {
    const cleanup = () => {
      clearTimeout(timeout);
      chrome.stderr.removeListener("data", inspect);
      chrome.removeListener("error", onError);
      chrome.removeListener("exit", onExit);
    };
    const inspect = () => {
      const match = chromeLog.match(/DevTools listening on (ws:\/\/\S+)/);
      if (match) {
        cleanup();
        resolve(match[1]);
      }
    };
    const onError = (error) => {
      cleanup();
      reject(new Error("Could not start Chrome: " + error.message));
    };
    const onExit = (code, signal) => {
      cleanup();
      reject(new Error("Chrome exited before DevTools was ready (code=" + code
        + ", signal=" + signal + "). stderr: " + chromeLog));
    };
    const timeout = setTimeout(() => {
      cleanup();
      reject(new Error("Chrome DevTools endpoint was not ready after 15000ms. stderr: " + chromeLog));
    }, 15000);
    chrome.stderr.on("data", inspect);
    chrome.once("error", onError);
    chrome.once("exit", onExit);
    inspect();
  });

  socket = new WebSocket(endpoint);
  await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error("Timed out connecting to Chrome DevTools")), 10000);
    socket.addEventListener("open", () => {
      clearTimeout(timeout);
      resolve();
    }, { once: true });
    socket.addEventListener("error", () => {
      clearTimeout(timeout);
      reject(new Error("Could not connect to Chrome DevTools WebSocket"));
    }, { once: true });
  });

  let nextId = 0;
  const pending = new Map();
  const errors = [];
  const networkFailures = [];
  const requestUrls = new Map();

  socket.onmessage = ({ data }) => {
    let message;
    try {
      message = JSON.parse(data);
    } catch (error) {
      errors.push("Invalid Chrome DevTools message: " + String(error));
      return;
    }

    if (message.id !== undefined) {
      const request = pending.get(message.id);
      if (request) {
        pending.delete(message.id);
        clearTimeout(request.timeout);
        if (message.error) request.reject(new Error(JSON.stringify(message.error)));
        else request.resolve(message.result);
      }
    }

    if (message.method === "Runtime.exceptionThrown") {
      errors.push(message.params.exceptionDetails?.text ?? JSON.stringify(message.params.exceptionDetails));
    }
    if (message.method === "Runtime.consoleAPICalled" && message.params.type === "error") {
      const detail = (message.params.args ?? [])
        .map((argument) => argument.value ?? argument.description ?? "")
        .filter(Boolean)
        .join(" ");
      errors.push(detail || "Chrome console.error was called");
    }
    if (message.method === "Network.requestWillBeSent") {
      requestUrls.set(message.params.requestId, message.params.request.url);
    }
    if (message.method === "Network.loadingFailed") {
      networkFailures.push({
        url: requestUrls.get(message.params.requestId) ?? message.params.requestId,
        error: message.params.errorText,
        blockedReason: message.params.blockedReason ?? null,
      });
    }
  };

  socket.onclose = () => {
    for (const [id, request] of pending) {
      clearTimeout(request.timeout);
      request.reject(new Error("Chrome DevTools WebSocket closed while a command was pending"));
      pending.delete(id);
    }
  };

  function send(method, params = {}, sessionId, timeoutMs = COMMAND_TIMEOUT_MS) {
    const id = ++nextId;
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        pending.delete(id);
        reject(new Error("Chrome DevTools command " + method + " timed out after " + timeoutMs + "ms"));
      }, timeoutMs);
      pending.set(id, { resolve, reject, timeout });
      try {
        socket.send(JSON.stringify({ id, method, params, sessionId }));
      } catch (error) {
        clearTimeout(timeout);
        pending.delete(id);
        reject(error);
      }
    });
  }

  const { targetId } = await send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await send("Target.attachToTarget", { targetId, flatten: true });
  const call = (method, params, timeoutMs) => send(method, params, sessionId, timeoutMs);

  await call("Page.enable");
  await call("Runtime.enable");
  await call("Network.enable");
  await call("Runtime.addBinding", { name: "faseelaTrunkReady" });
  await call("Runtime.addBinding", { name: "faseelaReady" });

  const evaluate = async (expression, timeoutMs = COMMAND_TIMEOUT_MS) => {
    const result = await call("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    }, timeoutMs);
    assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails));
    return result.result.value;
  };

  function waitForBinding(bindingName, label, timeoutMs = STARTUP_TIMEOUT_MS) {
    return new Promise((resolve, reject) => {
      let timeout;
      const cleanup = () => {
        clearTimeout(timeout);
        socket.removeEventListener("message", onMessage);
        socket.removeEventListener("close", onClose);
      };
      const onMessage = ({ data }) => {
        let message;
        try {
          message = JSON.parse(data);
        } catch {
          return;
        }
        if (message.sessionId === sessionId
          && message.method === "Runtime.bindingCalled"
          && message.params.name === bindingName) {
          cleanup();
          resolve(message.params.payload);
        }
      };
      const onClose = () => {
        cleanup();
        reject(new Error("Chrome closed before " + label + " completed"));
      };

      socket.addEventListener("message", onMessage);
      socket.addEventListener("close", onClose, { once: true });
      timeout = setTimeout(async () => {
        cleanup();
        let pageState = "unavailable";
        try {
          pageState = await evaluate("(() => {"
            + "const canvas = document.querySelector('canvas');"
            + "return {url: location.href, readyState: document.readyState,"
            + "wasmBindings: typeof window.wasmBindings,"
            + "canvas: canvas ? {width: canvas.width, height: canvas.height,"
            + "rect: canvas.getBoundingClientRect().toJSON()} : null};"
            + "})()", 5000);
        } catch (error) {
          pageState = "diagnostic evaluation failed: " + String(error);
        }
        reject(new Error(label + " timed out after " + timeoutMs + "ms. Page state: "
          + JSON.stringify(pageState) + ". Browser errors: " + JSON.stringify(errors.slice(-8))
          + ". Network failures: " + JSON.stringify(networkFailures.slice(-8))
          + ". HTTP failures: " + JSON.stringify(httpFailures.slice(-8))));
      }, timeoutMs);
    });
  }

  // Trunk's event proves WASM instantiation only. FaseelaUiReady is emitted by
  // startup.rs after eframe has initialized the renderer and created the app.
  await call("Page.addScriptToEvaluateOnNewDocument", {
    source: [
      'addEventListener("TrunkApplicationStarted", () => {',
      '  window.faseelaTrunkReady(JSON.stringify({url: location.href, wasmBindings: typeof window.wasmBindings}));',
      '}, {once: true});',
      'addEventListener("FaseelaUiReady", () => {',
      '  window.faseelaReady(JSON.stringify({url: location.href, canvas: Boolean(document.querySelector("canvas"))}));',
      '}, {once: true});',
    ].join("\n"),
  });

  const navigateAndWaitForApp = async () => {
    // Install both listeners before navigation so neither readiness event is missed.
    const trunkStarted = waitForBinding("faseelaTrunkReady", "Trunk WebAssembly initialization");
    const appStarted = waitForBinding("faseelaReady", "Faseela eframe WebRunner startup");
    const navigation = await call("Page.navigate", { url: address });
    if (navigation.errorText) {
      throw new Error("Chrome navigation failed: " + navigation.errorText);
    }
    const [trunkInfo, appInfo] = await Promise.all([trunkStarted, appStarted]);
    console.log("Trunk WASM initialized; eframe WebRunner started: " + appInfo);
    await settle();
    return { trunkInfo, appInfo };
  };

  // Wait for real animation frames rather than a machine-dependent fixed delay.
  const settle = () => evaluate(
    "new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => requestAnimationFrame(resolve))))"
  );

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
    await call("Emulation.setDeviceMetricsOverride", {
      width, height, deviceScaleFactor: 1, mobile: false,
    });
    await settle();
  }

  const captures = [];
  const visualHashes = new Set();
  const hash = (data) => {
    let value = 2166136261;
    for (const byte of Buffer.from(data, "base64")) value = Math.imul(value ^ byte, 16777619);
    return (value >>> 0).toString(16).padStart(8, "0");
  };

  async function screenshot(name, width, height) {
    await call("Input.dispatchMouseEvent", { type: "mouseMoved", x: width - 2, y: height - 2 });
    await settle();
    const geometry = await evaluate("(() => {"
      + "const canvas = document.querySelector('canvas');"
      + "if (!canvas) return {hasCanvas: false};"
      + "const rect = canvas.getBoundingClientRect();"
      + "return {hasCanvas: true, width: innerWidth, height: innerHeight,"
      + "canvasWidth: canvas.width, canvasHeight: canvas.height,"
      + "left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom,"
      + "scrollWidth: document.documentElement.scrollWidth};"
      + "})()");
    assert.equal(geometry.hasCanvas, true, "Showcase canvas must exist after eframe startup");
    assert.equal(geometry.width, width);
    assert.equal(geometry.height, height);
    assert.equal(geometry.canvasWidth, width);
    assert.equal(geometry.canvasHeight, height);
    assert.equal(geometry.left, 0);
    assert.equal(geometry.top, 0);
    assert.equal(geometry.right, width);
    assert.equal(geometry.bottom, height);
    assert(geometry.scrollWidth <= width);

    const { data } = await call("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: false,
    });
    await writeFile(join(output, name + ".png"), Buffer.from(data, "base64"));
    const imageHash = hash(data);
    visualHashes.add(imageHash);
    captures.push({ name, ...geometry, imageHash });
    return imageHash;
  }

  await resize(1280, 800);
  console.log("Opening production showcase; waiting for WASM and eframe startup...");
  await navigateAndWaitForApp();
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

  const viewports = [[320, 568], [360, 640], [390, 844], [680, 800], [768, 1024], [1280, 800], [1920, 1080]];
  for (const [width, height] of viewports) {
    await resize(width, height);
    await screenshot("viewport-" + width + "x" + height, width, height);
    console.log("Captured " + width + "x" + height);
  }

  await resize(360, 640);
  await click(310, 30); // Open mobile section disclosure.
  await screenshot("mobile-navigation", 360, 640);
  await key("Escape", "Escape", 27);
  await screenshot("mobile-navigation-closed", 360, 640);

  if (process.env.FASEELA_BROWSER_MATRIX === "1") {
    const stableSectionHashes = [];
    const themeDirectionHashes = [];
    for (const [preset, presetX] of [["dark", 322], ["midnight", 417], ["forest", 495], ["light", 559]]) {
      for (const rtl of [false, true]) {
        await resize(1280, 800);
        await navigateAndWaitForApp();
        await click(90, 368); // Appearance
        await click(presetX, 180);
        await click(90, 440); // RTL & Arabic in the current LTR desktop sidebar.
        if (rtl) await click(350, 178); // RTL; direction takes effect on the next frame.
        let overviewHash;
        for (let section = 0; section < 10; section++) {
          await resize(1280, 800);
          await click(rtl ? 1160 : 90, 116 + section * 36);
          for (const [width, height] of viewports) {
            await resize(width, height);
            const imageHash = await screenshot(
              preset + "-" + (rtl ? "rtl" : "ltr") + "-section-" + section + "-" + width + "x" + height,
              width,
              height
            );
            if (width === 1280 && height === 800) {
              if (section === 0) overviewHash = imageHash;
              stableSectionHashes.push(imageHash);
            }
          }
        }
        themeDirectionHashes.push(overviewHash);
        console.log("Captured all sections and viewports: " + preset + " " + (rtl ? "RTL" : "LTR"));
      }
    }

    for (let index = 0; index < stableSectionHashes.length; index += 10) {
      assert.equal(new Set(stableSectionHashes.slice(index, index + 10)).size, 10,
        "Each of ten sections should paint a distinct desktop frame (#" + index / 10 + ")");
    }
    assert.equal(new Set(themeDirectionHashes).size, 8,
      "All four themes and both directions should change the rendered overview");
  }

  assert.equal(httpFailures.length, 0,
    "Showcase server returned failed asset requests: " + JSON.stringify(httpFailures, null, 2));
  assert.equal(errors.length, 0, JSON.stringify(errors, null, 2));
  assert(visualHashes.size >= 8,
    "Expected real canvas updates; observed " + visualHashes.size + " distinct images");

  await writeFile(join(output, "results.json"), JSON.stringify({
    captures,
    distinctImages: visualHashes.size,
    errors,
    networkFailures,
    serverFailures: httpFailures,
  }, null, 2));
  console.log(captures.length + " production-browser captures (" + visualHashes.size
    + " distinct); no JavaScript/WASM errors. " + output);
} finally {
  socket?.close();
  if (chrome.exitCode === null) {
    const exited = new Promise((resolve) => chrome.once("exit", resolve));
    chrome.kill();
    await exited;
  }
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
  await rm(profile, { recursive: true, force: true });
}
