import { resolve } from "node:path";
import {
  assertLayoutRectContains,
  assertLayoutTextStyle,
  getLayoutNode,
  type LayoutSnapshot,
  layoutRectBottom,
  layoutRectRight,
  queryLayoutNodes,
} from "@wabou/test/layout";
import {
  type LayoutFixtureCase,
  renderLayoutFixtures,
} from "@wabou/test/layout/node";

const command = process.env.WABOU_LAYOUT_COMMAND
  ? process.env.WABOU_LAYOUT_COMMAND.split(" ").filter(Boolean)
  : [resolve("target/release/wabou")];
const selected = process.argv.slice(2).filter(Boolean);
const checks = [
  "visible-overflow",
  "sibling-collision",
  "visual-quality",
] as const;

const assertClose = (actual: number, expected: number, label: string) => {
  if (Math.abs(actual - expected) > 1)
    throw new Error(`${label}: expected ${expected}, received ${actual}`);
};

const assertIconLayout = (snapshot: LayoutSnapshot) => {
  const icon = getLayoutNode(snapshot, {
    role: "img",
    name: "Layout fixture icon",
  });
  assertClose(icon.rect.width, 18, "icon width");
  assertClose(icon.rect.height, 18, "icon height");
};

const assertOverviewSvgPaint = (snapshot: LayoutSnapshot) => {
  const icons = queryLayoutNodes(snapshot, { tag: "svg" });
  if (icons.length < 12)
    throw new Error(
      `Gallery overview projected only ${icons.length} SVG icons`,
    );
  for (const icon of icons) {
    if (icon.rect.width <= 0 || icon.rect.height <= 0) {
      throw new Error(
        `Gallery overview SVG ${icon.id.lo}:${icon.id.hi} has empty paint bounds`,
      );
    }
  }
};

const assertSidebarLayout = (snapshot: LayoutSnapshot) => {
  const boundary = getLayoutNode(snapshot, {
    name: "Sidebar fixture boundary",
  });
  const sidebar = getLayoutNode(snapshot, { name: "Fixture sidebar" });
  const navigation = getLayoutNode(snapshot, { name: "Fixture navigation" });
  const footer = getLayoutNode(snapshot, { name: "Fixture sidebar footer" });
  assertLayoutRectContains(boundary.contentRect, sidebar.rect, {
    label: "sidebar",
  });
  assertLayoutRectContains(sidebar.rect, navigation.rect, {
    label: "sidebar navigation",
  });
  assertLayoutRectContains(sidebar.rect, footer.rect, {
    label: "sidebar footer",
  });
  assertClose(layoutRectBottom(navigation.rect), footer.rect.y, "footer edge");
  assertClose(
    layoutRectBottom(footer.rect),
    layoutRectBottom(sidebar.rect),
    "sidebar bottom edge",
  );
  if (navigation.computed.overflowY !== "Scroll")
    throw new Error("sidebar content did not establish a scroll viewport");
  const selected = getLayoutNode(snapshot, {
    role: "button",
    name: "Section 4",
  });
  if (new Map(selected.attrs).get("aria-selected") !== "true") {
    throw new Error("sidebar fixture did not retain its selected destination");
  }
};

const assertScrollAreaLayout = (snapshot: LayoutSnapshot) => {
  const viewport = getLayoutNode(snapshot, {
    role: "region",
    name: "Fixture scroll viewport",
  });
  const finalRow = getLayoutNode(snapshot, { text: "Scrollable row 12" });
  if (viewport.computed.overflowY !== "Scroll")
    throw new Error("ScrollArea did not retain overflow-y scrolling");
  if (layoutRectBottom(finalRow.rect) <= layoutRectBottom(viewport.contentRect))
    throw new Error("ScrollArea fixture did not produce a real scroll range");
};

const assertSliderLayout = (snapshot: LayoutSnapshot) => {
  const slider = getLayoutNode(snapshot, {
    role: "slider",
    name: "Fixture volume",
  });
  assertClose(slider.rect.width, 384, "slider width");
  assertClose(slider.rect.height, 28, "slider height");
};

