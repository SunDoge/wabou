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

  expect(upload.minNumericValue).toBe(20);
  expect(upload.maxNumericValue).toBe(40);
  expect(upload.numericValue).toBe(25);
  expect(upload.valueText).toBe("25 percent");

  screen.getByRole("progressbar", { name: "Update upload" }).click();
  expect(upload.numericValue).toBe(35);
  expect(upload.valueText).toBe("75 percent");
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

  expect(progress.numericValue).toBeNull();
  expect(progress.valueText).toBeNull();
  expect(progress.text).toContain("Preparing files");
  expect(progress.text).toContain("In progress");
});

test("owns native size variants and closes the fill only when complete", () => {
  const screen = renderComponent(() => (
    <View>
      <Progress label="Tiny task" value={25} size="xs" />
      <Progress label="Small task" value={50} size="sm" />
      <Progress label="Default task" value={75} />
      <Progress label="Large task" value={100} size="lg" />
    </View>
  ));

  const progressParts = (name: string) => {
    const root = screen.getByRole("progressbar", { name });
    const track = root.children[0];
    const fill = track?.children[0];
    if (!track || !fill) throw new Error(`${name} did not render its parts`);
    return { track, fill };
  };

  expect(progressParts("Tiny task").track.className).toContain("h-1");
  expect(progressParts("Small task").track.className).toContain("h-1.5");
  expect(progressParts("Default task").track.className).toContain("h-2");
  expect(progressParts("Large task").track.className).toContain("h-2.5");
  expect(progressParts("Default task").fill.className).toContain(
    "rounded-r-none",
  );
  expect(progressParts("Large task").fill.className).not.toContain(
    "rounded-r-none",
  );
});
