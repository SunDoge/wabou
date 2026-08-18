import { createRoot, createSignal, flush } from "solid-js";
import * as m from "./generated/messages.js";

export function runExperiment(): string[] {
  return createRoot((dispose) => {
    const [locale, setLocale] = createSignal<"en" | "zh">("en");
    const greeting = () =>
      m.greeting({ name: "Wabou" }, { locale: locale() });
    const itemCount = (count: number) =>
      m.item_count({ count }, { locale: locale() });
    const results = [
      locale(),
      greeting(),
      itemCount(1),
      itemCount(3),
    ];

    flush(() => setLocale("zh"));
    results.push(
      locale(),
      greeting(),
      itemCount(3),
    );
    dispose();
    return results;
  });
}

globalThis.__wabouParaglideExperiment = runExperiment;

declare global {
  // biome-ignore lint/style/noVar: the QuickJS probe reads a global binding.
  var __wabouParaglideExperiment: () => string[];
}
