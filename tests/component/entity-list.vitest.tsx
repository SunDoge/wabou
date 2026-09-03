import { ForEntity, validateEntityKeys } from "@wabou/core";
import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { describe, expect, test } from "vitest";

describe("ForEntity", () => {
  test("preserves a retained child when sibling entities are removed", () => {
    const alpha = { id: "alpha", label: "Alpha" };
    const beta = { id: "beta", label: "Beta" };
    const [items, setItems] = createSignal([alpha, beta]);
    const screen = renderComponent(() => (
      <ForEntity each={items()} by={(item) => item.id}>
        {(item) => <view role="label" aria-label={item.label} />}
      </ForEntity>
    ));
    const identity = screen.getByRole("label", { name: "Beta" }).identity;

    setItems([beta]);
    screen.flush();

    expect(screen.getByRole("label", { name: "Beta" }).identity).toEqual(
      identity,
    );
  });

  test("rejects duplicate application keys", () => {
    expect(() =>
      validateEntityKeys(
        [
          { id: "same", label: "First" },
          { id: "same", label: "Second" },
        ],
        (item) => item.id,
      ),
    ).toThrow("ForEntity received duplicate key same");
  });
});