const assertTabsLayout = (snapshot: LayoutSnapshot) => {
  const list = getLayoutNode(snapshot, {
    role: "tablist",
    name: "Fixture settings sections",
  });
  const panel = getLayoutNode(snapshot, { role: "tabpanel" });
  const card = getLayoutNode(snapshot, {
    role: "group",
    name: "Fixture account card",
  });
  const description = getLayoutNode(snapshot, {
    text: "Update your public profile and contact details without compressing the panel into a narrow column.",
  });
  if (panel.rect.width < 450)
    throw new Error(`tab panel was compressed to ${panel.rect.width}px`);
  assertClose(panel.rect.width, list.rect.width, "tab panel width");
  assertClose(card.rect.width, panel.contentRect.width, "tab card width");
  if (!description.textMetrics || description.textMetrics.lineBox.width < 300)
    throw new Error(
      `tab description remained compressed: ${description.textMetrics?.lineBox.width ?? 0}px`,
    );
};

const assertAlertLayout = (snapshot: LayoutSnapshot) => {
  const alert = getLayoutNode(snapshot, {
    role: "alert",
    name: "Fixture failed native build",
  });
  const description = getLayoutNode(snapshot, {
    text: "The linker could not create the application bundle. Review the output before retrying the build.",
  });
  const actions = getLayoutNode(snapshot, {
    role: "group",
    name: "Fixture recovery actions",
  });
  assertLayoutRectContains(alert.contentRect, description.rect, {
    label: "alert description",
  });
  assertLayoutRectContains(alert.contentRect, actions.rect, {
    label: "alert recovery actions",
  });
  if (!description.textMetrics || description.textMetrics.lineBox.width < 260)
    throw new Error(
      `alert description was compressed to ${description.textMetrics?.lineBox.width ?? 0}px`,
    );
  for (const button of queryLayoutNodes(snapshot, { role: "button" }))
    assertLayoutRectContains(actions.contentRect, button.rect, {
      label: button.semantic?.label ?? "alert action",
    });
};

const assertToolLayout = (snapshot: LayoutSnapshot) => {
  const root = getLayoutNode(snapshot, { role: "group", name: "Tool fixture" });
  const trigger = getLayoutNode(snapshot, {
    role: "button",
    name: "mcp__workspace__read_repository_file: crates/wabou-runtime/src/gpui_projection_boundary.rs: Running",
  });
  const details = getLayoutNode(snapshot, {
    role: "region",
    name: "Tool details",
  });
  const summary = getLayoutNode(snapshot, {
    text: "crates/wabou-runtime/src/gpui_projection_boundary.rs",
  });
  const title = getLayoutNode(snapshot, {
    text: "mcp__workspace__read_repository_file",
  });
  assertLayoutRectContains(root.rect, trigger.rect, {
    label: "tool header",
  });
  assertLayoutRectContains(root.rect, details.rect, {
    label: "tool details",
  });
  assertLayoutRectContains(trigger.contentRect, summary.rect, {
    label: "tool summary",
  });
  assertLayoutRectContains(trigger.contentRect, title.rect, {
    label: "tool title",
  });
  if (trigger.rect.height < 40)
    throw new Error(`tool trigger is too short: ${trigger.rect.height}`);
};

const assertReasoningLayout = (snapshot: LayoutSnapshot) => {
  const root = getLayoutNode(snapshot, {
    role: "group",
    name: "Reasoning fixture",
  });
  const trigger = getLayoutNode(snapshot, {
    role: "button",
    name: "Reasoning",
  });
  const details = getLayoutNode(snapshot, {
    role: "region",
    name: "Reasoning details",
  });
  assertLayoutRectContains(root.rect, trigger.rect, {
    label: "reasoning trigger",
  });
  assertLayoutRectContains(root.rect, details.rect, {
    label: "reasoning details",
  });
  if (trigger.rect.height < 36)
    throw new Error(`reasoning trigger is too short: ${trigger.rect.height}`);
};

const assertPromptSuggestionLayout = (snapshot: LayoutSnapshot) => {
  const group = getLayoutNode(snapshot, {
    role: "group",
    name: "Prompt suggestion fixture",
  });
  const review = getLayoutNode(snapshot, {
    role: "button",
    name: "Review current changes",
  });
  const verify = getLayoutNode(snapshot, {
    role: "button",
    name: "Run project checks",
  });
  const plan = getLayoutNode(snapshot, {
    role: "button",
    name: "Plan a feature",
  });
  for (const suggestion of [review, verify, plan]) {
    assertLayoutRectContains(group.rect, suggestion.rect, {
      label: "prompt suggestion",
    });
    if (suggestion.rect.width < 176) {
      throw new Error(
        `prompt suggestion lost readable width: ${suggestion.rect.width}`,
      );
    }
  }
  if (Math.abs(review.rect.y - verify.rect.y) > 0.5) {
    throw new Error("prompt suggestions did not form a two-column first row");
  }
  if (plan.rect.y <= review.rect.y) {
    throw new Error("third prompt suggestion did not wrap to the next row");
  }
};

