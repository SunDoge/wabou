import { View } from "@wabou/ui";

/**
 * A renderer-neutral paint and geometry probe. The Wabou fixture runner uses
 * the native primitives, while the browser comparison aliases `@wabou/ui` to
 * a DOM adapter. Keep this component inside the common primitive subset.
 */
export function CrossRenderFixture() {
  return (
    <View
      aria-label="compare/root"
      class="relative w-full h-full overflow-hidden bg-[#f7f8fa]"
    >
      <View
        aria-label="compare/solid"
        class="absolute left-10 top-10 w-28 h-20 bg-[#2563eb]"
      />
      <View
        aria-label="compare/alpha-base"
        class="absolute left-48 top-10 w-28 h-20 bg-[#2563eb]"
      />
      <View
        aria-label="compare/alpha-over"
        class="absolute w-20 h-16 bg-[#dc2626]/50"
        style={{ left: "232px", top: "60px" }}
      />
      <View
        aria-label="compare/rounded-clip"
        class="absolute left-10 top-40 w-36 h-24 rounded-2xl border-2 border-strong bg-surface overflow-hidden"
      >
        <View
          aria-label="compare/clipped-child"
          class="ml-20 mt-12 w-24 h-20 bg-[#14b8a6]"
        />
      </View>
      <View
        aria-label="compare/shadow"
        class="absolute left-56 top-40 w-40 h-24 rounded-xl border border-subtle bg-surface shadow-sm"
      />
      <View
        aria-label="compare/translucent"
        class="absolute left-10 top-72 w-96 h-16 rounded-xl bg-[#111827]/25"
      />
    </View>
  );
}
