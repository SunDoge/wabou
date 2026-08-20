import { expect, test } from "bun:test";
import { dispatchHostMessage, subscribeJson } from "./host-messages";

test("JSON host-message subscriptions parse, decode, report errors and unsubscribe", () => {
  const values: number[] = [];
  const errors: unknown[] = [];
  const unsubscribe = subscribeJson(
    "counter",
    (value: number) => values.push(value),
    {
      decode(value) {
        if (typeof value !== "number") throw new TypeError("expected number");
        return value;
      },
      onError(error) {
        errors.push(error);
      },
    },
  );
  dispatchHostMessage("counter", "4");
  dispatchHostMessage("counter", new TextEncoder().encode("5"));
  dispatchHostMessage("counter", '"wrong"');
  dispatchHostMessage("counter", 5);
  unsubscribe();
  dispatchHostMessage("counter", "6");
  expect(values).toEqual([4, 5]);
  expect(errors).toHaveLength(2);
});