const assertSelectLayout = (snapshot: LayoutSnapshot) => {
  const trigger = getLayoutNode(snapshot, {
    role: "combobox",
    name: "Fixture select",
  });
  const listbox = getLayoutNode(snapshot, {
    role: "listbox",
    name: "Fixture select",
  });
  assertLayoutRectContains(
    {
      x: 0,
      y: 0,
      width: snapshot.status.viewportWidth,
      height: snapshot.status.viewportHeight,
    },
    listbox.rect,
    { label: "select popup" },
  );
  if (listbox.rect.y < layoutRectBottom(trigger.rect))
    throw new Error("select popup overlaps its trigger");
  const selectedOption = queryLayoutNodes(snapshot, { role: "option" }).find(
    (node) =>
      node.attrs.some(
        ([name, value]) => name === "aria-selected" && value === "true",
      ),
  );
  if (!selectedOption)
    throw new Error("select popup did not expose its selected option");
  if (!listbox.classes.includes("select-none"))
    throw new Error(
      "select popup exposed option labels to native text selection",
    );
  assertClose(trigger.rect.height, 32, "select trigger height");
  for (const option of queryLayoutNodes(snapshot, { role: "option" }))
    assertClose(option.rect.height, 32, "select option height");
};

const assertControlBaselineLayout = (snapshot: LayoutSnapshot) => {
  const small = getLayoutNode(snapshot, {
    role: "button",
    name: "Fixture small button",
  });
  const ordinary = getLayoutNode(snapshot, {
    role: "button",
    name: "Fixture default button",
  });
  const large = getLayoutNode(snapshot, {
    role: "button",
    name: "Fixture large button",
  });
  const input = getLayoutNode(snapshot, {
    role: "textbox",
    name: "Fixture baseline input",
  });
  const switchControl = getLayoutNode(snapshot, {
    role: "switch",
    name: "Fixture default switch",
  });
  const compactSwitch = getLayoutNode(snapshot, {
    role: "switch",
    name: "Fixture compact switch",
  });
  const buttonGroup = getLayoutNode(snapshot, {
    role: "group",
    name: "Fixture destructive button group",
  });
  const destructiveButton = getLayoutNode(snapshot, {
    role: "button",
    name: "Fixture destructive group action",
  });
  const heading = getLayoutNode(snapshot, {
    role: "heading",
    name: "Fixture section title",
  });
  const body = getLayoutNode(snapshot, {
    text: "Controls share one compact desktop rhythm without local offsets.",
  });

  assertClose(small.rect.height, 28, "small button height");
  assertClose(ordinary.rect.height, 32, "default button height");
  assertClose(large.rect.height, 40, "large button height");
  assertClose(input.rect.height, 32, "input height");
  assertClose(switchControl.rect.width, 40, "switch target width");
  assertClose(switchControl.rect.height, 24, "switch target height");
  assertClose(compactSwitch.rect.width, 40, "compact switch target width");
  assertClose(compactSwitch.rect.height, 24, "compact switch target height");
  assertLayoutRectContains(buttonGroup.contentRect, destructiveButton.rect, {
    label: "destructive button group edge",
  });
  assertLayoutTextStyle(heading, {
    fontSize: 24,
    fontWeight: 600,
  });
  assertLayoutTextStyle(body, {
    fontSize: 16,
    fontWeight: 400,
  });
};

