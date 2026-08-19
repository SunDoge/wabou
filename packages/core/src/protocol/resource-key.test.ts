import { expect, test } from "bun:test";
import { createResourceKeyFamily } from "./resource-key";

const imageKeys = createResourceKeyFamily("image");
const fontKeys = createResourceKeyFamily("font");

test("resource key families reject accidental cross-family handles", () => {
  const image = imageKeys.fromParts(7, 3);
  const font = fontKeys.fromParts(7, 3);

  expect(imageKeys.is(image)).toBe(true);
  expect(imageKeys.is(font)).toBe(false);
  expect(imageKeys.equals(image, font as never)).toBe(false);
  expect(() => imageKeys.assert(font)).toThrow("image resource key");
});

test("resource keys validate JSON and preserve both SlotMap halves", () => {
  const key = imageKeys.fromJSON({ lo: 0xffff_fffe, hi: 0xffff_ffff });
  expect(key).toMatchObject({ lo: 0xffff_fffe, hi: 0xffff_ffff });
  expect(imageKeys.format(key)).toBe("image:4294967294v4294967295");
  expect(() => imageKeys.fromJSON({ lo: 1, hi: 2 })).toThrow();
});

test("resource tables reject stale generations and other families", () => {
  const table = imageKeys.table<string>();
  const live = imageKeys.fromParts(4, 3);
  const stale = imageKeys.fromParts(4, 1);
  const wrongFamily = fontKeys.fromParts(4, 3);

  table.set(live, "pixels");
  expect(table.get(live)).toBe("pixels");
  expect(table.get(stale)).toBeUndefined();
  expect(table.get(wrongFamily as never)).toBeUndefined();
  expect(table.delete(stale)).toBe(false);
  expect(table.delete(live)).toBe(true);
});
