import { renderComponent } from "@wabou/test/component";
import {
  normalizeProgressValue,
  Progress,
  ProgressFill,
  ProgressLabel,
  ProgressRoot,
  ProgressTrack,
  ProgressValueLabel,
  View,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("normalizes arbitrary finite ranges without invalid geometry", () => {
  expect(normalizeProgressValue(15, 10, 30)).toEqual({
    value: 15,
    min: 10,
    max: 30,
    percent: 25,
  });
  expect(normalizeProgressValue(200, 0, 100).value).toBe(100);
  expect(normalizeProgressValue(Number.NaN, 4, 4)).toEqual({
    value: 4,
    min: 4,
    max: 5,
    percent: 0,
  });
});

test("keeps the shorthand reactive and publishes normalized semantics", () => {
  const Example = () => {
    const [value, setValue] = createSignal(25);
    return (
      <View>
        <Progress label="Upload" value={value()} minValue={20} maxValue={40} />
        <ProgressRoot
          label="Update upload"
          value={value()}
          minValue={20}
          maxValue={40}
          onClick={() => setValue(35)}
        />
      </View>
    );
  };
  const screen = renderComponent(Example);
  const upload = screen.getByRole("progressbar", { name: "Upload" });

  expect(upload.attribute("aria-valuemin")).toBe("20");
  expect(upload.attribute("aria-valuemax")).toBe("40");
  expect(upload.attribute("aria-valuenow")).toBe("25");
  expect(upload.attribute("aria-valuetext")).toBe("25 percent");

  screen.getByRole("progressbar", { name: "Update upload" }).click();
  expect(upload.attribute("aria-valuenow")).toBe("35");
  expect(upload.attribute("aria-valuetext")).toBe("75 percent");
});

test("composes labels and omits a determinate value while pending", () => {
  const screen = renderComponent(() => (
    <ProgressRoot label="Preparing files" indeterminate>
      <View>
        <ProgressLabel />
        <ProgressValueLabel />
      </View>
      <ProgressTrack>
        <ProgressFill />
      </ProgressTrack>
    </ProgressRoot>
  ));
  const progress = screen.getByRole("progressbar", {
    name: "Preparing files",
  });

  expect(progress.attribute("aria-valuenow")).toBeNull();
  expect(progress.attribute("aria-valuetext")).toBeNull();
  expect(progress.text).toContain("Preparing files");
  expect(progress.text).toContain("In progress");
});
