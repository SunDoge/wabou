import { renderComponent } from "@wabou/test/component";
import {
  AnnotationLayer,
  type AnnotationRegion,
  clampAnnotationRegion,
  ImageOverlayLayer,
  ImageViewport,
  imageViewportTransform,
  View,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("uses one invertible contain transform for media and annotations", () => {
  const transform = imageViewportTransform({
    viewport: { width: 400, height: 300 },
    image: { width: 200, height: 100 },
    zoom: 1.5,
    pan: { x: 10, y: -5 },
  });

  expect(transform.scale).toBe(3);
  expect(transform.frame).toEqual({ x: -90, y: -5, width: 600, height: 300 });
  const viewportPoint = transform.imageToViewport({ x: 25, y: 40 });
  expect(viewportPoint).toEqual({ x: -15, y: 115 });
  expect(transform.viewportToImage(viewportPoint)).toEqual({ x: 25, y: 40 });
});

test("pans media and read-only overlays through one controlled transform", () => {
  const Harness = () => {
    const [pan, setPan] = createSignal({ x: 0, y: 0 });
    return (
      <ImageViewport
        aria-label="Pannable page"
        imageSize={{ width: 200, height: 100 }}
        media={<View class="w-full h-full bg-surface" />}
        pannable
        pan={pan()}
        onPanChange={setPan}
      >
        <ImageOverlayLayer
          aria-label="Translated text"
          items={[{ id: "line", x: 20, y: 10, width: 40, height: 20 }]}
        >
          {() => <View role="note" aria-label="Translation" />}
        </ImageOverlayLayer>
      </ImageViewport>
    );
  };
  const screen = renderComponent(Harness);
  const viewport = screen.getByRole("group", { name: "Pannable page" });
  viewport.resize({ width: 400, height: 300 });
  const overlay = screen.getByRole("note", { name: "Translation" }).parent!;
  expect(overlay.style("left")).toBe("40px");
  expect(overlay.style("top")).toBe("70px");

  viewport.pointerDown({ clientX: 100, clientY: 100 });
  viewport.pointerMove({ clientX: 130, clientY: 120 });
  viewport.pointerUp({ clientX: 130, clientY: 120 });
  expect(overlay.style("left")).toBe("70px");
  expect(overlay.style("top")).toBe("90px");
});

test("clamps editable regions to image space", () => {
  expect(
    clampAnnotationRegion(
      { id: "outside", x: 95, y: 45, width: 20, height: 20 },
      { width: 100, height: 50 },
      4,
    ),
  ).toEqual({ id: "outside", x: 95, y: 45, width: 5, height: 5 });
});

test("creates and moves regions through the authored pointer path", () => {
  const Harness = () => {
    const [regions, setRegions] = createSignal<readonly AnnotationRegion[]>([
      {
        id: "dialogue",
        label: "Dialogue",
        x: 20,
        y: 10,
        width: 40,
        height: 30,
      },
    ]);
    return (
      <ImageViewport
        aria-label="Manga page"
        imageSize={{ width: 200, height: 100 }}
        media={<View class="w-full h-full bg-surface" />}
      >
        <AnnotationLayer
          aria-label="OCR regions"
          regions={regions()}
          createRegionId={() => "created"}
          onRegionsChange={setRegions}
        />
      </ImageViewport>
    );
  };
  const screen = renderComponent(Harness);
  const viewport = screen.getByRole("group", { name: "Manga page" });
  viewport.resize({ width: 400, height: 300 });
  let dialogue = screen.getByRole("button", { name: "Dialogue" });
  expect(dialogue.style("left")).toBe("40px");
  expect(dialogue.style("top")).toBe("70px");
  expect(dialogue.style("width")).toBe("80px");
  dialogue.pointerDown({ clientX: 100, clientY: 100 });
  dialogue.pointerMove({ clientX: 140, clientY: 120 });
  dialogue.pointerUp({ clientX: 140, clientY: 120 });
  expect(dialogue.style("left")).toBe("80px");
  expect(dialogue.style("top")).toBe("90px");
  dialogue = screen.getByRole("button", { name: "Dialogue" });
  dialogue.pointerDown({ clientX: 140, clientY: 120 });
  dialogue.pointerMove({ clientX: 1_000, clientY: 1_000 });
  dialogue.pointerUp({ clientX: 1_000, clientY: 1_000 });
  expect(dialogue.style("left")).toBe("320px");
  expect(dialogue.style("top")).toBe("190px");
  expect(dialogue.style("width")).toBe("80px");
  const layer = screen.getByRole("group", { name: "OCR regions" });
  layer.pointerDown({ offsetX: 20, offsetY: 60 });
  layer.pointerMove({ offsetX: 100, offsetY: 120 });
  layer.pointerUp({ offsetX: 100, offsetY: 120 });
  const created = screen.getByRole("button", { name: "Annotation created" });
  expect(created.style("left")).toBe("20px");
  expect(created.style("top")).toBe("60px");
  expect(created.style("width")).toBe("80px");
  expect(created.style("height")).toBe("60px");
});

test("pans through a passthrough annotation layer", () => {
  let pan = { x: 0, y: 0 };
  const screen = renderComponent(() => {
    const [value, setValue] = createSignal(pan);
    return (
      <ImageViewport
        aria-label="Page viewport"
        imageSize={{ width: 800, height: 1200 }}
        pan={value()}
        pannable
        onPanChange={(next) => {
          pan = next;
          setValue(next);
        }}
        media={<View />}
      >
        <AnnotationLayer
          aria-label="Regions"
          regions={[]}
          interactionMode="passthrough"
        />
      </ImageViewport>
    );
  });
  const viewport = screen.getByRole("group", { name: "Page viewport" });
  viewport.pointerDown({ clientX: 20, clientY: 30 });
  viewport.pointerMove({ clientX: 65, clientY: 80 });
  viewport.pointerUp({ clientX: 65, clientY: 80 });
  expect(pan).toEqual({ x: 45, y: 50 });
});
