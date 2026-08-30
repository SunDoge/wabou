import {
  Badge,
  Button,
  createLatestAsyncResource,
  Icon,
  Markdown,
  PageHeader,
  PageViewport,
  SearchField,
  Text,
  View,
} from "@wabou/ui";
import refreshCw from "lucide-static/icons/refresh-cw.svg?raw";
import sparkles from "lucide-static/icons/sparkles.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  For as ForValue,
  Show,
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
    return available.find((skill) => skill.id === selectedId()) ?? available[0];
  });
  createEffect(
    () => selected()?.id ?? "",
    (id) => {
      setSelectedId(id);
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

        <View class="min-w-0 min-h-96 flex flex-row items-stretch overflow-hidden rounded-xl border border-subtle bg-surface shadow-xs">
          <View class="w-72 flex-none min-w-0 flex flex-col border-r border-subtle bg-surface-muted">
            <View class="flex-none p-3 border-b border-subtle">
              <SearchField
                value={query()}
                onValueChange={setQuery}
                aria-label={i18n.message(m.search_skills, {})}
                placeholder={i18n.message(m.search_skills, {})}
              />
            </View>
            <View class="min-h-0 flex-1 overflow-y-auto p-2 gap-1">
              <Show
                when={!skills.loading() || (skills.value()?.length ?? 0) > 0}
                fallback={
                  <Text role="status" class="p-3 text-sm text-muted">
                    {i18n.message(m.loading_skills, {})}
                  </Text>
                }
              >
                <ForValue
                  each={filtered()}
                  fallback={
                    <View class="p-4 flex flex-col items-center gap-2">
                      <Icon source={sparkles} size={20} class="text-muted" />
                      <Text role="status" class="text-sm text-muted">
                        {i18n.message(
                          (skills.value()?.length ?? 0) === 0
                            ? m.no_skills
                            : m.no_skills_found,
                          {},
                        )}
                      </Text>
                    </View>
                  }
                >
                  {(skill) => (
                    <Button
                      variant={
                        selected()?.id === skill.id ? "secondary" : "ghost"
                      }
                      class="w-full h-auto min-w-0 px-3 py-2.5 items-start justify-start"
                      aria-label={skill.name}
                      aria-selected={selected()?.id === skill.id}
                      onClick={() => setSelectedId(skill.id)}
                    >
                      <View class="min-w-0 flex-1 flex flex-col items-start gap-1">
                        <Text class="w-full truncate text-sm font-medium text-primary">
                          {skill.name}
                        </Text>
                        <Show when={skill.description}>
                          <Text class="w-full truncate text-xs text-secondary">
                            {skill.description}
                          </Text>
                        </Show>
                        <View class="flex flex-row items-center gap-1.5 pt-0.5">
                          <Badge variant="outline" weight="normal">
                            {i18n.message(
                              skill.scope === "project"
                                ? m.skill_scope_project
                                : m.skill_scope_user,
                              {},
                            )}
                          </Badge>
                          <Badge variant="ghost" weight="normal">
                            {skill.source === "pi" ? "Pi" : "Agents"}
                          </Badge>
                        </View>
                      </View>
                    </Button>
                  )}
                </ForValue>
              </Show>
            </View>
          </View>

          <View class="min-w-0 flex-1 flex flex-col">
            <Show
              when={!skills.error()}
              fallback={
                <View class="p-6 flex flex-col gap-2">
                  <Text
                    role="heading"
                    class="font-semibold text-danger-primary"
                  >
                    {i18n.message(m.skills_load_failed, {})}
                  </Text>
                  <Text
                    role="alert"
                    class="text-sm text-muted whitespace-normal"
                  >
                    {String(skills.error())}
                  </Text>
                </View>
              }
            >
              <Show
                when={selected()}
                fallback={
                  <View class="min-h-80 p-8 flex flex-col items-center justify-center gap-3">
                    <Icon source={sparkles} size={28} class="text-muted" />
                    <Text class="text-sm text-muted">
                      {i18n.message(m.select_skill, {})}
                    </Text>
                  </View>
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
                    <View class="min-w-0 flex-1 p-6">
                      <Markdown
                        source={skill().content}
                        aria-label={skill().name}
                        class="max-w-3xl"
                      />
                    </View>
                  </>
                )}
              </Show>
            </Show>
          </View>
        </View>
      </View>
    </PageViewport>
  );
}
