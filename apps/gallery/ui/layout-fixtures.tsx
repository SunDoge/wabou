import { defineLayoutFixtures } from "@wabou/test/layout/fixtures";
import {
  Button,
  DirectoryPicker,
  Icon,
  ProjectionBoundary,
  Text,
  View,
} from "@wabou/ui";
import { createEffect, createSignal, onCleanup } from "solid-js";
import { CrossRenderFixture } from "../../../tests/render-reference/cross-render-fixture";
import { galleryLayoutFixtures } from "./layout-fixture-pages";
import { OverviewPage } from "./pages/overview";
import { PulseOverview } from "./pulse/overview";

let activeOwners = 0;

function TrackedFixture(props: { name: string; width: string }) {
  activeOwners++;
  onCleanup(() => activeOwners--);
  return (
    <View
      aria-label={props.name}
      class="flex flex-col gap-2 p-4"
      style={{ width: props.width }}
    >
      <Text aria-label={`${props.name} owner count`}>{activeOwners}</Text>
      <Text>{props.name}</Text>
    </View>
  );
}

function EffectFixture() {
  const [status, setStatus] = createSignal("pending");
  createEffect(
    () => "ready",
    (value) => {
      setStatus(value);
    },
  );
  return <Text aria-label="effect status">{status()}</Text>;
}

declare global {
  var __wabou_projection_probe_set_left: ((value: string) => void) | undefined;
}

function ProjectionBoundaryProbeFixture() {
  const [left, setLeft] = createSignal("left-before");
  globalThis.__wabou_projection_probe_set_left = setLeft;
  onCleanup(() => {
    delete globalThis.__wabou_projection_probe_set_left;
  });
  return (
    <View class="w-full h-full flex flex-row gap-4 p-4 bg-canvas">
      <ProjectionBoundary
        aria-label="Mutable projection boundary"
        class="w-48 h-24 p-3 bg-surface"
      >
        <Text>{left()}</Text>
      </ProjectionBoundary>
      <ProjectionBoundary
        aria-label="Stable projection boundary"
        class="w-48 h-24 p-3 bg-surface"
      >
        <Text>stable sibling</Text>
      </ProjectionBoundary>
    </View>
  );
}

defineLayoutFixtures({
  "demos/pulse-overview": { width: 1240, height: 820, render: PulseOverview },
  "demos/pulse-compact": { width: 900, height: 700, render: PulseOverview },
  "foundations/paint-reference": {
    width: 960,
    height: 720,
    render: CrossRenderFixture,
  },
  "foundations/muted-contrast": {
    width: 480,
    height: 180,
    render: () => (
      <View class="w-full h-full p-4 gap-3 bg-canvas">
        <View class="px-3 py-2 bg-surface">
          <Text class="text-sm text-muted">Muted text on a surface</Text>
        </View>
        <View class="px-3 py-2 bg-control">
          <Text class="text-sm text-muted">Muted text on a control</Text>
        </View>
        <View class="px-3 py-2 bg-control-hover">
          <Text class="text-sm text-muted">Muted text on hover</Text>
        </View>
      </View>
    ),
  },
  narrow: {
    width: 640,
    height: 480,
    render: () => <TrackedFixture name="narrow" width="120px" />,
  },
  wide: {
    width: 960,
    height: 720,
    render: () => <TrackedFixture name="wide" width="320px" />,
  },
  "effect/synchronous": () => <EffectFixture />,
  "runtime/projection-boundary": () => <ProjectionBoundaryProbeFixture />,
  "primitive/Icon": {
    width: 120,
    height: 80,
    render: () => (
      <View class="w-full h-full items-center justify-center bg-canvas">
        <Icon
          label="Layout fixture icon"
          size={18}
          source={
            '<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="8"/></svg>'
          }
        />
      </View>
    ),
  },
  "gallery/Overview": {
    width: 1280,
    height: 1200,
    render: () => (
      <OverviewPage
        theme="light"
        onCycleTheme={() => {}}
        onExplore={() => {}}
      />
    ),
  },
  "component/DirectoryPicker": {
    width: 560,
    height: 120,
    render: () => (
      <View class="w-full h-full p-6 bg-canvas">
        <DirectoryPicker
          aria-label="Fixture directory"
          browseAriaLabel="Choose fixture directory"
          browseLabel="Browse"
          value="/home/user/a-project-with-a-long-directory-name"
          onValueChange={() => {}}
        />
      </View>
    ),
  },
  ...galleryLayoutFixtures,
});
