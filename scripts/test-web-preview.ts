/** Browser proof using Chrome's isolated DevTools protocol; no npm browser package. */
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const url = process.argv[2] ?? "http://127.0.0.1:5179/";
const chromeOverride = process.env.CHROME;
const chrome =
  chromeOverride ?? Bun.which("google-chrome") ?? Bun.which("chromium");
if (!chrome)
  throw new Error("Chrome/Chromium is required; set CHROME to its executable.");
const profile = await mkdtemp(join(tmpdir(), "wabou-web-proof-"));
const output = resolve(import.meta.dirname, "../target/web-preview/proof");
await mkdir(output, { recursive: true });
const browser = Bun.spawn(
  [
    chrome,
    "--headless=new",
    "--no-sandbox",
    "--disable-dev-shm-usage",
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    "about:blank",
  ],
  {
    stdout: "ignore",
    stderr: "ignore",
    env: Object.fromEntries(
      Object.entries(process.env).filter(
        ([key]) => key !== "DISPLAY" && key !== "WAYLAND_DISPLAY",
      ),
    ),
  },
);
let socket: WebSocket | undefined;
try {
  let port = "";
  const start = Date.now();
  while (!port) {
    if (browser.exitCode !== null)
      throw new Error("Chrome exited before exposing DevTools");
    if (Date.now() - start > 10000)
      throw new Error("Chrome did not expose DevTools within 10s");
    try {
      port = (
        await readFile(join(profile, "DevToolsActivePort"), "utf8")
      ).split("\n")[0];
    } catch {
      await Bun.sleep(50);
    }
  }
  const tab = (await fetch(`http://127.0.0.1:${port}/json/new?about:blank`, {
    method: "PUT",
  }).then((r) => r.json())) as { webSocketDebuggerUrl: string };
  const connection = new WebSocket(tab.webSocketDebuggerUrl);
  socket = connection;
  await new Promise<void>((resolve, reject) => {
    connection.onopen = () => resolve();
    connection.onerror = reject;
  });
  let sequence = 0;
  const pending = new Map<
    number,
    { resolve: (v: unknown) => void; reject: (e: unknown) => void }
  >();
  const errors: unknown[] = [];
  let pageLoaded: (() => void) | undefined;
  connection.onmessage = (event) => {
    const message = JSON.parse(String(event.data));
    if (message.method === "Page.loadEventFired") pageLoaded?.();
    if (message.method === "Runtime.exceptionThrown")
      errors.push(message.params.exceptionDetails);
    const task = pending.get(message.id);
    if (task) {
      pending.delete(message.id);
      if (message.error) task.reject(message.error);
      else task.resolve(message.result);
    }
  };
  const send = <T = unknown>(method: string, params: object = {}): Promise<T> =>
    new Promise((resolve, reject) => {
      const id = ++sequence;
      const timeout = setTimeout(() => {
        pending.delete(id);
        reject(new Error(`DevTools request timed out: ${method}`));
      }, 15000);
      pending.set(id, {
        resolve: (value) => {
          clearTimeout(timeout);
          resolve(value as T);
        },
        reject: (reason) => {
          clearTimeout(timeout);
          reject(reason);
        },
      });
      connection.send(JSON.stringify({ id, method, params }));
    });
  async function evaluate<T = unknown>(expression: string) {
    const reply = await send<{
      result: { value: T };
      exceptionDetails?: unknown;
    }>("Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    if (reply.exceptionDetails)
      throw new Error(JSON.stringify(reply.exceptionDetails));
    return reply.result.value;
  }
  async function until(condition: string) {
    await evaluate(
      `new Promise((resolve,reject)=>{ const deadline=Date.now()+10000; const timer=setInterval(()=>{ if (${condition}) {clearInterval(timer);resolve(true);} else if(Date.now()>deadline){clearInterval(timer);reject(new Error('Browser assertion timed out: '+${JSON.stringify(condition)}));}},16); })`,
    );
  }
  const assert = (condition: unknown, message: string) => {
    if (!condition) throw new Error(message);
  };
  await send("Runtime.enable");
  await send("Page.enable");
  await send("Page.addScriptToEvaluateOnNewDocument", {
    source: `
      globalThis.__canvasSizeWrites = 0;
      for (const property of ['width', 'height']) {
        const descriptor = Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype, property);
        Object.defineProperty(HTMLCanvasElement.prototype, property, {
          ...descriptor,
          set(value) {
            globalThis.__canvasSizeWrites++;
            descriptor.set.call(this, value);
          },
        });
      }
      globalThis.__browserApis = Object.fromEntries(['requestAnimationFrame','cancelAnimationFrame','ResizeObserver','setTimeout','clearTimeout','setInterval','clearInterval','fetch','TextEncoder','TextDecoder'].map(key=>[key,globalThis[key]]));
    `,
  });
  await send("Emulation.setDeviceMetricsOverride", {
    width: 1100,
    height: 700,
    deviceScaleFactor: 1,
    mobile: false,
  });
  const loaded = new Promise<void>((resolve, reject) => {
    const timeout = setTimeout(
      () => reject(new Error("Preview page did not load within 15s")),
      15000,
    );
    pageLoaded = () => {
      clearTimeout(timeout);
      resolve();
    };
  });
  await send("Page.navigate", { url });
  await loaded;
  pageLoaded = undefined;
  await until(
    `document.querySelector('#status')?.textContent === 'Ready' || document.querySelector('#status')?.textContent === 'Preview failed'`,
  );
  assert(
    await evaluate(`document.querySelector('#status').textContent === 'Ready'`),
    await evaluate<string>(`document.querySelector('#error').textContent`),
  );
  assert(
    await evaluate(`(()=>{
      const output = new Float64Array(3);
      const id = __wabou_motion_create(JSON.stringify({kind:'tween',duration:1,easing:'linear'}));
      const sampled = __wabou_motion_sample(id, 0.5, output);
      const dropped = __wabou_motion_drop(id);
      return sampled && dropped && Math.abs(output[0] - 0.5) < 1e-9 && output[2] === 0;
    })()`),
    "Browser Wasm motion ABI did not sample a Rust-owned tween",
  );
  await until(
    `globalThis.__wabou_preview.snapshot().some(n=>n.text === 'Count: 0')`,
  );
  assert(
    await evaluate(
      `Object.entries(globalThis.__browserApis).every(([key,value])=>globalThis[key]===value)`,
    ),
    "Wabou replaced the surrounding browser APIs",
  );
  const capture = async (name: string) => {
    const { data } = await send<{ data: string }>("Page.captureScreenshot", {
      format: "png",
    });
    await writeFile(join(output, name), Buffer.from(data, "base64"));
    return data as string;
  };
  const initial = await capture("initial-1x.png");
  const initialCanvasSizeWrites = await evaluate<number>(
    `globalThis.__canvasSizeWrites`,
  );
  async function click(text: string) {
    const point = await evaluate<{ x: number; y: number }>(
      `(()=>{ const n=__wabou_preview.snapshot().find(n=>n.text===${JSON.stringify(text)}); const c=__wabou_preview.canvas.getBoundingClientRect(); return {x:c.left+1+(n.rect[0]+n.rect[2])/2,y:c.top+1+(n.rect[1]+n.rect[3])/2}; })()`,
    );
    await send("Input.dispatchMouseEvent", {
      type: "mousePressed",
      button: "left",
      clickCount: 1,
      ...point,
    });
    await send("Input.dispatchMouseEvent", {
      type: "mouseReleased",
      button: "left",
      clickCount: 1,
      ...point,
    });
  }
  await click("Increment");
  await until(`__wabou_preview.snapshot().some(n=>n.text==='Count: 1')`);
  assert(
    (await evaluate<number>(`globalThis.__canvasSizeWrites`)) ===
      initialCanvasSizeWrites,
    "A same-size scene update reset the canvas drawing buffer",
  );
  const incremented = await capture("incremented-1x.png");
  assert(
    initial !== incremented,
    "Counter changed in Rust layout but not in captured GPU pixels",
  );
  await click("Reset");
  await until(`__wabou_preview.snapshot().some(n=>n.text==='Count: 0')`);
  await send("Emulation.setDeviceMetricsOverride", {
    width: 390,
    height: 700,
    deviceScaleFactor: 2,
    mobile: false,
  });
  await until(
    `__wabou_preview.canvas.width===__wabou_preview.canvas.clientWidth*2 && __wabou_preview.snapshot()[0].rect[2]<390`,
  );
  await click("Increment");
  await until(`__wabou_preview.snapshot().some(n=>n.text==='Count: 1')`);
  await capture("narrow-2x.png");
  const tree = await evaluate<{ rect: number[]; text: string | null }[]>(
    `__wabou_preview.snapshot()`,
  );
  const width = await evaluate<number>(`__wabou_preview.canvas.clientWidth`);
  assert(
    tree.every(
      (n) =>
        n.rect.every(Number.isFinite) && n.rect[0] >= 0 && n.rect[2] <= width,
    ),
    "Narrow layout overflowed canvas",
  );
  // Verify a pure DPR change independently of a CSS resize.
  await send("Emulation.setDeviceMetricsOverride", {
    width: 390,
    height: 700,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await until(
    `__wabou_preview.canvas.width===__wabou_preview.canvas.clientWidth`,
  );
  if (process.argv.includes("--hmr")) {
    const app = resolve(import.meta.dirname, "../examples/web-preview/app.tsx");
    const original = await readFile(app, "utf8");
    const modified = original.replace("Rendered by Wabou", "Updated by Vite");
    assert(modified !== original, "HMR fixture header is missing");
    await evaluate(`globalThis.__hmrSnapshot = __wabou_preview.snapshot`);
    async function restoreSource() {
      if ((await readFile(app, "utf8")) !== modified)
        throw new Error(
          "HMR source changed concurrently; refusing to overwrite it",
        );
      await writeFile(app, original);
    }
    try {
      await writeFile(app, modified);
      await until(
        `globalThis.__wabou_preview?.snapshot().some(n=>n.text==='Updated by Vite')`,
      );
      assert(
        await evaluate(`__wabou_preview.snapshot===globalThis.__hmrSnapshot`),
        "HMR recreated the browser/Wasm host",
      );
      await capture("hmr-1x.png");
    } finally {
      await restoreSource();
    }
  }
  assert(errors.length === 0, `Browser exceptions: ${JSON.stringify(errors)}`);
  await writeFile(
    join(output, "report.json"),
    JSON.stringify(
      {
        url,
        renderer: "Vello WebGL2",
        js: "browser Solid",
        checks: [
          "native application mount entry",
          "browser APIs preserved",
          "initial Rust layout",
          "GPU pixels change on click",
          "reset",
          "narrow responsive layout",
          "2x canvas size and hit testing",
          "DPR changes without a CSS resize",
          ...(process.argv.includes("--hmr")
            ? ["TSX updates without recreating Wasm host"]
            : []),
          "no browser exceptions",
        ],
        tree,
      },
      null,
      2,
    ),
  );
  console.log(`Browser preview checks passed. Artifacts: ${output}`);
} finally {
  socket?.close();
  browser.kill();
  await browser.exited;
  await rm(profile, { recursive: true, force: true });
}
