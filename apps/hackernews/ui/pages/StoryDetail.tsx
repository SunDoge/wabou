// Story detail page.
import { useNavigate, useParams } from "@wabou/router";
import { Button, Text } from "@wabou/primitives";
import ArrowLeft from "lucide-solid/icons/arrow-left";
import Bookmark from "lucide-solid/icons/bookmark";
import ExternalLink from "lucide-solid/icons/external-link";
import { createMemo, type JSX, Show } from "solid-js";
import { useTheme } from "../contexts/ThemeContext";
import {
  isSaved,
  relativeTime,
  savedStories,
  stories,
  storyHost,
  toggleSaved,
} from "../stories";

export function StoryDetail(): JSX.Element {
  const params = useParams();
  const navigate = useNavigate();
  const { palette } = useTheme();
  const story = createMemo(() =>
    [...stories(), ...savedStories()].find(
      (item) => item.id === Number(params.id),
    ),
  );
  const goBack = () => navigate(-1);

  return (
    <section
      class="h-full min-h-0 flex flex-col"
      style={{ "background-color": palette().background }}
    >
      <header
        class="h-14 flex-none px-6 flex items-center border-b"
        style={{ "border-color": palette().border }}
      >
        <Button
          unstyled
          variant="ghost"
          class="h-8 px-3 flex items-center gap-2 rounded-md text-xs"
          style={(state) => ({
            "background-color": state.hovered
              ? palette().hover
              : palette().surface,
            color: palette().textSecondary,
          })}
          onClick={goBack}
        >
          <ArrowLeft size={14} />
          Back
        </Button>
      </header>

      <Show when={story()} fallback={<MissingStory />}>
        {(current) => (
          <article class="flex-1 min-h-0 overflow-y-auto px-10 py-9">
            <div class="max-w-3xl mx-auto">
              <p
                class="m-0 mb-3 text-xs font-semibold"
                style={{ color: palette().accent }}
              >
                {storyHost(current().url)}
              </p>
              <h1
                class="m-0 text-3xl font-semibold leading-tight"
                style={{ color: palette().text }}
              >
                {current().title}
              </h1>
              <div
                class="mt-5 flex gap-3 text-sm"
                style={{ color: palette().textMuted }}
              >
                <Text>{current().score} points</Text>
                <span>·</span>
                <Text>{current().by}</Text>
                <span>·</span>
                <Text>{relativeTime(current().time)}</Text>
              </div>

              <div class="mt-9 flex gap-3">
                <Show when={current().url}>
                  <a
                    class="h-9 px-4 flex flex-none items-center gap-2 rounded-md text-sm font-medium whitespace-nowrap"
                    style={{
                      "background-color": palette().accent,
                      color: "#ffffff",
                    }}
                    href={current().url}
                  >
                    Open article <ExternalLink size={14} />
                  </a>
                </Show>
                <span
                  class="h-9 px-4 flex flex-none items-center rounded-md text-sm whitespace-nowrap"
                  style={{
                    "background-color": palette().surface,
                    color: palette().textSecondary,
                  }}
                >
                  <Text>{current().descendants ?? 0} comments</Text>
                </span>
                <Button
                  unstyled
                  variant="ghost"
                  selected={isSaved(current().id)}
                  class="h-9 px-3 flex flex-none items-center gap-2 rounded-md"
                  style={(state) => ({
                    "background-color": isSaved(current().id)
                      ? palette().accentSoft
                      : state.hovered
                        ? palette().hover
                        : palette().surface,
                    color: isSaved(current().id)
                      ? palette().accent
                      : palette().textSecondary,
                  })}
                  onClick={() => toggleSaved(current())}
                >
                  <Bookmark
                    size={14}
                    fill={isSaved(current().id) ? "currentColor" : "none"}
                  />
                  <span class="text-sm">
                    {isSaved(current().id) ? "Saved" : "Save"}
                  </span>
                </Button>
              </div>

              <div
                class="mt-12 pt-5 border-t text-xs"
                style={{
                  "border-color": palette().border,
                  color: palette().textMuted,
                }}
              >
                Deterministic local fixture rendered with Solid
              </div>
            </div>
          </article>
        )}
      </Show>
    </section>
  );
}

function MissingStory(): JSX.Element {
  const { palette } = useTheme();
  return (
    <div
      class="flex-1 flex flex-col items-center justify-center gap-2"
      style={{ color: palette().textMuted }}
    >
      <strong style={{ color: palette().text }}>Story not found</strong>
      <span class="text-sm">Return to the feed and choose another story.</span>
    </div>
  );
}