const assertSelectionControlsLayout = (snapshot: LayoutSnapshot) => {
  const iconOnly = getLayoutNode(snapshot, {
    role: "checkbox",
    name: "Fixture icon-only checkbox",
  });
  const checkbox = getLayoutNode(snapshot, {
    role: "checkbox",
    name: "Keep completed downloads available after the application restarts",
  });
  const checkboxLabel = getLayoutNode(snapshot, {
    text: "Keep completed downloads available after the application restarts",
  });
  const radio = getLayoutNode(snapshot, {
    role: "radio",
    name: "Automatically choose the best download policy for this network",
  });
  const radioLabel = getLayoutNode(snapshot, {
    text: "Automatically choose the best download policy for this network",
  });
  const iconOnlyRadio = getLayoutNode(snapshot, {
    role: "radio",
    name: "Fixture icon-only radio",
  });

  assertClose(iconOnly.rect.width, 40, "icon-only checkbox target width");
  assertClose(iconOnly.rect.height, 28, "icon-only checkbox target height");
  assertClose(iconOnlyRadio.rect.width, 40, "icon-only radio target width");
  assertClose(iconOnlyRadio.rect.height, 28, "icon-only radio target height");
  assertLayoutRectContains(checkbox.contentRect, checkboxLabel.rect, {
    label: "wrapped checkbox label",
  });
  assertLayoutRectContains(radio.contentRect, radioLabel.rect, {
    label: "wrapped radio label",
  });
  if (checkboxLabel.rect.height < 40)
    throw new Error("long checkbox label did not wrap to two lines");
  if (radioLabel.rect.height < 40)
    throw new Error("long radio label did not wrap to two lines");
};

const assertEmptyLayout = (snapshot: LayoutSnapshot) => {
  const header = getLayoutNode(snapshot, { name: "Fixture empty header" });
  const description = getLayoutNode(snapshot, {
    text: "Try another search or create a project to continue working from this device.",
  });
  assertLayoutRectContains(header.rect, description.rect, {
    label: "empty-state description",
  });
  assertClose(description.rect.height, 40, "two-line empty-state summary");
};

const assertOnboardingLayout = (snapshot: LayoutSnapshot) => {
  const viewport = getLayoutNode(snapshot, {
    role: "region",
    name: "Fixture onboarding",
  });
  const header = getLayoutNode(snapshot, {
    name: "Fixture onboarding header",
  });
  const task = getLayoutNode(snapshot, {
    role: "group",
    name: "Fixture onboarding task",
  });
  const action = getLayoutNode(snapshot, {
    role: "button",
    name: "Continue",
  });
  assertLayoutRectContains(viewport.contentRect, header.rect, {
    label: "onboarding header",
  });
  assertLayoutRectContains(viewport.contentRect, task.rect, {
    label: "onboarding task",
  });
  assertLayoutRectContains(task.contentRect, action.rect, {
    label: "onboarding primary action",
  });
  if (viewport.computed.overflowY !== "Scroll")
    throw new Error("Onboarding did not retain its native scroll boundary");
};

const assertDialogLayout = (snapshot: LayoutSnapshot) => {
  const dialog = getLayoutNode(snapshot, {
    role: "dialog",
    name: "Fixture dialog",
  });
  const body = getLayoutNode(snapshot, { name: "Fixture dialog body" });
  const footer = getLayoutNode(snapshot, { name: "Fixture dialog footer" });
  assertLayoutRectContains(dialog.contentRect, body.rect, {
    label: "dialog scroll body",
  });
  assertLayoutRectContains(dialog.contentRect, footer.rect, {
    label: "dialog footer",
  });
  if (body.computed.overflowY !== "Scroll")
    throw new Error("dialog body did not establish an independent scroll area");
  if (layoutRectBottom(body.rect) > footer.rect.y)
    throw new Error("dialog body overlaps its fixed footer");
  assertClose(
    layoutRectBottom(footer.rect),
    layoutRectBottom(dialog.contentRect),
    "dialog footer bottom edge",
  );
};

const assertCardSurfaceLayout = (snapshot: LayoutSnapshot) => {
  const card = getLayoutNode(snapshot, {
    role: "group",
    name: "Fixture card surface",
  });
  const header = getLayoutNode(snapshot, { name: "Fixture card header" });
  const content = getLayoutNode(snapshot, { name: "Fixture card content" });
  const footer = getLayoutNode(snapshot, { name: "Fixture card footer" });
  for (const [label, node] of [
    ["card header", header],
    ["card content", content],
    ["card footer", footer],
  ] as const) {
    assertLayoutRectContains(card.contentRect, node.rect, { label });
  }
  if (layoutRectBottom(header.rect) > content.rect.y)
    throw new Error("card header overlaps its content");
  if (layoutRectBottom(content.rect) > footer.rect.y)
    throw new Error("card content overlaps its footer");
};

