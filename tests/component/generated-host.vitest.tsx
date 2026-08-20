import { createTestHost, renderComponent } from "@wabou/test/component";
import { Button, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  createBindingsDemoTestCapability,
  type DescribePaletteRequest,
  useBindingsDemoClient,
} from "../../apps/gallery/ui/generated/host-bindings";

test("drives a generated DTO capability through a component", async () => {
  let received: DescribePaletteRequest | undefined;
  const bindingsDemo = createBindingsDemoTestCapability({
    describePalette: async (request) => {
      received = request;
      return {
        status: "palette",
        title: `${request.name} palette`,
        swatches: [`${request.name}-1`, `${request.name}-2`],
      };
    },
  });
  const fixture = createTestHost({ bindingsDemo });
  const Palette = () => {
    const client = useBindingsDemoClient();
    const [title, setTitle] = createSignal("Not loaded");
    return (
      <View>
        <Button
          aria-label="Load palette"
          onClick={async () => {
            const response = await client.describePalette({
              name: "Ocean",
              swatchCount: 2,
            });
            setTitle(
              response.status === "palette" ? response.title : response.message,
            );
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
