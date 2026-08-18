import { expect, test } from "bun:test";
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
