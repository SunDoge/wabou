// Story list page.

import { Button, createHover, Icon, Text, TextInput } from "@wabou/primitives";
import { useNavigate } from "@wabou/router";
import bookmark from "lucide-static/icons/bookmark.svg?raw";
import messageSquare from "lucide-static/icons/message-square.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { createMemo, For, type JSX, Show } from "solid-js";
import { LoadingList } from "../components/LoadingList";
import { useTheme } from "../contexts/ThemeContext";
import {
  activeView,
  isSaved,
  isVisited,
  loadError,
  loading,
  loadStories,
  markVisited,
  query,
  relativeTime,
  type Story,
  setQuery,
  storyHost,
  toggleSaved,
  viewLabels,
  visibleSource,
} from "../stories";

export function StoryList(): JSX.Element {
  const navigate = useNavigate();
  const { palette } = useTheme();
  const visibleStories = createMemo(() => {
    const needle = query().trim().toLowerCase();
    if (!needle) return visibleSource();
    return visibleSource().filter(
      (story) =>
        story.title.toLowerCase().includes(needle) ||
        story.by.toLowerCase().includes(needle) ||
        storyHost(story.url).includes(needle),
    );
  });

  const openStory = (story: Story) => {
    markVisited(story.id);
    void navigate({ to: `/story/${story.id}` });
  };

  return (
    <section class="h-full min-h-0 flex flex-col">
      <header
        class="h-16 flex-none px-6 flex items-center gap-5 border-b"
        style={{
          "background-color": palette().raised,
          "border-color": palette().border,
        }}
      >
        <div class="min-w-0 flex-1">
          <h1 class="m-0 text-xl font-semibold leading-tight">
            {viewLabels[activeView()]}
          </h1>
          <p class="m-0 text-xs" style={{ color: palette().textMuted }}>
            {activeView() === "saved"
              ? "Stories saved during this session"
              : "Live from the Hacker News API"}
          </p>
        </div>
        <label
          class="w-60 h-9 px-3 flex items-center gap-2 border rounded-md"
          style={{
            "background-color": palette().background,
            "border-color": palette().accent,
            color: palette().textMuted,
          }}
        >
          <Icon source={search} size={15} />
          <TextInput
            class="w-full min-w-0 border-0 bg-transparent text-sm"
            style={{ color: palette().text }}
            value={query()}
            placeholder="Search"
            aria-label="Filter stories"
            onInput={(event) => setQuery(event.currentTarget.value)}
          />
          <Show when={query()}>
            <Icon source={x} size={14} onClick={() => setQuery("")} />
          </Show>
        </label>
      </header>

      <div class="flex-1 min-h-0 overflow-y-auto overflow-x-hidden px-4 py-2">
        <Show
          when={!loading() || activeView() === "saved"}
          fallback={<LoadingList />}
        >
          <Show when={!loadError()} fallback={<LoadError />}>
            <Show when={visibleStories().length > 0} fallback={<EmptyState />}>
              <ol class="m-0 p-0">
                <For each={visibleStories()}>
                  {(story, index) => {
                    const { hovered, bindings } = createHover();
                    return (
                      // biome-ignore lint/a11y/useSemanticElements: wabou currently mislays out flex button elements.
                      <div
                        {...bindings}
                        class="min-h-20 px-3 py-3 flex items-center gap-3 border-b"
                        style={{
                          "background-color": hovered()
                            ? palette().hover
                            : "transparent",
                          "border-color": palette().borderSoft,
                          color: isVisited(story.id)
                            ? palette().textMuted
                            : palette().text,
                        }}
                        onClick={() => openStory(story)}
                        onKeyDown={(event) => {
                          if (event.key === "Enter" || event.key === " ") {
                            openStory(story);
                          }
                        }}
                        role="button"
                        tabIndex={0}
                      >
                        <span
                          class="w-7 flex-none text-right text-xs"
                          style={{ color: palette().textMuted }}
                        >
                          {index() + 1}
                        </span>
                        <div class="flex-1 min-w-0">
                          <div class="leading-tight">
                            <strong class="text-sm font-medium">
                              {story.title}
                            </strong>
                          </div>
                          <div
                            class="mt-2 flex items-center gap-2 text-xs overflow-hidden whitespace-nowrap"
                            style={{ color: palette().textMuted }}
                          >
                            <Show when={story.url}>
                              <span
                                class="max-w-44 overflow-hidden whitespace-nowrap"
                                style={{ color: palette().textSecondary }}
                              >
                                {storyHost(story.url)}
                              </span>
                              <span>·</span>
                            </Show>
                            <Text>{story.score} points</Text>
                            <span>·</span>
                            <Text>{story.by}</Text>
                            <span>·</span>
                            <Text>{relativeTime(story.time)}</Text>
                          </div>
                        </div>
                        <span
                          class="w-12 flex-none flex items-center justify-end gap-1 text-xs"
                          style={{ color: palette().textMuted }}
                        >
                          <Icon source={messageSquare} size={13} />
                          {story.descendants ?? 0}
                        </span>
                        <BookmarkAction story={story} />
                      </div>
                    );
                  }}
                </For>
              </ol>
            </Show>
          </Show>
        </Show>
      </div>

      <footer
        class="h-9 flex-none px-6 flex items-center border-t text-xs"
        style={{
          "background-color": palette().raised,
          "border-color": palette().border,
          color: palette().textMuted,
        }}
      >
        <Text>{visibleStories().length} stories</Text>
      </footer>
    </section>
  );
}

