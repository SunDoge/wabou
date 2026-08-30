import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
  Icon,
  PromptSuggestion,
  PromptSuggestions,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import gitPullRequest from "lucide-static/icons/git-pull-request.svg?raw";
import lightbulb from "lucide-static/icons/lightbulb.svg?raw";
import testTube from "lucide-static/icons/test-tube-2.svg?raw";
import { i18n, m } from "./i18n";

export function ConversationWelcome(props: {
  workspace: string;
  choosePrompt: (prompt: string) => void;
}) {
  const prompts = () => [
    {
      icon: gitPullRequest,
      title: i18n.message(m.starter_review_title, {}),
      description: i18n.message(m.starter_review_detail, {}),
      prompt: i18n.message(m.starter_review_prompt, {}),
    },
    {
      icon: testTube,
      title: i18n.message(m.starter_verify_title, {}),
      description: i18n.message(m.starter_verify_detail, {}),
      prompt: i18n.message(m.starter_verify_prompt, {}),
    },
    {
      icon: lightbulb,
      title: i18n.message(m.starter_plan_title, {}),
      description: i18n.message(m.starter_plan_detail, {}),
      prompt: i18n.message(m.starter_plan_prompt, {}),
    },
  ];
  return (
    <Empty variant="plain" class="min-h-0 p-0 py-5 gap-4">
      <EmptyHeader class="max-w-xl gap-2">
        <EmptyMedia
          variant="icon"
          class="w-11 h-11 mb-1 rounded-xl bg-selected"
        >
          <Icon source={bot} size={21} class="text-accent" />
        </EmptyMedia>
        <EmptyTitle class="text-xl font-semibold">
          {i18n.message(m.empty_workspace_title, {
            workspace: workspaceName(props.workspace),
          })}
        </EmptyTitle>
        <EmptyDescription class="max-w-xl text-secondary">
          {i18n.message(m.empty_detail, {})}
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent class="max-w-3xl">
        <PromptSuggestions
          itemCount={prompts().length}
          role="group"
          aria-label={i18n.message(m.starter_prompts, {})}
        >
          {prompts().map((item) => (
            <PromptSuggestion
              icon={item.icon}
              title={item.title}
              description={item.description}
              onClick={() => props.choosePrompt(item.prompt)}
            />
          ))}
        </PromptSuggestions>
      </EmptyContent>
    </Empty>
  );
}

function workspaceName(path: string): string {
  const normalized = path.replace(/[\\/]+$/, "");
  return normalized.split(/[\\/]/).at(-1) || path;
}
