import { Text, View } from "@wabou/ui";

export type CrossRenderPolicy = {
  compareGeometry?: boolean;
  comparePixels?: boolean;
  layoutTolerance?: number;
  regionTolerance?: number;
};

export const CROSS_RENDER_POLICIES: Record<string, CrossRenderPolicy> = {
  "compare/clipped-child": { comparePixels: false },
  "compare/radius-full": { regionTolerance: 0.03 },
  "compare/radius-xl": { regionTolerance: 0.012 },
  "compare/rotated": { compareGeometry: false, regionTolerance: 0.045 },
  "compare/translated": { compareGeometry: false },
  "compare/text-body": { comparePixels: false, layoutTolerance: 1 },
  "compare/text-bold": { comparePixels: false, layoutTolerance: 1 },
  "compare/text-wrap": { comparePixels: false, layoutTolerance: 1 },
};

const ProbeLabel = (props: { children: string }) => (
  <Text class="text-xs text-muted whitespace-nowrap">{props.children}</Text>
);

/**
 * Renderer-neutral geometry and paint matrix. Wabou uses native primitives;
 * Chromium aliases `@wabou/ui` to a DOM adapter. Keep it inside the common
 * primitive subset so every case is authored once.
 */
export function CrossRenderFixture() {
  return (
    <View
      aria-label="compare/root"
      class="relative w-full h-full overflow-hidden bg-[#f7f8fa] text-[#172033] font-sans"
      style={{ "font-family": "DejaVu Sans" }}
    >
      <View
        class="absolute p-4 border-2 border-[#94a3b8] rounded-xl bg-white"
        style={{ left: "24px", top: "24px", width: "288px", height: "176px" }}
      >
        <ProbeLabel>BOX MODEL + FLEX</ProbeLabel>
        <View
          aria-label="compare/flex-row"
          class="mt-3 w-full h-20 flex flex-row items-center gap-3 p-3 border border-[#cbd5e1] bg-[#eef2ff]"
        >
          <View
            aria-label="compare/flex-fixed"
            class="w-12 h-10 flex-none rounded-md bg-[#2563eb]"
          />
          <View
            aria-label="compare/flex-grow"
            class="min-w-0 h-8 flex-1 rounded-sm bg-[#14b8a6]"
          />
          <View
            aria-label="compare/flex-tail"
            class="w-8 h-12 flex-none bg-[#f59e0b]"
          />
        </View>
      </View>

      <View
        class="absolute p-4 border border-[#cbd5e1] rounded-xl bg-white"
        style={{ left: "328px", top: "24px", width: "288px", height: "176px" }}
      >
        <ProbeLabel>GRID + GAP</ProbeLabel>
        <View
          aria-label="compare/grid"
          class="mt-3 w-full h-24 grid grid-cols-3 gap-2 p-2 bg-[#f1f5f9]"
        >
          <View aria-label="compare/grid-a" class="bg-[#7c3aed]" />
          <View aria-label="compare/grid-b" class="bg-[#0ea5e9]" />
          <View aria-label="compare/grid-c" class="bg-[#10b981]" />
          <View aria-label="compare/grid-d" class="bg-[#f97316]" />
          <View aria-label="compare/grid-e" class="bg-[#e11d48]" />
          <View aria-label="compare/grid-f" class="bg-[#64748b]" />
        </View>
      </View>

      <View
        class="absolute p-4 border border-[#cbd5e1] rounded-xl bg-white"
        style={{ left: "632px", top: "24px", width: "304px", height: "176px" }}
      >
        <ProbeLabel>ALIGNMENT + FRACTIONS</ProbeLabel>
        <View
          aria-label="compare/justify"
          class="mt-3 w-full h-24 flex flex-col justify-between p-2 bg-[#f8fafc]"
        >
          <View
            aria-label="compare/justify-top"
            class="w-1/3 h-5 bg-[#2563eb]"
          />
          <View
            aria-label="compare/justify-mid"
            class="w-1/2 h-5 bg-[#14b8a6]"
          />
          <View
            aria-label="compare/justify-bottom"
            class="w-full h-5 bg-[#f59e0b]"
          />
        </View>
      </View>

      <View
        class="absolute p-4 border border-[#cbd5e1] rounded-xl bg-white"
        style={{ left: "24px", top: "216px", width: "440px", height: "192px" }}
      >
        <ProbeLabel>RADII + CLIPPING + TRANSFORM</ProbeLabel>
        <View class="mt-4 flex flex-row items-center gap-5">
          <View
            aria-label="compare/radius-sm"
            class="w-16 h-16 rounded-sm border-2 border-[#334155] bg-[#dbeafe]"
          />
          <View
            aria-label="compare/radius-xl"
            class="w-16 h-16 rounded-xl border-2 border-[#334155] bg-[#ccfbf1]"
          />
          <View
            aria-label="compare/radius-full"
            class="w-16 h-16 rounded-full border-2 border-[#334155] bg-[#fef3c7]"
          />
          <View
            aria-label="compare/clipped"
            class="relative w-20 h-16 rounded-lg overflow-hidden bg-[#e2e8f0]"
          >
            <View
              aria-label="compare/clipped-child"
              class="absolute w-16 h-14 bg-[#dc2626]/50"
              style={{ left: "40px", top: "28px" }}
            />
          </View>
          <View
            aria-label="compare/rotated"
            class="w-12 h-12 rotate-45 bg-[#8b5cf6]"
          />
          <View
            aria-label="compare/translated"
            class="w-8 h-12 translate-x-4 bg-[#ec4899]"
          />
        </View>
      </View>

      <View
        class="absolute p-4 border border-[#cbd5e1] rounded-xl bg-white"
        style={{ left: "480px", top: "216px", width: "456px", height: "192px" }}
      >
        <ProbeLabel>SHADOW + COMPOSITING</ProbeLabel>
        <View class="mt-5 flex flex-row items-center gap-7">
          <View
            aria-label="compare/shadow-sm"
            class="w-20 h-16 rounded-lg border border-[#e2e8f0] bg-white shadow-sm"
          />
          <View
            aria-label="compare/shadow-md"
            class="w-20 h-16 rounded-lg border border-[#e2e8f0] bg-white shadow-md"
          />
          <View
            aria-label="compare/shadow-lg"
            class="w-20 h-16 rounded-lg border border-[#e2e8f0] bg-white shadow-lg"
          />
          <View
            aria-label="compare/alpha-stack"
            class="relative w-20 h-16 bg-[#2563eb]"
          >
            <View
              aria-label="compare/alpha-over"
              class="absolute left-4 top-3 w-16 h-12 bg-[#dc2626]/50"
            />
          </View>
        </View>
      </View>

      <View
        class="absolute p-5 border border-[#cbd5e1] rounded-xl bg-white"
        style={{ left: "24px", top: "424px", width: "912px", height: "228px" }}
      >
        <ProbeLabel>TEXT METRICS (PIXEL COMPARISON DISABLED)</ProbeLabel>
        <View class="mt-4 flex flex-row items-start gap-8">
          <Text
            aria-label="compare/text-body"
            class="w-56 text-base text-[#334155]"
          >
            The quick brown fox jumps over 0123456789.
          </Text>
          <Text
            aria-label="compare/text-bold"
            class="w-56 text-lg font-bold text-[#0f172a]"
          >
            Native UI metrics
          </Text>
          <Text
            aria-label="compare/text-wrap"
            class="w-72 text-sm text-[#475569]"
          >
            A wrapped paragraph exposes differences in intrinsic width, line
            height, word spacing, and line breaking across renderers.
          </Text>
        </View>
      </View>
    </View>
  );
}
