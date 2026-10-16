import { DevServerErrorOverlay } from "./dev-server-error.mjs";
import { shadow } from "@wabou/core/style";
import { createComponent, createContext, getOwner, useContext } from "solid-js";
//#region src/components/theme.ts
/**
* Geometry contract for Wabou's default desktop theme.
*
* Components consume these recipes instead of independently choosing height,
* padding, radius, and icon rhythm. The values intentionally favor desktop
* density over touch-first sizing.
*/
const componentsThemeContract = Object.freeze({
	controlHeight: Object.freeze({
		sm: 28,
		default: 32,
		lg: 40,
		icon: 32
	}),
	controlPaddingX: Object.freeze({
		sm: 8,
		default: 10,
		lg: 12,
		icon: 0
	}),
	iconSize: Object.freeze({
		sm: 14,
		default: 16,
		lg: 18
	}),
	typography: Object.freeze({
		xs: Object.freeze({
			size: 12,
			lineHeight: 16
		}),
		sm: Object.freeze({
			size: 14,
			lineHeight: 20
		}),
		md: Object.freeze({
			size: 16,
			lineHeight: 24
		}),
		lg: Object.freeze({
			size: 18,
			lineHeight: 28
		}),
		xl: Object.freeze({
			size: 20,
			lineHeight: 28
		})
	}),
	controlRadius: 6,
	containerRadius: 8,
	containerPadding: 20,
	sectionGap: 16,
	/** Disabled controls remain readable while losing active-surface emphasis. */
	disabledOpacity: .6
});
/**
* Shared disabled chrome for framed controls.
*
* Opacity alone leaves primary and destructive controls looking actionable,
* especially on light surfaces. Disabled controls therefore move onto the
* neutral surface palette as well as losing emphasis and pointer affordance.
*/
function componentsDisabledControlClass(disabled) {
	return disabled ? "cursor-not-allowed border-subtle bg-surface-muted text-muted opacity-60" : "";
}
/** Disabled treatment for controls whose checked/selected color must remain visible. */
function componentsDisabledInteractiveClass(disabled) {
	return disabled ? "cursor-not-allowed opacity-60" : "";
}
/** Disabled content nested inside a surface that already owns the opacity. */
function componentsDisabledContentClass(disabled) {
	return disabled ? "cursor-not-allowed text-muted" : "";
}
/** Shared unavailable-row treatment for listboxes, menus, and command surfaces. */
function componentsDisabledItemClass(disabled) {
	return disabled ? "cursor-not-allowed bg-surface-muted text-muted opacity-60" : "";
}
function componentsControlContentSize(size) {
	switch (size) {
		case "sm": return "h-7 px-2 gap-1 text-xs";
		case "lg": return "h-10 px-3 gap-2 text-base";
		case "icon": return "w-8 h-8 p-0 gap-0 text-sm";
		default: return "h-8 px-2.5 gap-2 text-sm";
	}
}
function componentsControlSize(size) {
	return `${componentsControlContentSize(size)} rounded-md`;
}
/**
* Shared native panel chrome. Component composition owns padding and layout;
* this contract owns the edge geometry that must not drift between surfaces.
*/
function componentsSurfaceClass(surface) {
	switch (surface) {
		case "raised": return "rounded-lg border border-subtle bg-surface";
		case "floating": return "rounded-lg border border-strong bg-surface";
		case "modal": return "rounded-xl border border-strong bg-surface";
	}
}
/**
* Native elevation recipes adapted from gpui-component. Both implementations
* pass standard deviation directly to their renderer, so these values should
* not use CSS's doubled blur radius. Floating surfaces also carry a subtle
* foreground-colored ring: black in light mode, white in dark mode.
*/
function componentsElevation(theme, elevation) {
	if (elevation === "raised") return [
		shadow({
			spread: 1,
			stdDev: 0,
			color: 10
		}),
		shadow({
			offsetY: 1,
			stdDev: 1,
			color: 20
		}),
		shadow({
			offsetY: 4,
			stdDev: 5,
			spread: -2,
			color: 16
		})
	];
	if (elevation === "floating") return [
		shadow({
			spread: 1,
			stdDev: 0,
			color: theme === "dark" ? 4294967071 : 20
		}),
		shadow({
			offsetY: 8,
			stdDev: 12,
			spread: -5,
			color: 36
		}),
		shadow({
			offsetY: 2,
			stdDev: 3,
			spread: -2,
			color: 31
		})
	];
	return [shadow({
		offsetY: 20,
		stdDev: 25,
		spread: -5,
		color: 26
	}), shadow({
		offsetY: 8,
		stdDev: 10,
		spread: -6,
		color: 26
	})];
}
const defaultTheme = { theme: () => "light" };
const ThemeContext = createContext(defaultTheme);
function ComponentsRoot(props) {
	return [props.children, createComponent(DevServerErrorOverlay, {})];
}
function ComponentsProvider(props) {
	return createComponent(ThemeContext, {
		value: { theme: () => props.theme ?? "light" },
		get children() {
			return createComponent(ComponentsRoot, { get children() {
				return props.children;
			} });
		}
	});
}
function useComponentsTheme() {
	return (getOwner() ? useContext(ThemeContext) : defaultTheme).theme;
}
//#endregion
export { ComponentsProvider, componentsControlContentSize, componentsControlSize, componentsDisabledContentClass, componentsDisabledControlClass, componentsDisabledInteractiveClass, componentsDisabledItemClass, componentsElevation, componentsSurfaceClass, componentsThemeContract, useComponentsTheme };

//# sourceMappingURL=theme.mjs.map