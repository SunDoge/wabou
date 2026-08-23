import { describe, expect, test } from "bun:test";
import { writer } from "@wabou/core/renderer";
import { GRAPHIC_SOURCE } from "@wabou/core/protocol";
import { createRoot, createSignal, flush } from "solid-js";
import { resolveButtonFocusOrder } from "./button";
import {
  CodeEditor,
  createButton,
  createFocus,
  createFocusWithin,
  createHover,
  createPress,
  createShortcuts,
  createTabs,
  Icon,
  Image,
  PasswordInput,
  Svg,
  Text,
  TextArea,
  TextInput,
  View,
} from "./index";

describe("interaction primitives", () => {
  test("headless button composes hover, press, focus and keyboard activation", () =>
    createRoot((dispose) => {
      let presses = 0;
      const button = createButton({ onPress: () => presses++ });
      button.bindings.onPointerEnter();
      button.bindings.onPointerDown();
      button.bindings.onFocus();
      flush();
      expect(button.state()).toMatchObject({
        hovered: true,
        pressed: true,
        focused: true,
        focusVisible: false,
      });
      button.bindings.onPointerUp();
      button.bindings.onClick({
        preventDefault() {},
        stopPropagation() {},
      });
      button.bindings.onKeyDown({
        key: "Enter",
        preventDefault() {},
        stopPropagation() {},
      });
      expect(presses).toBe(2);
      dispose();
    }));

  test("expose reactive hover and focus bindings", () =>
    createRoot((dispose) => {
      const hover = createHover();
      const focus = createFocus();
      const within = createFocusWithin();
      hover.bindings.onPointerEnter();
      focus.bindings.onFocus();
      within.bindings.onFocusIn();
      flush();
      expect([hover.hovered(), focus.focused(), within.focusWithin()]).toEqual([
        true,
        true,
        true,
      ]);
      hover.bindings.onPointerLeave();
      focus.bindings.onBlur();
      within.bindings.onFocusOut();
      flush();
      expect([hover.hovered(), focus.focused(), within.focusWithin()]).toEqual([
        false,
        false,
        false,
      ]);
      dispose();
    }));

  test("shows focus feedback for keyboard focus but not retained pointer focus", () =>
    createRoot((dispose) => {
      const button = createButton();

      button.bindings.onKeyDown({
        key: "Tab",
        preventDefault() {},
        stopPropagation() {},
      });
      button.bindings.onFocus();
      flush();
      expect(button.state()).toMatchObject({
        focused: true,
        focusVisible: true,
      });

      button.bindings.onPointerDown();
      flush();
      expect(button.state()).toMatchObject({
        focused: true,
        focusVisible: false,
      });
      dispose();
    }));

  test("honours native focus modality when no JavaScript target saw Tab", () =>
    createRoot((dispose) => {
      const focus = createFocus();
      focus.bindings.onFocus({ payload: { focusVisible: true } });
      flush();
      expect([focus.focused(), focus.focusVisible()]).toEqual([true, true]);
      dispose();
    }));

  test("normalizes press state and callback", () =>
    createRoot((dispose) => {
      let presses = 0;
      const press = createPress({ onPress: () => presses++ });
      press.bindings.onPointerDown();
      flush();
      expect(press.pressed()).toBe(true);
      press.bindings.onPointerUp();
      press.bindings.onClick({});
      flush();
      expect(press.pressed()).toBe(false);
      expect(presses).toBe(1);
      dispose();
    }));
});

