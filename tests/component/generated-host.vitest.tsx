import { useHost } from "@wabou/core";
import { createTestHost, renderComponent } from "@wabou/test/component";
import { Button, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import type { DescribePaletteRequest } from "../../apps/gallery/ui/generated/rust-bindings";

test("drives a generated Rust host module through a component", async () => {
  let received: DescribePaletteRequest | undefined;
  const fixture = createTestHost({
    bindingsDemo: {
      describePalette: (request: DescribePaletteRequest) => {
        received = request;
        return {
          title: `${request.name} palette`,
          swatches: [`${request.name}-1`, `${request.name}-2`],
        };
      },
    },
  });
  const Palette = () => {
    const host = useHost();
    const [title, setTitle] = createSignal("Not loaded");
    return (
      <View>
        <Button
          aria-label="Load palette"
          onClick={() => {
            const response = host.bindingsDemo.describePalette({
              name: "Ocean",
              swatchCount: 2,
            });
            setTitle(response.title);
          }}
        >
          Load palette
        </Button>
        <Text role="status">{title()}</Text>
      </View>
    );
  };
  const screen = renderComponent(Palette, { host: fixture.host });

  screen.getByRole("button", { name: "Load palette" }).click();

  await screen.waitFor(() => {
    expect(screen.getByRole("status", { name: "Ocean palette" }).text).toBe(
      "Ocean palette",
    );
  });
  expect(received).toEqual({ name: "Ocean", swatchCount: 2 });
  expect(fixture.callsTo("bindingsDemo.describePalette")).toHaveLength(1);
});