function BookmarkAction(props: { story: Story }): JSX.Element {
  const { palette } = useTheme();
  const toggle = (event: MouseEvent | KeyboardEvent) => {
    event.stopPropagation();
    toggleSaved(props.story);
  };

  return (
    <Button
      unstyled
      variant="ghost"
      class="w-5 h-5 flex-none flex items-center justify-center"
      style={(state) => ({
        "min-height": "20px",
        padding: 0,
        "border-width": 0,
        "background-color": state.hovered ? palette().hover : "transparent",
        color: isSaved(props.story.id) ? palette().accent : palette().textMuted,
      })}
      onClick={(event) => toggle(event as MouseEvent)}
      aria-label={isSaved(props.story.id) ? "Remove saved story" : "Save story"}
    >
      <Icon
        source={bookmark}
        size={16}
        fill={isSaved(props.story.id) ? "currentColor" : "none"}
      />
    </Button>
  );
}

function LoadError(): JSX.Element {
  const { palette } = useTheme();
  return (
    <div
      class="min-h-80 flex flex-col items-center justify-center gap-3 text-center"
      style={{ color: palette().textMuted }}
    >
      <strong style={{ color: palette().text }}>Could not load stories</strong>
      <span class="text-sm" style={{ color: palette().danger }}>
        {loadError()}
      </span>
      <Button
        unstyled
        variant="ghost"
        class="text-sm"
        style={(state) => ({
          "border-width": 0,
          "background-color": state.hovered ? palette().hover : "transparent",
          color: palette().accent,
        })}
        onClick={() => void loadStories(undefined, true)}
      >
        Try again
      </Button>
    </div>
  );
}

function EmptyState(): JSX.Element {
  const { palette } = useTheme();
  return (
    <div
      class="min-h-80 flex flex-col items-center justify-center gap-2 text-center"
      style={{ color: palette().textMuted }}
    >
      <Icon source={bookmark} size={22} />
      <strong style={{ color: palette().text }}>
        {activeView() === "saved" ? "No saved stories" : "No matching stories"}
      </strong>
      <span class="text-sm">
        {activeView() === "saved"
          ? "Use the bookmark icon to keep a story here."
          : "Try another search."}
      </span>
    </div>
  );
}
