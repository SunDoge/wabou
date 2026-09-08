import { renderComponent } from "@wabou/test/component";
import { ShaderLayer, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

const source =
  "fn wabou_effect(uv: vec2<f32>) -> vec4<f32> { return vec4<f32>(uv, 0.0, 1.0); }";

const moduleSource = `
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
`;

test("keeps custom shader configuration reactive on a stable native node", () => {
  const Example = () => {
    const [paused, setPaused] = createSignal(false);
    return (
      <View>
        <ShaderLayer
          aria-label="Aurora preview"
          source={source}
          values={[0.8, 0.3]}
          speed={0.75}
          paused={paused()}
          onClick={() => setPaused(true)}
        />
      </View>
    );
  };
  const screen = renderComponent(Example);
  const layer = screen.getByRole("img", { name: "Aurora preview" });
  const identity = layer.identity;
  expect(layer.tag).toBe("shader-layer");
  expect(layer.widgetConfig).toMatchObject({
    source,
    values: [0.8, 0.3],
    speed: 0.75,
    paused: false,
  });

  layer.click();
  const updated = screen.getByRole("img", { name: "Aurora preview" });
  expect(updated.identity).toEqual(identity);
  expect(updated.widgetConfig).toMatchObject({ paused: true });
});

test("forwards the complete WGSL module contract", () => {
  const screen = renderComponent(() => (
    <ShaderLayer module source={moduleSource} values={[0, 0, 0, 0.5]} />
  ));
  expect(screen.getByRole("img").widgetConfig).toMatchObject({
    source: moduleSource,
    module: true,
    values: [0, 0, 0, 0.5],
  });
});