describe("shortcuts primitive", () => {
  test("matches exact modifiers and normalizes chord keys", () => {
    const invoked: string[] = [];
    const shortcuts = createShortcuts({
      "Primary+T": () => invoked.push("new"),
      "Control+Shift+Tab": () => invoked.push("previous"),
    });
    let prevented = 0;
    const event = (key: string, mods: number, primary = false) => ({
      key,
      mods,
      primary,
      preventDefault: () => prevented++,
    });

    expect(shortcuts.handleKeyDown(event("T", 2, true))).toBe(true);
    expect(shortcuts.handleKeyDown(event("t", 8, true))).toBe(true);
    expect(shortcuts.handleKeyDown(event("t", 2))).toBe(false);
    expect(shortcuts.handleKeyDown(event("t", 8))).toBe(false);
    expect(shortcuts.handleKeyDown(event("Tab", 3))).toBe(true);
    expect(shortcuts.handleKeyDown(event("t", 3))).toBe(false);
    expect(invoked).toEqual(["new", "new", "previous"]);
    expect(prevented).toBe(3);
  });

  test("ignores repeats unless explicitly enabled", () => {
    let calls = 0;
    const shortcuts = createShortcuts({
      "Control+Tab": {
        handler: () => calls++,
        allowRepeat: true,
      },
      "Primary+W": () => calls++,
    });
    const preventDefault = () => {};

    expect(
      shortcuts.handleKeyDown({
        key: "Tab",
        mods: 2,
        primary: true,
        repeat: true,
        preventDefault,
      }),
    ).toBe(true);
    expect(
      shortcuts.handleKeyDown({
        key: "w",
        mods: 8,
        primary: true,
        repeat: true,
        preventDefault,
      }),
    ).toBe(false);
    expect(calls).toBe(1);
  });

  test("supports shortcuts that intentionally preserve default handling", () => {
    let prevented = false;
    const shortcuts = createShortcuts({
      Escape: {
        handler: () => {},
        preventDefault: false,
      },
    });

    expect(
      shortcuts.handleKeyDown({
        key: "Escape",
        mods: 0,
        primary: false,
        preventDefault: () => {
          prevented = true;
        },
      }),
    ).toBe(true);
    expect(prevented).toBe(false);
  });

  test("JSX bindings preserve async handler results for event diagnostics", () => {
    const result = Promise.resolve("navigated");
    const shortcuts = createShortcuts({
      "Primary+L": () => result,
    });

    expect(
      shortcuts.bindings.onKeyDown({
        key: "l",
        mods: 2,
        primary: true,
        preventDefault: () => {},
      }),
    ).toBe(result);
  });

  test("rejects invalid and overlapping shortcut declarations", () => {
    expect(() => createShortcuts({ "Hyper+T": () => {} })).toThrow(
      "Unknown shortcut modifier",
    );
    expect(() =>
      createShortcuts({
        "Primary+T": () => {},
        "Control+T": () => {},
      }),
    ).toThrow("Ambiguous shortcuts");
  });
});

describe("tabs primitive", () => {
  test("keeps stable activation while tabs are added and reordered", () =>
    createRoot((dispose) => {
      const changes: Array<string | undefined> = [];
      const tabs = createTabs({
        initialTabs: [
          { id: "one", title: "One" },
          { id: "two", title: "Two" },
        ],
        key: (tab) => tab.id,
        initialActiveKey: "two",
        onActiveChange: (key) => changes.push(key),
      });

      expect(tabs.activeTab()?.title).toBe("Two");
      expect(
        tabs.add({ id: "three", title: "Three" }, { activate: false }),
      ).toBe(true);
      flush();
      expect(tabs.activeKey()).toBe("two");
      expect(tabs.move("two", 0)).toBe(true);
      flush();
      expect(tabs.tabs().map((tab) => tab.id)).toEqual(["two", "one", "three"]);
      expect(tabs.activeKey()).toBe("two");
      expect(changes).toEqual([]);
      dispose();
    }));

  test("selects a deterministic neighbour when the active tab closes", () =>
    createRoot((dispose) => {
      const tabs = createTabs({
        initialTabs: ["one", "two", "three"],
        key: (tab) => tab,
        initialActiveKey: "two",
      });

      expect(tabs.close("two")).toBe(true);
      flush();
      expect(tabs.activeKey()).toBe("three");
      expect(tabs.close("three")).toBe(true);
      flush();
      expect(tabs.activeKey()).toBe("one");
      expect(tabs.close("one")).toBe(true);
      flush();
      expect(tabs.activeKey()).toBeUndefined();
      dispose();
    }));

  test("cycles selection and rejects duplicate identities", () =>
    createRoot((dispose) => {
      const tabs = createTabs({ initialTabs: [1, 2, 3], key: (tab) => tab });

      tabs.selectPrevious();
      flush();
      expect(tabs.activeKey()).toBe(3);
      tabs.selectNext();
      flush();
      expect(tabs.activeKey()).toBe(1);
      tabs.selectLast();
      flush();
      expect(tabs.activeKey()).toBe(3);
      tabs.selectFirst();
      flush();
      expect(tabs.activeKey()).toBe(1);
      expect(tabs.add(1)).toBe(false);
      expect(tabs.tabs()).toEqual([1, 2, 3]);
      dispose();
    }));

  test("rejects duplicate initial keys", () => {
    expect(() =>
      createRoot((dispose) => {
        createTabs({ initialTabs: [1, 1], key: (tab) => tab });
        dispose();
      }),
    ).toThrow("Duplicate tab key: 1");
  });

  test("moves native focus with keyboard tab navigation", () =>
    createRoot((dispose) => {
      const focused: string[] = [];
      const tabs = createTabs({
        initialTabs: ["one", "two", "three"],
        key: (tab) => tab,
      });
      const handle = (key: string) => ({ focus: () => focused.push(key) });
      for (const key of tabs.tabs()) tabs.register(key, handle(key));
      let prevented = 0;

      expect(
        tabs.handleKeyDown("one", {
          key: "ArrowLeft",
          preventDefault: () => prevented++,
        }),
      ).toBe(true);
      flush();
      expect(tabs.activeKey()).toBe("three");
      expect(focused).toEqual(["three"]);

      expect(tabs.handleKeyDown("three", { key: "Home" })).toBe(true);
      flush();
      expect(tabs.activeKey()).toBe("one");
      expect(focused).toEqual(["three", "one"]);
      expect(tabs.handleKeyDown("one", { key: "Enter" })).toBe(false);
      expect(prevented).toBe(1);
      dispose();
    }));

  test("uses up and down arrows for vertical tabs", () =>
    createRoot((dispose) => {
      const tabs = createTabs({
        initialTabs: [1, 2],
        key: (tab) => tab,
        orientation: "vertical",
      });

      expect(tabs.handleKeyDown(1, { key: "ArrowRight" })).toBe(false);
      expect(tabs.handleKeyDown(1, { key: "ArrowDown" })).toBe(true);
      flush();
      expect(tabs.activeKey()).toBe(2);
      dispose();
    }));
});

