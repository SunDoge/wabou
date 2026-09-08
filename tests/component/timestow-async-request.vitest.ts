import { expect, test } from "vitest";
import { createAsyncRequestGate } from "../../apps/timestow/ui/async-request";

test("a newer request supersedes the previous asynchronous result", () => {
  const requests = createAsyncRequestGate();
  const first = requests.begin();
  const second = requests.begin();

  expect(requests.isCurrent(first)).toBe(false);
  expect(requests.isCurrent(second)).toBe(true);
});

test("parallel work shares an epoch until its selection is invalidated", () => {
  const requests = createAsyncRequestGate();
  const root = requests.capture();
  const child = requests.capture();

  expect(requests.isCurrent(root)).toBe(true);
  expect(requests.isCurrent(child)).toBe(true);
  requests.invalidate();
  expect(requests.isCurrent(root)).toBe(false);
  expect(requests.isCurrent(child)).toBe(false);
});