const assertDarkSurfaceLayout = (snapshot: LayoutSnapshot) => {
  const card = getLayoutNode(snapshot, {
    role: "group",
    name: "Fixture dark card",
  });
  const title = getLayoutNode(snapshot, {
    role: "heading",
    name: "Fixture dark title",
  });
  const input = getLayoutNode(snapshot, {
    role: "textbox",
    name: "Fixture dark input",
  });
  const item = getLayoutNode(snapshot, {
    role: "button",
    name: "Fixture dark item",
  });
  assertLayoutRectContains(card.contentRect, title.rect, {
    label: "dark card title",
  });
  if (title.computed.textColor === "#000000")
    throw new Error("dark surface resolved its primary foreground to black");
  if (new Map(item.attrs).get("aria-selected") !== "true")
    throw new Error("dark sidebar item lost its selected state");
  if (input.rect.width <= 0 || input.rect.height <= 0)
    throw new Error("dark input has empty native bounds");
};

const assertCompactSurfaceLayout = (snapshot: LayoutSnapshot) => {
  const card = getLayoutNode(snapshot, {
    role: "group",
    name: "Fixture compact card",
  });
  const input = getLayoutNode(snapshot, {
    role: "textbox",
    name: "Fixture compact input",
  });
  assertLayoutRectContains(card.contentRect, input.rect, {
    label: "compact input",
  });
  if (card.rect.width > snapshot.status.viewportWidth)
    throw new Error("compact card exceeds the narrow viewport");
};

const assertAdaptiveSplitPaneLayout = (snapshot: LayoutSnapshot) => {
  const boundary = getLayoutNode(snapshot, {
    name: "Adaptive split pane boundary",
  });
  const main = getLayoutNode(snapshot, { name: "Fixture split main" });
  const detail = getLayoutNode(snapshot, { name: "Fixture split detail" });
  assertLayoutRectContains(boundary.contentRect, main.rect, {
    label: "split pane main",
  });
  assertLayoutRectContains(boundary.contentRect, detail.rect, {
    label: "split pane detail",
  });
  assertClose(main.rect.y, detail.rect.y, "split pane top edge");
  assertClose(main.rect.height, detail.rect.height, "split pane height");
  assertClose(detail.rect.x - layoutRectRight(main.rect), 12, "split pane gap");
  assertClose(
    layoutRectRight(detail.rect),
    layoutRectRight(boundary.contentRect),
    "split pane right edge",
  );
};

const assertImageViewportLayout = (snapshot: LayoutSnapshot) => {
  const viewport = getLayoutNode(snapshot, { name: "Fixture image viewport" });
  const layer = getLayoutNode(snapshot, { name: "Fixture annotation layer" });
  const region = getLayoutNode(snapshot, { name: "Fixture speech region" });
  assertLayoutRectContains(viewport.contentRect, layer.rect, {
    label: "annotation layer",
  });
  assertLayoutRectContains(viewport.contentRect, region.rect, {
    label: "annotation region",
  });
  // The bordered 686×486 content box contains an 800×1200 image at 0.405 scale.
  assertClose(region.rect.width, 97.2, "projected annotation width");
  assertClose(region.rect.height, 72.9, "projected annotation height");
};

const assertImageListLayout = (snapshot: LayoutSnapshot) => {
  const list = getLayoutNode(snapshot, { name: "Fixture image list" });
  const first = getLayoutNode(snapshot, { name: "Fixture page 1" });
  assertLayoutRectContains(list.contentRect, first.rect, {
    label: "first image row",
  });
  assertClose(first.rect.height, 80, "virtual image row height");
};

const assertQrCodeLayout = (snapshot: LayoutSnapshot) => {
  const code = getLayoutNode(snapshot, { name: "Fixture QR code" });
  assertClose(code.rect.width, 196, "QR code width");
  assertClose(code.rect.height, 196, "QR code height");
  assertLayoutRectContains(
    {
      x: 0,
      y: 0,
      width: snapshot.status.viewportWidth,
      height: snapshot.status.viewportHeight,
    },
    code.rect,
    { label: "QR code" },
  );
};

const assertIconFrameLayout = (snapshot: LayoutSnapshot) => {
  const frame = getLayoutNode(snapshot, { name: "Fixture icon frame" });
  const icon = getLayoutNode(snapshot, { name: "Fixture framed icon" });
  assertClose(frame.rect.width, 48, "icon frame width");
  assertClose(frame.rect.height, 48, "icon frame height");
  assertClose(icon.rect.width, 23, "framed icon width");
  assertClose(icon.rect.height, 23, "framed icon height");
  assertClose(
    icon.rect.x + icon.rect.width / 2,
    frame.rect.x + frame.rect.width / 2,
    "framed icon horizontal center",
  );
  assertClose(
    icon.rect.y + icon.rect.height / 2,
    frame.rect.y + frame.rect.height / 2,
    "framed icon vertical center",
  );
};

