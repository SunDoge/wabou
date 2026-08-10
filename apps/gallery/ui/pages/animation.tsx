import {
  type AnimationControls,
  animate,
  animateKeyframes,
} from "@wabou/animation";
import {
  Alert,
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  ComponentsProvider,
  Fps,
  Input,
  Progress,
  Separator,
  Switch,
  TextArea,
  useComponentsTheme,
} from "@wabou/components";
import { createWindow, useWindow } from "@wabou/core";
import {
  createHover,
  createScrollReset,
  Button as PrimitiveButton,
  ScrollArea,
  Text,
  translate2d,
  View,
} from "@wabou/primitives";
import {
  createMemoryHistory,
  MemoryRouter,
  Route,
  useLocation,
  useNavigate,
  useParams,
} from "@wabou/router";
import {
  type Handle,
  mount,
  px,
  rgba,
  shadow,
  number as styleNumber,
} from "@wabou/core";
import wabouUtilityManifest from "@wabou/vite/utility-manifest";
import {
  createSignal,
  For,
  type JSX,
  Match,
  onCleanup,
  onMount,
  Switch as ShowCase,
} from "solid-js";
import "virtual:wabou-stylesheet";

import { Preview } from "../preview";
import { ThemeText } from "./showcase";

function AnimationPage() {
  // w-72 (288px) track minus w-10 (40px) animated item.
  const trackTravel = 248;
  const [springX, setSpringX] = createSignal(16);
  const [linearX, setLinearX] = createSignal(0);
  const [easeX, setEaseX] = createSignal(0);
  const [keyframeX, setKeyframeX] = createSignal(0);
  const [color, setColor] = createSignal("#38bdf8");
  const [opacity, setOpacity] = createSignal(0.25);
  const [paused, setPaused] = createSignal(false);
  const [speed, setSpeed] = createSignal(1);
  let animations: AnimationControls[] = [];

  const stop = () => {
    for (const animation of animations) animation.stop();
    animations = [];
  };
  const restart = () => {
    stop();
    setSpringX(16);
    setLinearX(0);
    setEaseX(0);
    setKeyframeX(0);
    setColor("#38bdf8");
    setOpacity(0.25);
    setPaused(false);
    setSpeed(1);
    animations = [
      // Springs need symmetric headroom so overshoot remains visible instead
      // of being clipped against either rounded edge of the track.
      animate(16, trackTravel - 16, {
        type: "spring",
        stiffness: 120,
        damping: 18,
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.5,
        onUpdate: setSpringX,
      }),
      animate(0, trackTravel, {
        duration: 2.4,
        ease: "linear",
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.5,
        onUpdate: setLinearX,
      }),
      animate(0, trackTravel, {
        duration: 2.4,
        ease: [0.22, 1, 0.36, 1],
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.5,
        onUpdate: setEaseX,
      }),
      animateKeyframes([0, trackTravel, 72, 212, 0], {
        duration: 6,
        times: [0, 0.3, 0.5, 0.72, 1],
        ease: "easeInOut",
        repeat: Infinity,
        repeatDelay: 0.8,
        onUpdate: setKeyframeX,
      }),
      animate("#38bdf8", "#a855f7", {
        duration: 2.4,
        ease: "easeInOut",
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.5,
        onUpdate: setColor,
      }),
      animate(0.25, 1, {
        duration: 1.8,
        ease: "easeOut",
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.8,
        onUpdate: setOpacity,
      }),
    ];
  };
  const togglePaused = () => {
    const next = !paused();
    for (const animation of animations) {
      if (next) animation.pause();
      else animation.play();
    }
    setPaused(next);
  };
  const toggleSpeed = () => {
    const next = speed() === 1 ? 0.5 : 1;
    for (const animation of animations) animation.speed = next;
    setSpeed(next);
  };

  onMount(restart);
  onCleanup(stop);

  return (
    <View class="flex flex-col gap-5">
      <Preview title="Spring physics">
        <View class="p-4">
          <View class="w-72 flex flex-col gap-4">
            <View class="h-14 flex items-center rounded-full bg-slate-800 overflow-hidden">
              <View
                class="w-10 h-10 rounded-full bg-sky-400"
                transform={translate2d(springX(), 0)}
              />
            </View>
            <View class="flex justify-between">
              <ThemeText
                dark="text-xs text-slate-500"
                light="text-xs text-slate-500"
              >
                stiffness 120
              </ThemeText>
              <ThemeText
                dark="text-xs text-slate-500"
                light="text-xs text-slate-500"
              >
                damping 18
              </ThemeText>
            </View>
          </View>
        </View>
      </Preview>

      <Preview title="Easing comparison">
        <View class="p-4">
          <View class="w-72 flex flex-col gap-3">
            <View class="h-8 flex items-center rounded-md bg-slate-800">
              <View
                class="w-10 h-6 flex items-center justify-center rounded bg-slate-400"
                transform={translate2d(linearX(), 0)}
              >
                <Text class="text-xs font-bold text-slate-950">L</Text>
              </View>
            </View>
            <View class="h-8 flex items-center rounded-md bg-slate-800">
              <View
                class="w-10 h-6 flex items-center justify-center rounded bg-emerald-400"
                transform={translate2d(easeX(), 0)}
              >
                <Text class="text-xs font-bold text-emerald-950">E</Text>
              </View>
            </View>
            <ThemeText
              dark="text-xs text-slate-500"
              light="text-xs text-slate-500"
            >
              Linear versus cubic-bezier easing
            </ThemeText>
          </View>
        </View>
      </Preview>

      <Preview title="Color and opacity interpolation">
        <View class="p-4">
          <View
            class="w-44 h-24 flex items-center justify-center rounded-xl"
            style={{ "background-color": color(), opacity: opacity() }}
          >
            <Text class="text-sm font-semibold text-white">Native paint</Text>
          </View>
        </View>
      </Preview>

      <Preview title="Keyframes">
        <View class="p-4">
          <View class="w-72 flex flex-col gap-3">
            <View class="h-12 flex items-center rounded-lg bg-slate-800 overflow-hidden">
              <View
                class="w-10 h-8 flex items-center justify-center rounded-md bg-violet-400"
                transform={translate2d(keyframeX(), 0)}
              >
                <Text class="text-xs font-bold text-violet-950">K</Text>
              </View>
            </View>
            <ThemeText
              dark="text-xs text-slate-500"
              light="text-xs text-slate-500"
            >
              0 → 248 → 72 → 212 → 0
            </ThemeText>
          </View>
        </View>
      </Preview>

      <View class="flex justify-center gap-2">
        <Button variant="outline" onClick={togglePaused}>
          {paused() ? "Resume" : "Pause"}
        </Button>
        <Button variant="outline" onClick={toggleSpeed}>
          {speed()}× speed
        </Button>
        <Button onClick={restart}>Restart animations</Button>
      </View>
    </View>
  );
}

export { AnimationPage };
