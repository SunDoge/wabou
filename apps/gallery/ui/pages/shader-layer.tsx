import { Badge, Button, ShaderLayer, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";

const AURORA_SHADER = /* wgsl */ `
fn palette(t: f32) -> vec3<f32> {
  let navy = vec3<f32>(0.018, 0.035, 0.095);
  let violet = vec3<f32>(0.42, 0.20, 0.92);
  let cyan = vec3<f32>(0.10, 0.72, 0.86);
  return mix(mix(navy, violet, smoothstep(0.05, 0.72, t)), cyan, smoothstep(0.62, 1.1, t));
}

fn wabou_effect(uv: vec2<f32>) -> vec4<f32> {
  let time = wabou.resolution_time_scale.z;
  let aspect = wabou.resolution_time_scale.x / max(wabou.resolution_time_scale.y, 1.0);
  var p = (uv - vec2<f32>(0.5)) * vec2<f32>(aspect, 1.0);
  p.x = p.x + 0.06 * sin(time * 0.31);
  let wave_a = sin(p.x * 5.0 + time * 0.72) * 0.15;
  let wave_b = cos(p.x * 3.1 - time * 0.48) * 0.12;
  let ribbon_a = exp(-13.0 * abs(p.y - wave_a + 0.04));
  let ribbon_b = exp(-18.0 * abs(p.y + wave_b - 0.11));
  let glow = exp(-3.8 * length(p - vec2<f32>(0.24 * sin(time * 0.27), -0.08)));
  let grain = 0.018 * sin(uv.x * 87.0 + uv.y * 53.0 + time);
  let energy = clamp(ribbon_a * 0.72 + ribbon_b * 0.58 + glow * 0.38 + grain, 0.0, 1.25);
  let color = palette(energy) + vec3<f32>(0.03, 0.04, 0.08) * (1.0 - uv.y);
  return vec4<f32>(color, 1.0);
}`;

export function ShaderLayerPage() {
  const [paused, setPaused] = createSignal(false);
  const [speed, setSpeed] = createSignal(0.75);

  return (
    <View class="flex flex-col gap-5">
      <View class="relative h-80 overflow-hidden rounded-xl border border-subtle shadow-lg">
        <ShaderLayer
          aria-label="Animated aurora shader"
          source={AURORA_SHADER}
          speed={speed()}
          paused={paused()}
          class="absolute inset-0 w-full h-full rounded-xl"
        />
        <View class="absolute left-0 right-0 bottom-0 p-5 flex items-end justify-between gap-4">
          <View class="flex flex-col items-start gap-1">
            <Badge variant="secondary">Native WGSL</Badge>
            <Text class="text-lg font-semibold text-white">Aurora field</Text>
            <Text class="text-sm text-slate-200">
              One retained widget, animated without per-frame Solid updates.
            </Text>
          </View>
        </View>
      </View>

      <View class="p-4 flex flex-wrap items-center justify-between gap-4 rounded-lg border border-subtle bg-surface">
        <View class="flex flex-col gap-1">
          <Text class="text-sm font-medium text-primary">
            Native frame clock
          </Text>
          <Text class="text-xs text-secondary">
            WGSL is validated once; pipelines and textures are retained across
            frames and resize.
          </Text>
        </View>
        <View class="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => setSpeed((value) => (value === 0.75 ? 0.35 : 0.75))}
          >
            {speed() === 0.75 ? "Slow down" : "Normal speed"}
          </Button>
          <Button size="sm" onClick={() => setPaused((value) => !value)}>
            {paused() ? "Resume" : "Pause"}
          </Button>
        </View>
      </View>
    </View>
  );
}