const assertInputGroupLayout = (snapshot: LayoutSnapshot) => {
  const group = getLayoutNode(snapshot, { name: "Fixture input group" });
  const addon = getLayoutNode(snapshot, { name: "Fixture scheme addon" });
  const input = getLayoutNode(snapshot, { name: "Fixture hostname input" });
  const addonText = getLayoutNode(snapshot, { text: "https://" });
  assertClose(addon.rect.height, input.rect.height, "input group item height");
  assertLayoutRectContains(group.contentRect, addon.rect, {
    label: "input group addon",
  });
  assertLayoutRectContains(group.contentRect, input.rect, {
    label: "input group input",
  });
  if (!addonText.textMetrics || !input.textMetrics)
    throw new Error("InputGroup text metrics were not published");
  assertClose(
    addonText.textMetrics.baseline,
    input.textMetrics.baseline,
    "input group text baseline",
  );
};

const assertDirectoryPickerLayout = (snapshot: LayoutSnapshot) => {
  const input = getLayoutNode(snapshot, {
    role: "textbox",
    name: "Fixture directory",
  });
  const browse = getLayoutNode(snapshot, {
    role: "button",
    name: "Choose fixture directory",
  });
  if (
    !input.parentId ||
    !browse.parentId ||
    input.parentId.lo !== browse.parentId.lo ||
    input.parentId.hi !== browse.parentId.hi
  ) {
    throw new Error(
      "directory input and browse action do not share one surface",
    );
  }
  const group = snapshot.nodes.find(
    (node) =>
      node.id.lo === input.parentId?.lo && node.id.hi === input.parentId?.hi,
  );
  if (!group) throw new Error("directory picker surface is missing");
  assertLayoutRectContains(group.contentRect, input.rect, {
    label: "directory input",
  });
  assertLayoutRectContains(group.contentRect, browse.rect, {
    label: "directory browse action",
  });
  assertClose(group.rect.height, 32, "directory picker height");
};

const assertMessageLayout = (snapshot: LayoutSnapshot) => {
  const group = getLayoutNode(snapshot, { name: "Fixture message group" });
  const bubble = getLayoutNode(snapshot, {
    name: "Fixture failed message bubble",
  });
  const text = getLayoutNode(snapshot, {
    text: "Delivery failed. Retry from the action menu.",
  });
  const first = getLayoutNode(snapshot, { name: "Fixture failed message" });
  const following = getLayoutNode(snapshot, {
    name: "Fixture following message",
  });
  const actions = getLayoutNode(snapshot, { name: "Fixture message actions" });
  assertLayoutRectContains(group.contentRect, bubble.rect, {
    label: "message bubble",
  });
  assertLayoutRectContains(bubble.contentRect, text.rect, {
    label: "message text",
  });
  if (text.rect.width < 180)
    throw new Error(`message text was compressed to ${text.rect.width}px`);
  const gap = following.rect.y - layoutRectBottom(first.rect);
  if (Math.abs(gap - 12) > 0.5) {
    throw new Error(
      `hidden message actions changed transcript rhythm: gap=${gap}px`,
    );
  }
  if (actions.rect.y !== first.rect.y) {
    throw new Error(
      `message actions did not overlay their owner: actions y=${actions.rect.y}, message y=${first.rect.y}`,
    );
  }
};

const assertPiAgentToolbarLayout = (snapshot: LayoutSnapshot) => {
  const toolbar = getLayoutNode(snapshot, {
    name: "Pi agent toolbar fixture",
  });
  const controls = [
    ...queryLayoutNodes(snapshot, { role: "button" }),
    ...queryLayoutNodes(snapshot, { role: "combobox" }),
  ].filter(
    (node) =>
      node.rect.y >= toolbar.rect.y &&
      layoutRectBottom(node.rect) <= layoutRectBottom(toolbar.rect),
  );
  if (controls.length < 5)
    throw new Error(`Pi Agent toolbar lost controls; found ${controls.length}`);
  for (const control of controls)
    assertLayoutRectContains(toolbar.contentRect, control.rect, {
      label: control.semantic?.label ?? control.semantic?.role ?? control.tag,
    });
};

