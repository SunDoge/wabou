import {
  AdaptiveSplitPane,
  AdaptiveSplitPaneDetail,
  AdaptiveSplitPaneMain,
  Badge,
  Button,
  ContentState,
  createLatestAsyncResource,
  Icon,
  Listbox,
  Markdown,
  PageHeader,
  PageViewport,
  ScrollArea,
  SearchField,
  Text,
  View,
} from "@wabou/ui";
import refreshCw from "lucide-static/icons/refresh-cw.svg?raw";
import sparkles from "lucide-static/icons/sparkles.svg?raw";
import triangleAlert from "lucide-static/icons/triangle-alert.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  Match,
  Show,
  Switch,
} from "solid-js";
import type { PiSkill } from "./api";
import { i18n, m } from "./i18n";

export interface SkillsPageProps {
  cwd: string;
  project: string;
  load(cwd: string): Promise<readonly PiSkill[]>;
  close(): void;
}

const skillSearchText = (skill: PiSkill) =>
  `${skill.name}\n${skill.description}\n${skill.path}`.toLocaleLowerCase();

export function SkillsPage(props: SkillsPageProps) {
  const [query, setQuery] = createSignal("");
  const [selectedId, setSelectedId] = createSignal("");
  const [compact, setCompact] = createSignal<boolean>();
  const skills = createLatestAsyncResource<string, readonly PiSkill[]>({
    source: () => props.cwd.trim() || undefined,
    load: (cwd) => props.load(cwd),
  });
  const filtered = createMemo(() => {
    const needle = query().trim().toLocaleLowerCase();
    const available = skills.value() ?? [];
    return needle
      ? available.filter((skill) => skillSearchText(skill).includes(needle))
      : available;
  });
  const selected = createMemo(() => {
    const available = filtered();
    return available.find((skill) => skill.id === selectedId());
  });
  const skillOptions = createMemo(() =>
    filtered().map((skill) => ({
      value: skill.id,
      label: skill.name,
      description: skill.description,
    })),
  );
  createEffect(
    () =>
      [
        filtered(),
        compact() !== undefined,
        compact() ?? false,
        selectedId(),
      ] as const,
    ([available, measured, isCompact, current]) => {
      if (available.some((skill) => skill.id === current)) return;
      setSelectedId(measured && !isCompact ? (available[0]?.id ?? "") : "");
    },
  );

  return (
    <PageViewport class="bg-canvas" contentClass="min-h-full p-8">
      <View class="w-full max-w-6xl mx-auto flex flex-col gap-6 pb-8">
        <PageHeader
          title={i18n.message(m.skills, {})}
          description={i18n.message(m.skills_intro, { project: props.project })}
          actions={
            <View class="flex flex-row items-center gap-2">
              <Button
                variant="outline"
                aria-label={i18n.message(m.refresh_skills, {})}
                onClick={() => void skills.refresh()}
              >
                <Icon source={refreshCw} size={14} />
                {i18n.message(m.refresh, {})}
              </Button>
              <Button variant="outline" onClick={props.close}>
                {i18n.message(m.back_to_agents, {})}
              </Button>
            </View>
          }
        />

        <View
          role="group"
          aria-label={i18n.message(m.skills, {})}
          class="min-w-0 min-h-96 h-2/3 overflow-hidden rounded-xl border border-subtle bg-surface shadow-xs"
        >
          <Show
            when={!skills.error()}
            fallback={
              <ContentState
                state="error"
                title={i18n.message(m.skills_load_failed, {})}
                description={String(skills.error())}
                aria-label={i18n.message(m.skills_load_failed, {})}
                class="h-full min-h-80 p-8"
                renderMedia={() => <Icon source={triangleAlert} size={18} />}
                renderAction={() => (
                  <Button
                    variant="outline"
                    aria-label={i18n.message(m.retry, {})}
                    onClick={() => void skills.refresh()}
                  >
                    <Icon source={refreshCw} size={14} />
                    {i18n.message(m.retry, {})}
                  </Button>
                )}
              />
            }
          >
            <AdaptiveSplitPane
              aria-label="Skill browser layout"
              compactAt={720}
              onCompactChange={setCompact}
              class="h-full"
            >
              <AdaptiveSplitPaneMain class="w-72 h-full flex-none border-r border-subtle bg-surface-muted">
                <View class="w-full h-full min-w-0 flex flex-col">
                  <View class="flex-none p-3 border-b border-subtle">
                    <SearchField
                      value={query()}
                      onValueChange={setQuery}
                      aria-label={i18n.message(m.search_skills, {})}
                      placeholder={i18n.message(m.search_skills, {})}
                    />
                  </View>
                  <View class="min-h-0 flex-1 p-2 flex flex-col">
                    <Switch>
                      <Match
                        when={
                          skills.loading() &&
                          (skills.value()?.length ?? 0) === 0
                        }
                      >
                        <ContentState
                          state="loading"
                          title={i18n.message(m.loading_skills, {})}
                        />
                      </Match>
                      <Match when={filtered().length === 0}>
                        <ContentState
                          state="empty"
                          title={i18n.message(
                            (skills.value()?.length ?? 0) === 0
                              ? m.no_skills
                              : m.no_skills_found,
                            {},
                          )}
                          renderMedia={() => (
                            <Icon
                              source={sparkles}
                              size={20}
                              class="text-secondary"
                            />
                          )}
                        />
                      </Match>
                      <Match when={true}>
                        <Listbox
                          fill
                          aria-label={i18n.message(m.skills, {})}
                          options={skillOptions()}
                          value={selectedId() || undefined}
                          itemHeight={56}
                          itemClass="px-2"
                          onValueChange={setSelectedId}
                          renderTrailing={(option) => {
                            const skill = filtered().find(
                              (candidate) => candidate.id === option.value,
                            );
                            return skill === undefined ? null : (
                              <Badge variant="outline" weight="normal">
                                {i18n.message(
                                  skill.scope === "project"
                                    ? m.skill_scope_project
                                    : m.skill_scope_user,
                                  {},
                                )}
                              </Badge>
                            );
                          }}
                        />
                      </Match>
                    </Switch>
                  </View>
                </View>
              </AdaptiveSplitPaneMain>

              <AdaptiveSplitPaneDetail
                open={selected() !== undefined}
                onOpenChange={(open) => {
                  if (!open) setSelectedId("");
                }}
                aria-label={
                  selected()?.name ?? i18n.message(m.select_skill, {})
                }
                class="h-full flex-1"
                modalClass="w-11/12 max-w-3xl"
              >
                <View class="w-full h-full min-w-0 flex flex-col bg-surface">
                  <Show
                    when={selected()}
                    fallback={
                      <ContentState
                        state="empty"
                        title={i18n.message(m.select_skill, {})}
                        class="min-h-80 p-8"
                        renderMedia={() => <Icon source={sparkles} size={18} />}
                      />
                    }
                  >
                    {(skill) => (
                      <>
                        <View class="flex-none px-6 py-5 flex flex-col gap-2 border-b border-subtle">
                          <View class="min-w-0 flex flex-row items-center gap-2">
                            <Text
                              role="heading"
                              class="min-w-0 truncate text-xl font-semibold text-primary"
                            >
                              {skill().name}
                            </Text>
                            <Badge variant="secondary">
                              {i18n.message(
                                skill().scope === "project"
                                  ? m.skill_scope_project
                                  : m.skill_scope_user,
                                {},
                              )}
                            </Badge>
                          </View>
                          <Show when={skill().description}>
                            <Text class="text-sm text-secondary whitespace-normal">
                              {skill().description}
                            </Text>
                          </Show>
                          <Text class="w-full truncate text-xs text-secondary">
                            {skill().path}
                          </Text>
                        </View>
                        <ScrollArea
                          class="min-w-0 min-h-0 flex-1"
                          contentClass="p-6"
                        >
                          <Markdown
                            source={skill().content}
                            aria-label={skill().name}
                            class="max-w-3xl"
                          />
                        </ScrollArea>
                      </>
                    )}
                  </Show>
                </View>
              </AdaptiveSplitPaneDetail>
            </AdaptiveSplitPane>
          </Show>
        </View>
      </View>
    </PageViewport>
  );
}
