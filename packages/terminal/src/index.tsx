import type { Handle, WabouElementProps } from "@wabou/core/renderer";
import type { JSX } from "solid-js";

export type TerminalStyle = Record<string, string | number>;

export interface TerminalExitEvent {
  type: "terminalexit";
  reason: "exit";
}

export interface TerminalProgressEvent {
  type: "terminalprogress";
  state: "remove" | "set" | "error" | "indeterminate" | "pause";
  progress: number | null;
}

export interface TerminalNotificationEvent {
  type: "terminalnotification";
  title: string;
  body: string;
}

export interface TerminalTitleChangeEvent {
  type: "terminaltitlechange";
  title: string | null;
  subtitle: string | null;
}

export interface TerminalCwdChangeEvent {
  type: "terminalcwdchange";
  path: string;
}

export interface TerminalSelectionChangeEvent {
  type: "terminalselectionchange";
  text: string | null;
  kind: "simple" | "word" | "line" | "block" | null;
}

export interface TerminalBellEvent {
  type: "terminalbell";
}

export interface TerminalProps {
  class?: string;
  style?: TerminalStyle;
  /** Native host node, useful for focus and future imperative terminal APIs. */
  ref?: (node: Handle) => void;
  /** Native focus order. Defaults to zero so the terminal accepts keyboard input. */
  focusOrder?: number;
  /** Accessible name used by native semantics and behavior-test locators. */
  "aria-label"?: string;
  /** Initial process executable. Changing it after launch does not restart the PTY. */
  command?: string;
  /** Initial process arguments. Serialized across the native widget boundary. */
  args?: readonly string[];
  /** Initial process working directory. */
  cwd?: string;
  fontFamily?: string;
  fontSize?: string;
  lineHeight?: string;
  /** Selection fill; accepts Wabou's color syntax. */
  selectionBackground?: string;
  /** Optional selected-text color. Omit it to preserve ANSI foreground colors. */
  selectionForeground?: string;
  /** Use the host element's resolved color/background as terminal defaults. */
  inheritTheme?: boolean;
  allowClipboardRead?: boolean;
  /** Mirror OSC title changes to the native window. Leave off for multi-tab apps. */
  syncWindowTitle?: boolean;
  onTerminalExit?: (event: TerminalExitEvent) => void;
  onTerminalProgress?: (event: TerminalProgressEvent) => void;
  onTerminalNotification?: (event: TerminalNotificationEvent) => void;
  onTerminalTitleChange?: (event: TerminalTitleChangeEvent) => void;
  onTerminalCwdChange?: (event: TerminalCwdChangeEvent) => void;
  onTerminalSelectionChange?: (event: TerminalSelectionChangeEvent) => void;
  /** Fires when the application writes BEL; native attention is requested too. */
  onTerminalBell?: (event: TerminalBellEvent) => void;
}

/** Typed Solid wrapper around the Rust `terminal` widget. */
export function Terminal(props: TerminalProps): JSX.Element {
  return (
    <terminal
      class={props.class}
      style={props.style}
      ref={props.ref}
      focusOrder={props.focusOrder ?? 0}
      aria-label={props["aria-label"]}
      command={props.command}
      args={props.args ? JSON.stringify(props.args) : undefined}
      cwd={props.cwd}
      font-family={props.fontFamily}
      font-size={props.fontSize}
      line-height={props.lineHeight}
      selection-background={props.selectionBackground}
      selection-foreground={props.selectionForeground}
      inherit-theme={props.inheritTheme ? "true" : undefined}
      allow-clipboard-read={props.allowClipboardRead ? "true" : undefined}
      sync-window-title={props.syncWindowTitle ? "true" : undefined}
      onTerminalExit={props.onTerminalExit}
      onTerminalProgress={props.onTerminalProgress}
      onTerminalNotification={props.onTerminalNotification}
      onTerminalTitleChange={props.onTerminalTitleChange}
      onTerminalCwdChange={props.onTerminalCwdChange}
      onTerminalSelectionChange={props.onTerminalSelectionChange}
      onTerminalBell={props.onTerminalBell}
    />
  );
}

declare module "@wabou/core/registry" {
  interface WabouIntrinsicElements {
    /** Low-level native widget. Prefer the typed PascalCase `Terminal`. */
    terminal: Omit<WabouElementProps, "style" | "ref"> & {
      style?: TerminalStyle;
      ref?: (node: Handle) => void;
      command?: string;
      args?: string;
      cwd?: string;
      "font-family"?: string;
      "font-size"?: string;
      "line-height"?: string;
      "selection-background"?: string;
      "selection-foreground"?: string;
      "inherit-theme"?: string;
      "allow-clipboard-read"?: string;
      "sync-window-title"?: string;
      onTerminalExit?: (event: TerminalExitEvent) => void;
      onTerminalProgress?: (event: TerminalProgressEvent) => void;
      onTerminalNotification?: (event: TerminalNotificationEvent) => void;
      onTerminalTitleChange?: (event: TerminalTitleChangeEvent) => void;
      onTerminalCwdChange?: (event: TerminalCwdChangeEvent) => void;
      onTerminalSelectionChange?: (event: TerminalSelectionChangeEvent) => void;
      onTerminalBell?: (event: TerminalBellEvent) => void;
    };
  }
}