describe("host primitives", () => {
  test("button preserves explicit roving tab order unless disabled", () => {
    expect(resolveButtonFocusOrder(false, -1)).toBe(-1);
    expect(resolveButtonFocusOrder(false, 4)).toBe(4);
    expect(resolveButtonFocusOrder(false, undefined)).toBe(0);
    expect(resolveButtonFocusOrder(true, 4)).toBe(-1);
  });

  test("authors primitive semantics in JavaScript", () => {
    const semantics: Array<[string, string]> = [];
    const setAttribute = writer.setAttribute.bind(writer);
    const setTextBehavior = writer.setTextBehavior.bind(writer);
    const setTextMaxLines = writer.setTextMaxLines.bind(writer);
    writer.setAttribute = (_id, name, value) => {
      if (name === "role") {
        semantics.push([name, value]);
      }
    };
    writer.setTextBehavior = (_id, flags) => {
      semantics.push(["textBehavior", String(flags)]);
    };
    writer.setTextMaxLines = (_id, maxLines) => {
      semantics.push(["textMaxLines", String(maxLines)]);
    };
    try {
      Text({});
      Text({ maxLines: 2 });
      Svg({ source: "<svg/>" });
      Image({ resource: { lo: 2, hi: 1 } });
    } finally {
      writer.setAttribute = setAttribute;
      writer.setTextBehavior = setTextBehavior;
      writer.setTextMaxLines = setTextMaxLines;
    }
    expect(semantics).toEqual([
      ["role", "label"],
      ["textBehavior", "3"],
      ["textMaxLines", "0"],
      ["role", "label"],
      ["textBehavior", "1"],
      ["textMaxLines", "2"],
      ["role", "img"],
      ["role", "img"],
    ]);
  });

  test("maps explicit image resource handles onto the native protocol", () => {
    const sources: Array<[number, string]> = [];
    const setGraphicSource = writer.setGraphicSource.bind(writer);
    writer.setGraphicSource = (_id, kind, source) =>
      sources.push([kind, source]);
    try {
      Image({ resource: { lo: 42, hi: 3 } });
    } finally {
      writer.setGraphicSource = setGraphicSource;
    }
    expect(sources).toEqual([[GRAPHIC_SOURCE.ResourceRaster, "42:3"]]);
  });

  test("rejects invalid text line limits before crossing the bridge", () => {
    expect(() => Text({ maxLines: 0 })).toThrow(RangeError);
    expect(() => Text({ maxLines: 1.5 })).toThrow(RangeError);
  });

  test("authors native editor focus policy in JavaScript", () => {
    const attributes: Array<[string, string]> = [];
    const focusOrders: number[] = [];
    const setAttribute = writer.setAttribute.bind(writer);
    const setInteractionPolicy = writer.setInteractionPolicy.bind(writer);
    writer.setAttribute = (_id, name, value) => {
      if (name === "role" || name === "aria-disabled") {
        attributes.push([name, value]);
      }
    };
    writer.setInteractionPolicy = (_id, flags, focusOrder) => {
      if ((flags & 0x01) !== 0) focusOrders.push(focusOrder);
    };
    try {
      TextInput({});
      TextArea({});
      PasswordInput({ secret: "test-secret", focusOrder: 4 });
      CodeEditor({ "aria-label": "Config", disabled: true });
    } finally {
      writer.setAttribute = setAttribute;
      writer.setInteractionPolicy = setInteractionPolicy;
    }
    expect(attributes).toEqual([
      ["role", "textbox"],
      ["aria-disabled", "false"],
      ["role", "textbox"],
      ["aria-disabled", "false"],
      ["role", "textbox"],
      ["aria-disabled", "false"],
      ["role", "textbox"],
      ["aria-disabled", "true"],
    ]);
    expect(focusOrders).toEqual([0, 0, 4, -1]);
  });

  test("create explicit view, text, image, and editor host nodes", () =>
    createRoot((dispose) => {
      const view = View({}) as unknown as {
        tag: string;
      };
      const text = Text({ children: [0, " stories"] }) as unknown as {
        tag: string;
        firstChild: { tag: string } | null;
      };
      const image = Image({
        resource: { lo: 2, hi: 1 },
      }) as unknown as {
        tag: string;
      };
      const textarea = TextArea({ value: "two\nlines" }) as unknown as {
        tag: string;
      };
      const input = TextInput({ value: "one line" }) as unknown as {
        tag: string;
      };
      const password = PasswordInput({
        secret: "master-password",
      }) as unknown as {
        tag: string;
      };

      expect(view.tag).toBe("view");
      expect(text.tag).toBe("text");
      expect(text.firstChild?.tag).toBe("#text");
      expect(image.tag).toBe("img");
      expect(input.tag).toBe("input");
      expect(textarea.tag).toBe("textarea");
      expect(password.tag).toBe("password-input");
      dispose();
    }));

  test("Icon always adds alignment defaults and preserves custom class", () => {
    const classes: Array<[string, string]> = [];
    const attributes: Array<[string, string]> = [];
    const style: Array<[string, string]> = [];
    const setAttribute = writer.setAttribute.bind(writer);
    const setClassName = writer.setClassName.bind(writer);
    const setStyle = writer.setStyle.bind(writer);
    writer.setClassName = (_id, value) => {
      classes.push(["class", value]);
    };
    writer.setAttribute = (_id, name, value) => {
      if (name === "width" || name === "height") {
        attributes.push([name, value]);
      }
    };
    writer.setStyle = (_id, name, value) => {
      style.push([name, value]);
    };
    try {
      createRoot((dispose) => {
        Icon({
          source: "<svg/>",
          size: 14,
          class: "text-accent",
          label: "demo icon",
        });
        dispose();
      });
    } finally {
      writer.setAttribute = setAttribute;
      writer.setClassName = setClassName;
      writer.setStyle = setStyle;
    }
    expect(classes).toEqual([["class", "self-center shrink-0 text-accent"]]);
    expect(attributes).toEqual([
      ["width", "14"],
      ["height", "14"],
    ]);
    expect(style).toEqual(
      expect.arrayContaining([
        ["display", "inline-flex"],
        ["align-items", "center"],
        ["justify-content", "center"],
        ["align-self", "center"],
        ["flex-shrink", "0"],
        ["width", "14"],
        ["height", "14"],
        ["line-height", "1"],
      ]),
    );
  });

  test("Icon defaults to 1em when size is not provided", () => {
    const attributes: Array<[string, string]> = [];
    const style: Array<[string, string]> = [];
    const setAttribute = writer.setAttribute.bind(writer);
    const setStyle = writer.setStyle.bind(writer);
    writer.setAttribute = (_id, name, value) => {
      if (name === "width" || name === "height") {
        attributes.push([name, value]);
      }
    };
    writer.setStyle = (_id, name, value) => {
      if (name === "width" || name === "height") {
        style.push([name, value]);
      }
    };
    try {
      createRoot((dispose) => {
        Icon({ source: "<svg/>", class: "text-muted" });
        dispose();
      });
    } finally {
      writer.setAttribute = setAttribute;
      writer.setStyle = setStyle;
    }
    expect(attributes).toEqual([]);
    expect(style).toEqual(
      expect.arrayContaining([
        ["width", "1em"],
        ["height", "1em"],
      ]),
    );
  });

  test("Icon normalizes unitless string size to numeric px values", () => {
    const attributes: Array<[string, string]> = [];
    const setAttribute = writer.setAttribute.bind(writer);
    writer.setAttribute = (_id, name, value) => {
      if (name === "width" || name === "height") {
        attributes.push([name, value]);
      }
    };
    try {
      createRoot((dispose) => {
        Icon({ source: "<svg/>", size: "17" });
        dispose();
      });
    } finally {
      writer.setAttribute = setAttribute;
    }
    expect(attributes).toEqual([
      ["width", "17"],
      ["height", "17"],
    ]);
  });

  test("Icon keeps unit suffix strings untouched for CSS-size values", () => {
    const styles: Array<[string, string]> = [];
    const setStyle = writer.setStyle.bind(writer);
    writer.setStyle = (_id, name, value) => {
      if (name === "width" || name === "height") {
        styles.push([name, value]);
      }
    };
    try {
      createRoot((dispose) => {
        Icon({ source: "<svg/>", size: "1.5rem" });
        dispose();
      });
    } finally {
      writer.setStyle = setStyle;
    }
    expect(styles).toEqual(
      expect.arrayContaining([
        ["width", "1.5rem"],
        ["height", "1.5rem"],
      ]),
    );
  });

  test("Icon size updates when the prop changes", () => {
    const styles: Array<[string, string]> = [];
    const setStyle = writer.setStyle.bind(writer);
    writer.setStyle = (_id, name, value) => {
      if (name === "width" || name === "height") {
        styles.push([name, value]);
      }
    };
    try {
      createRoot((dispose) => {
        const [size, setSize] = createSignal<number | string>(14);
        Icon({
          source: "<svg/>",
          get size() {
            return size();
          },
        });
        flush();
        expect(styles).toEqual(
          expect.arrayContaining([
            ["width", "14"],
            ["height", "14"],
          ]),
        );
        styles.length = 0;
        setSize("1.5rem");
        flush();
        expect(styles).toEqual(
          expect.arrayContaining([
            ["width", "1.5rem"],
            ["height", "1.5rem"],
          ]),
        );
        dispose();
      });
    } finally {
      writer.setStyle = setStyle;
    }
  });

  test("Icon treats padded unit strings as CSS sizes", () => {
    const styles: Array<[string, string]> = [];
    const setStyle = writer.setStyle.bind(writer);
    writer.setStyle = (_id, name, value) => {
      if (name === "width" || name === "height") {
        styles.push([name, value]);
      }
    };
    try {
      createRoot((dispose) => {
        Icon({ source: "<svg/>", size: " 1em " });
        dispose();
      });
    } finally {
      writer.setStyle = setStyle;
    }
    expect(styles).toEqual(
      expect.arrayContaining([
        ["width", "1em"],
        ["height", "1em"],
      ]),
    );
  });

  test("Icon treats blank size as default", () => {
    const styles: Array<[string, string]> = [];
    const setStyle = writer.setStyle.bind(writer);
    writer.setStyle = (_id, name, value) => {
      if (name === "width" || name === "height") {
        styles.push([name, value]);
      }
    };
    try {
      createRoot((dispose) => {
        Icon({ source: "<svg/>", size: "" });
        dispose();
      });
    } finally {
      writer.setStyle = setStyle;
    }
    expect(styles).toEqual(
      expect.arrayContaining([
        ["width", "1em"],
        ["height", "1em"],
      ]),
    );
  });
});
