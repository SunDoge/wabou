import {
  createComponent,
  createContext,
  createEffect,
  createSignal,
  onCleanup,
  type JSX,
  useContext,
} from "solid-js";

export type ColorThemeEasing =
  | "linear"
  | "ease-in"
  | "ease-out"
  | "ease-in-out"
  | ((progress: number) => number);

export interface ColorThemeAnimationOptions {
  /** Animation duration in seconds, matching Wabou animation helpers. */
  duration?: number;
  easing?: ColorThemeEasing;
  colorSpace?: "oklab" | "srgb";
}

export interface ColorThemeAnimation {
  readonly finished: Promise<void>;
  cancel(): void;
}

export type ColorPalette = Uint32Array;

export interface ColorThemeController {
  current(): string | undefined;
  set(name: string): void;
  getPalette(name: string): ColorPalette;
  setPalette(colors: ColorPalette): void;
  animateTo(
    name: string,
    options?: ColorThemeAnimationOptions,
  ): ColorThemeAnimation;
}

type Vec3 = readonly [number, number, number];

const [current, setCurrent] = createSignal<string>();
let currentPalette: ColorPalette | undefined;
let activeAnimation: ColorThemeAnimation | undefined;

function paletteFor(name: string): ColorPalette {
  if (!name) throw new Error("Wabou color theme name cannot be empty");
  const raw = globalThis.__wabou_get_color_theme_palette(name);
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    throw new Error(
      `Unknown Wabou color theme \`${name}\`; declare it in the \`theme.themes\` section of vite.config.ts`,
    );
  }
  if (
    !Array.isArray(parsed) ||
    !parsed.every((value) => Number.isInteger(value))
  )
    throw new Error(
      `Wabou color theme \`${name}\` returned an invalid palette`,
    );
  return Uint32Array.from(parsed as number[]);
}

function easingFunction(easing: ColorThemeEasing | undefined) {
  if (typeof easing === "function") return easing;
  switch (easing) {
    case "linear":
      return (t: number) => t;
    case "ease-in":
      return (t: number) => t * t * t;
    case "ease-in-out":
      return (t: number) =>
        t < 0.5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2;
    default:
      return (t: number) => 1 - (1 - t) ** 3;
  }
}

function srgbToLinear(value: number): number {
  return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
}

function linearToSrgb(value: number): number {
  return value <= 0.0031308
    ? 12.92 * value
    : 1.055 * value ** (1 / 2.4) - 0.055;
}

function srgbToOklab(red: number, green: number, blue: number): Vec3 {
  const r = srgbToLinear(red);
  const g = srgbToLinear(green);
  const b = srgbToLinear(blue);
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  ];
}

function oklabToSrgb(lightness: number, a: number, b: number): Vec3 {
  const l = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (lightness - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    linearToSrgb(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
    linearToSrgb(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
    linearToSrgb(-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s),
  ];
}

function channel(value: number): number {
  return Math.round(Math.min(1, Math.max(0, value)) * 255);
}

function mixColor(from: number, to: number, progress: number, oklab: boolean) {
  const fromRgb: Vec3 = [
    ((from >>> 24) & 0xff) / 255,
    ((from >>> 16) & 0xff) / 255,
    ((from >>> 8) & 0xff) / 255,
  ];
  const toRgb: Vec3 = [
    ((to >>> 24) & 0xff) / 255,
    ((to >>> 16) & 0xff) / 255,
    ((to >>> 8) & 0xff) / 255,
  ];
  const fromValue = oklab ? srgbToOklab(...fromRgb) : fromRgb;
  const toValue = oklab ? srgbToOklab(...toRgb) : toRgb;
  const mixed: Vec3 = [
    fromValue[0] + (toValue[0] - fromValue[0]) * progress,
    fromValue[1] + (toValue[1] - fromValue[1]) * progress,
    fromValue[2] + (toValue[2] - fromValue[2]) * progress,
  ];
  const [red, green, blue] = oklab ? oklabToSrgb(...mixed) : mixed;
  const alpha = channel(
    ((from & 0xff) + ((to & 0xff) - (from & 0xff)) * progress) / 255,
  );
  return (
    ((channel(red) << 24) |
      (channel(green) << 16) |
      (channel(blue) << 8) |
      alpha) >>>
    0
  );
}

function submitPalette(colors: ColorPalette) {
  currentPalette = colors;
  globalThis.__wabou_set_color_palette(colors);
}

export const colorTheme: ColorThemeController = {
  current,
  set(name) {
    activeAnimation?.cancel();
    const palette = paletteFor(name);
    globalThis.__wabou_set_color_theme(name);
    currentPalette = palette;
    setCurrent(name);
  },
  getPalette: paletteFor,
  setPalette(colors) {
    if (!(colors instanceof Uint32Array))
      throw new TypeError("Wabou color palette must be a Uint32Array");
    submitPalette(colors.slice());
  },
  animateTo(name, options = {}) {
    const target = paletteFor(name);
    const source = currentPalette?.slice();
    if (!source || options.duration === 0) {
      this.set(name);
      return { finished: Promise.resolve(), cancel() {} };
    }
    if (source.length !== target.length)
      throw new Error("Wabou color theme palettes have inconsistent lengths");

    activeAnimation?.cancel();
    const durationMs = Math.max(0, options.duration ?? 0.28) * 1000;
    const ease = easingFunction(options.easing);
    const frame = new Uint32Array(source.length);
    let raf = 0;
    let start: number | undefined;
    let settled = false;
    let finish!: () => void;
    const finished = new Promise<void>((resolve) => {
      finish = resolve;
    });
    const controls: ColorThemeAnimation = {
      finished,
      cancel() {
        if (settled) return;
        settled = true;
        cancelAnimationFrame(raf);
        finish();
      },
    };
    activeAnimation = controls;

    const tick = (timestamp: number) => {
      if (settled) return;
      start ??= timestamp;
      const linear =
        durationMs === 0 ? 1 : Math.min(1, (timestamp - start) / durationMs);
      const progress = Math.min(1, Math.max(0, ease(linear)));
      for (let index = 0; index < frame.length; index++) {
        frame[index] = mixColor(
          source[index],
          target[index],
          progress,
          options.colorSpace !== "srgb",
        );
      }
      submitPalette(frame.slice());
      if (linear < 1) {
        raf = requestAnimationFrame(tick);
        return;
      }
      settled = true;
      currentPalette = target;
      globalThis.__wabou_set_color_theme(name);
      setCurrent(name);
      if (activeAnimation === controls) activeAnimation = undefined;
      finish();
    };
    raf = requestAnimationFrame(tick);
    return controls;
  },
};

const ColorThemeContext = createContext<ColorThemeController>(colorTheme);

/** Selects one compiled color palette for the current native window. */
export function ColorThemeProvider(props: {
  theme: string;
  transition?: ColorThemeAnimationOptions | false;
  children: JSX.Element;
}): JSX.Element {
  let initialized = false;
  createEffect(
    () => [props.theme, props.transition] as const,
    ([theme, transition]) => {
      const animation =
        initialized && transition
          ? colorTheme.animateTo(theme, transition)
          : (colorTheme.set(theme), undefined);
      initialized = true;
      return animation ? () => animation.cancel() : undefined;
    },
  );
  return createComponent(ColorThemeContext, {
    value: colorTheme,
    get children() {
      return props.children;
    },
  });
}

export function useColorTheme(): ColorThemeController {
  return useContext(ColorThemeContext);
}
