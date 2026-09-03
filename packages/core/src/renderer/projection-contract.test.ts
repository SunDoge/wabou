import { expect, test } from "bun:test";
import { createRoot, createSignal, flush } from "solid-js";
import { OP } from "../protocol";
import { createElement, spread, writer } from "./index";

test("one Solid text write emits one fine-grained host mutation", () => {
  createRoot((dispose) => {
    writer.flush();
    const text = createElement("text");
    const [value, setValue] = createSignal("before", { ownedWrite: true });
    spread(
      text,
      {
        get children() {
          return value();
        },
      },
      false,
    );
    flush();
    writer.flush();

    flush(() => setValue("after"));
    const frame = writer.flush();
    if (!frame) throw new Error("Solid text write emitted no protocol frame");
    const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

    expect(view.getUint32(4, true)).toBe(1);
    expect(frame[8]).toBe(OP.SetText);
    dispose();
    writer.flush();
  });
});