const assertPiAgentHeaderLayout = (snapshot: LayoutSnapshot) => {
  const sidebar = getLayoutNode(snapshot, {
    name: "Pi agent sidebar header",
  });
  const content = getLayoutNode(snapshot, {
    name: "Pi agent content header",
  });
  assertClose(sidebar.rect.y, content.rect.y, "Pi Agent header top edge");
  assertClose(sidebar.rect.height, 48, "Pi Agent sidebar header height");
  assertClose(content.rect.height, 48, "Pi Agent content header height");
  assertClose(
    layoutRectBottom(sidebar.rect),
    layoutRectBottom(content.rect),
    "Pi Agent header bottom edge",
  );
};

const assertMarkdownInlineLayout = (snapshot: LayoutSnapshot) => {
  const candidates = snapshot.nodes.filter((node) => node.text != null);
  const paragraph = candidates.find(
    (node) => node.text === "Before code after.",
  );
  if (!paragraph)
    throw new Error(
      `RichText text mismatch: ${JSON.stringify(candidates.map((node) => node.text))}`,
    );
  if (!paragraph.textMetrics)
    throw new Error("RichText did not publish native text metrics");
  if (paragraph.textMetrics.lineBox.height > 28)
    throw new Error(
      `RichText wrapped an inline paragraph: height=${paragraph.textMetrics.lineBox.height}px`,
    );
  if (paragraph.textMetrics.lineBox.width < 110)
    throw new Error(
      `RichText collapsed inline whitespace: width=${paragraph.textMetrics.lineBox.width}px`,
    );
};

const assertLabeledSeparatorLayout = (snapshot: LayoutSnapshot) => {
  const group = getLayoutNode(snapshot, {
    role: "group",
    name: "Labeled separator fixture",
  });
  const children = snapshot.nodes.filter(
    (node) =>
      node.parentId?.lo === group.id.lo && node.parentId.hi === group.id.hi,
  );
  const rules = children
    .filter((node) => node.rect.height <= 1.5)
    .sort((left, right) => left.rect.x - right.rect.x);
  if (rules.length !== 2)
    throw new Error(`labeled separator exposed ${rules.length} rules`);
  const [left, right] = rules;
  if (!left || !right || left.rect.width < 24 || right.rect.width < 24) {
    throw new Error("labeled separator collapsed a flexible rule");
  }
  assertClose(left.rect.width, right.rect.width, "separator rule symmetry");
  assertLayoutRectContains(group.contentRect, left.rect, {
    label: "left separator rule",
  });
  assertLayoutRectContains(group.contentRect, right.rect, {
    label: "right separator rule",
  });
};

const assertMarkdownConversationLayout = (snapshot: LayoutSnapshot) => {
  const heading = getLayoutNode(snapshot, { text: "Change" });
  const paragraph = getLayoutNode(snapshot, {
    text: "Updated the request path and kept healthz backward compatible.",
  });
  const codeBlock = getLayoutNode(snapshot, {
    role: "group",
    name: "Code block",
  });
  const copy = getLayoutNode(snapshot, {
    role: "button",
    name: "Copy code",
  });
  assertLayoutRectContains(codeBlock.rect, copy.rect, {
    label: "code copy action",
  });
  if (copy.text)
    throw new Error(`code copy action exposed visual text: ${copy.text}`);
  assertLayoutTextStyle(heading, {
    fontSize: 18,
    fontWeight: 600,
    label: "conversation heading",
  });
  assertLayoutTextStyle(paragraph, {
    fontSize: 16,
    fontWeight: 400,
    label: "conversation body",
  });
};

