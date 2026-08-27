import { afterEach, describe, expect, test } from "bun:test";
import { i18n, m } from "./i18n";

describe("Pi agent localization", () => {
  afterEach(() => i18n.set("en"));

  test("uses English as the promotional default", () => {
    expect(i18n.locale()).toBe("en");
    expect(String(i18n.message(m.empty_title, {}))).toBe(
      "What are we building?",
    );
  });

  test("can switch the application chrome to Chinese", () => {
    i18n.set("zh");

    expect(String(i18n.message(m.settings, {}))).toBe("设置");
    expect(String(i18n.message(m.send, {}))).toBe("发送");
  });
});
