import { createRoot, createSignal } from "solid-js";

globalThis.__wabouParaglideBaseline = () =>
  createRoot((dispose) => {
    const [locale, setLocale] = createSignal<"en" | "zh">("en");
    const results = [locale()];
    setLocale("zh");
    results.push(locale());
    dispose();
    return results;
  });

declare global {
  // biome-ignore lint/style/noVar: the bundle-size baseline uses a global binding.
  var __wabouParaglideBaseline: () => string[];
}
