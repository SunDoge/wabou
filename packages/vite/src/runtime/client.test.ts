import { expect, test } from "bun:test";
import {
  beginFullReload,
  commitFullReload,
  createHotContext,
  rollbackFullReload,
} from "./client";

test("failed full reload restores the last-good hot records", () => {
  const owner = `/last-good-${Date.now()}.tsx`;
  const current = createHotContext(owner);
  current.data.revision = "current";

  beginFullReload();
  const broken = createHotContext(owner);
  broken.data.revision = "broken";
  rollbackFullReload();

  const restored = createHotContext(owner);
  expect(restored.data).toBe(current.data);
  expect(restored.data.revision).toBe("current");
});

test("successful full reload commits the replacement hot records", () => {
  const owner = `/replacement-${Date.now()}.tsx`;
  const current = createHotContext(owner);
  current.data.revision = "current";

  beginFullReload();
  const replacement = createHotContext(owner);
  replacement.data.revision = "replacement";
  commitFullReload();

  const committed = createHotContext(owner);
  expect(committed.data).toBe(replacement.data);
  expect(committed.data).not.toBe(current.data);
  expect(committed.data.revision).toBe("replacement");
});