const overrides: Readonly<Record<string, Omit<LayoutFixtureCase, "id">>> = {
  "gallery/Overview": {
    // The complete overview is intentionally taller than the fixture viewport;
    // this contract targets its real GPUI SVG projection and paint path.
    checks: [],
    assert: assertOverviewSvgPaint,
  },
  // These two fixtures deliberately expose raw palette values rather than
  // product UI. Contrast is documented by the swatches themselves and must
  // not weaken the default quality contract for ordinary components.
  "foundations/Colors": {
    checks: ["visible-overflow", "sibling-collision"] as const,
  },
  "animation/Animation": {
    checks: ["visible-overflow", "sibling-collision"] as const,
  },
  // Carousel tracks and message reactions deliberately extend past their
  // logical content box; their component-specific clipping is tested lower.
  "widgets/Carousel": {
    checks: ["sibling-collision", "visual-quality"] as const,
  },
  "widgets/Message": {
    checks: ["sibling-collision", "text-collision", "visual-quality"] as const,
  },
  "component/Sidebar": { assert: assertSidebarLayout },
  "component/Tool": { assert: assertToolLayout },
  "component/Reasoning": { assert: assertReasoningLayout },
  "component/PromptSuggestion": { assert: assertPromptSuggestionLayout },
  "component/ScrollArea": { assert: assertScrollAreaLayout },
  "component/Select": { assert: assertSelectLayout },
  "component/ControlBaseline": { assert: assertControlBaselineLayout },
  "component/SelectionControls": { assert: assertSelectionControlsLayout },
  "component/Tabs": { assert: assertTabsLayout },
  "component/Alert": { assert: assertAlertLayout },
  "component/CardSurface": { assert: assertCardSurfaceLayout },
  "component/DarkSurface": { assert: assertDarkSurfaceLayout },
  "component/CompactSurface": { assert: assertCompactSurfaceLayout },
  "component/Slider": { assert: assertSliderLayout },
  "component/Empty": { assert: assertEmptyLayout },
  "component/Onboarding": { assert: assertOnboardingLayout },
  "component/Dialog": { assert: assertDialogLayout },
  "component/AdaptiveSplitPane": { assert: assertAdaptiveSplitPaneLayout },
  "component/ImageViewport": {
    width: 720,
    height: 520,
    // Resize hit slop intentionally extends beyond the 2px visual region.
    checks: ["sibling-collision", "visual-quality"] as const,
    assert: assertImageViewportLayout,
  },
  "image-viewport/ImageViewport": {
    checks: ["sibling-collision", "visual-quality"] as const,
  },
  "component/ImageList": {
    width: 360,
    height: 360,
    assert: assertImageListLayout,
  },
  "component/QRCode": { assert: assertQrCodeLayout },
  "component/IconFrame": { assert: assertIconFrameLayout },
  "component/InputGroup": { assert: assertInputGroupLayout },
  "component/DirectoryPicker": { assert: assertDirectoryPickerLayout },
  "component/MarkdownInline": { assert: assertMarkdownInlineLayout },
  "component/LabeledSeparator": { assert: assertLabeledSeparatorLayout },
  "component/MarkdownConversation": {
    assert: assertMarkdownConversationLayout,
  },
  "component/Message": { assert: assertMessageLayout },
  "component/PiAgentHeader": { assert: assertPiAgentHeaderLayout },
  "pi-agent/toolbar": { assert: assertPiAgentToolbarLayout },
  "primitive/Icon": { assert: assertIconLayout },
};
const fixtureCase = (id: string) => {
  const override = overrides[id];
  return { id, ...override, checks: override?.checks ?? checks };
};

const report = await renderLayoutFixtures({
  app: "apps/gallery",
  mode: "layout-test",
  command,
  skipBuild: process.env.WABOU_LAYOUT_SKIP_BUILD === "1",
  cases: selected.length === 0 ? "all" : selected.map(fixtureCase),
  checks: selected.length === 0 ? checks : undefined,
  overrides: selected.length === 0 ? overrides : undefined,
});

const effect = report.cases.find(({ id }) => id === "effect/synchronous");
if (effect)
  getLayoutNode(effect.snapshot, { name: "effect status", text: "ready" });

const narrow = report.cases.find(({ id }) => id === "narrow");
const wide = report.cases.find(({ id }) => id === "wide");
if (narrow && wide) {
  if (
    narrow.snapshot.status.viewportWidth !== 640 ||
    narrow.snapshot.status.viewportHeight !== 480 ||
    wide.snapshot.status.viewportWidth !== 960 ||
    wide.snapshot.status.viewportHeight !== 720
  )
    throw new Error("fixture-owned viewport metadata was not applied");
  if (getLayoutNode(narrow.snapshot, { name: "narrow" }).rect.width !== 120)
    throw new Error("narrow fixture width was not evaluated by Taffy");
  if (getLayoutNode(wide.snapshot, { name: "wide" }).rect.width !== 320)
    throw new Error("wide fixture width was not evaluated by Taffy");
  for (const fixture of [narrow, wide]) {
    const ownerCount = fixture.snapshot.nodes.find(
      (node) => node.text === "1" && node.tag === "text",
    );
    if (!ownerCount)
      throw new Error(
        `fixture \`${fixture.id}\` leaked its preceding Solid owner`,
      );
  }
}

console.log(
  `[wabou] validated ${report.cases.length} component layout fixtures in ${report.totalDurationMs.toFixed(1)}ms`,
);
