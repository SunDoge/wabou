import { expect, test } from "bun:test";
import { createEffect, createRoot, createSignal, flush } from "solid-js";
import { type CompiledMessage, createLocale } from "./i18n";

type Locale = "en" | "zh";
const greeting: CompiledMessage<{ name: string }, Locale> = (
  inputs,
  options,
) =>
  options?.locale === "zh" ? `你好，${inputs.name}！` : `Hello ${inputs.name}!`;

test("compiled messages receive the selected locale", () => {
  const changes: Locale[] = [];
  const locale = createLocale("en" as Locale, {
    onChange: (next) => changes.push(next),
  });

  expect(locale.message(greeting, { name: "Wabou" })).toBe("Hello Wabou!");
  locale.set("zh");
  expect(locale.locale()).toBe("zh");
  expect(locale.message(greeting, { name: "Wabou" })).toBe("你好，Wabou！");
  expect(changes).toEqual(["zh"]);
});

test("locale updates compose with an existing Solid flush", async () => {
  const warnings: unknown[][] = [];
  const originalWarn = console.warn;
  console.warn = (...args) => warnings.push(args);
  try {
    const observed: Locale[] = [];
    let setRequested!: (locale: Locale) => void;
    createRoot((dispose) => {
      const locale = createLocale("en" as Locale);
      const [requested, set] = createSignal<Locale>("en");
      setRequested = set;
      createEffect(requested, (value) => locale.set(value));
      createEffect(locale.locale, (value) => {
        observed.push(value);
      });
      flush();

      flush(() => setRequested("zh"));
      dispose();
    });
    await Promise.resolve();

    expect(observed).toEqual(["en", "zh"]);
    expect(
      warnings.some((args) =>
        String(args[0]).includes("FLUSH_IN_EFFECT_CALLBACK"),
      ),
    ).toBe(false);
  } finally {
    console.warn = originalWarn;
  }
});
