import {
  Icon,
  PromptSuggestion,
  PromptSuggestions,
  Text,
  View,
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
    <View class="min-h-80 items-center justify-center gap-5 py-10">
      <View class="w-11 h-11 rounded-xl bg-selected flex items-center justify-center">
        <Icon source={bot} size={21} class="text-accent" />
      </View>
      <View class="items-center gap-1">
        <Text class="text-xl font-semibold text-primary">
          {i18n.message(m.empty_workspace_title, {
            workspace: workspaceName(props.workspace),
          })}
        </Text>
        <Text class="max-w-xl text-sm text-secondary text-center whitespace-normal">
          {i18n.message(m.empty_detail, {})}
        </Text>
      </View>
      <PromptSuggestions
        class="max-w-3xl"
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
    </View>
  );
}

function workspaceName(path: string): string {
  const normalized = path.replace(/[\\/]+$/, "");
  return normalized.split(/[\\/]/).at(-1) || path;
}
