// Hacker News navigation sidebar.

import {
  Button,
  createActive,
  createHover,
  Icon,
  Text,
  View,
} from "@wabou/primitives";
import { useNavigate } from "@wabou/router";
import award from "lucide-static/icons/award.svg?raw";
import bookmark from "lucide-static/icons/bookmark.svg?raw";
import clock from "lucide-static/icons/clock.svg?raw";
import moon from "lucide-static/icons/moon.svg?raw";
import newspaper from "lucide-static/icons/newspaper.svg?raw";
import refreshCw from "lucide-static/icons/refresh-cw.svg?raw";
import sun from "lucide-static/icons/sun.svg?raw";
import { type JSX, onSettled } from "solid-js";
import { useTheme } from "../contexts/ThemeContext";
import {
  activeView,
  type View as FeedView,
  loading,
  loadStories,
  savedStories,
  selectView,
  stories,
} from "../stories";

export function Sidebar(): JSX.Element {
  const navigate = useNavigate();
  const { theme, palette, toggleTheme } = useTheme();

  onSettled(() => {
    if (stories().length === 0) void loadStories("top");
  });

  const openView = (view: FeedView) => {
    void navigate({ to: "/" });
    void selectView(view);
  };

  return (
    <aside
      class="flex-none flex flex-col border-r"
      style={{
        width: "26%",
        "min-width": "256px",
        "max-width": "320px",
        "background-color": palette().surface,
        "border-color": palette().border,
      }}
    >
      <div
        class="h-16 px-5 flex items-center gap-3 border-b"
        style={{ "border-color": palette().borderSoft }}
      >
        <div
          class="w-8 h-8 rounded-sm flex items-center justify-center font-bold"
          style={{ "background-color": palette().accent, color: "#ffffff" }}
        >
          Y
        </div>
        <strong
          class="text-sm whitespace-nowrap"
          style={{ color: palette().text }}
        >
          Hacker News
        </strong>
      </div>

      <nav class="flex-1 px-3 py-4" aria-label="Story feeds">
        <p
          class="m-0 px-2 pb-2 text-xs font-semibold"
          style={{ color: palette().textMuted }}
        >
          Feeds
        </p>
        <SidebarNavItem
          active={activeView() === "top"}
          count={activeView() === "top" ? stories().length : undefined}
          icon={<Icon source={newspaper} size={17} />}
          label="Top"
          onActivate={() => openView("top")}
        />
        <SidebarNavItem
          active={activeView() === "new"}
          icon={<Icon source={clock} size={17} />}
          label="New"
          onActivate={() => openView("new")}
        />
        <SidebarNavItem
          active={activeView() === "best"}
          icon={<Icon source={award} size={17} />}
          label="Best"
          onActivate={() => openView("best")}
        />

        <p
          class="m-0 mt-5 px-2 pb-2 text-xs font-semibold"
          style={{ color: palette().textMuted }}
        >
          Library
        </p>
        <SidebarNavItem
          active={activeView() === "saved"}
          count={savedStories().length || undefined}
          icon={<Icon source={bookmark} size={17} />}
          label="Saved"
          onActivate={() => openView("saved")}
        />
      </nav>

      <div
        class="px-3 py-3 border-t"
        style={{ "border-color": palette().border }}
      >
        <SidebarAction
          icon={<Icon source={theme() === "light" ? moon : sun} size={16} />}
          label={theme() === "light" ? "Dark theme" : "Light theme"}
          onActivate={toggleTheme}
        />
        <SidebarAction
          disabled={loading()}
          icon={<Icon source={refreshCw} size={16} />}
          label={loading() ? "Refreshing..." : "Refresh"}
          onActivate={() => void loadStories(undefined, true)}
        />
      </div>
    </aside>
  );
}

interface SidebarItemProps {
  active?: boolean;
  count?: number;
  disabled?: boolean;
  icon: JSX.Element;
  label: string;
  onActivate: () => void;
}

function SidebarNavItem(props: SidebarItemProps): JSX.Element {
  const hover = createHover();
  const active = createActive();
  const { palette } = useTheme();

  return (
    <View
      role="link"
      aria-current={props.active ? "page" : undefined}
      tabIndex={0}
      class={`w-full h-10 px-2 flex items-center gap-3 rounded-md text-sm ${props.active ? "font-semibold" : ""}`}
      style={{
        "background-color": props.active
          ? palette().raised
          : active.active()
            ? palette().raised
            : hover.hovered()
              ? palette().hover
              : "transparent",
        color: props.active ? palette().text : palette().textSecondary,
      }}
      onPointerEnter={hover.bindings.onPointerEnter}
      onPointerLeave={() => {
        hover.bindings.onPointerLeave();
        active.bindings.onPointerLeave();
      }}
      onPointerDown={active.bindings.onPointerDown}
      onPointerUp={active.bindings.onPointerUp}
      onPointerCancel={active.bindings.onPointerCancel}
      onClick={props.onActivate}
      onKeyDown={(event: { key: string; preventDefault(): void }) => {
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        props.onActivate();
      }}
    >
      <SidebarItemContent {...props} />
    </View>
  );
}

function SidebarAction(props: SidebarItemProps): JSX.Element {
  const { palette } = useTheme();
  return (
    <Button
      unstyled
      variant="ghost"
      class={`w-full h-10 px-2 flex items-center gap-3 rounded-md text-sm ${props.disabled ? "opacity-50" : ""}`}
      style={(state) => ({
        "background-color": state.pressed
          ? palette().raised
          : state.hovered
            ? palette().hover
            : "transparent",
        color: palette().textSecondary,
      })}
      disabled={props.disabled}
      onClick={props.onActivate}
    >
      <SidebarItemContent {...props} />
    </Button>
  );
}

function SidebarItemContent(props: SidebarItemProps): JSX.Element {
  const { palette } = useTheme();
  return (
    <>
      <span
        class="pointer-events-none w-5 h-5 flex-none flex items-center justify-center"
        style={{ color: props.active ? palette().accent : palette().textMuted }}
      >
        {props.icon}
      </span>
      <Text class="pointer-events-none flex-none whitespace-nowrap">
        {props.label}
      </Text>
      {props.count !== undefined ? (
        <Text
          class="pointer-events-none ml-auto flex-none text-xs whitespace-nowrap"
          style={{ color: palette().textMuted }}
        >
          {props.count}
        </Text>
      ) : null}
    </>
  );
}
