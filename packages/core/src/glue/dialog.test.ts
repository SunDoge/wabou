import { expect, test } from "bun:test";

let nextRequest = 1;
const calls: Array<[number, number, unknown]> = [];

Object.assign(globalThis, {
  __wabou_effect_abi: 3,
  __wabou_effect_submit: (capability: number, method: number, json: string) => {
    const requestId = nextRequest++;
    calls.push([capability, method, JSON.parse(json)]);
    return requestId;
  },
});

const { dialog } = await import("./dialog");

test("file dialogs normalize filters and preserve cancellation", async () => {
  const opened = dialog.open({
    multiple: true,
    filters: [{ name: "Images", extensions: [".png", "jpg"] }],
  });
  expect(calls.at(-1)).toEqual([
    5,
    1,
    {
      filters: [{ name: "Images", extensions: ["png", "jpg"] }],
      multiple: true,
    },
  ]);
  __wabou_effect_complete(1, 5, 1, 0, JSON.stringify(["/tmp/a.png"]));
  expect(await opened).toEqual(["/tmp/a.png"]);

  const saved = dialog.save({ defaultName: "report.json" });
  __wabou_effect_complete(2, 5, 2, 0, "null");
  expect(await saved).toBeNull();
});

test("message dialogs expose a finite native result", async () => {
  const result = dialog.message({
    message: "Discard changes?",
    level: "warning",
    buttons: "yesNoCancel",
  });
  expect(calls.at(-1)).toEqual([
    5,
    4,
    {
      message: "Discard changes?",
      level: "warning",
      buttons: "yesNoCancel",
    },
  ]);
  __wabou_effect_complete(3, 5, 4, 0, JSON.stringify("no"));
  expect(await result).toBe("no");
});
