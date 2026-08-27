import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DirectoryPicker,
  Icon,
  Text,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import play from "lucide-static/icons/play.svg?raw";
import { createSignal } from "solid-js";
import { i18n, m } from "./i18n";

export function WorkspaceSetup(props: {
  path: string;
  error?: string;
  updatePath: (path: string) => void;
  start: () => Promise<unknown>;
}) {
  const [starting, setStarting] = createSignal(false);
  const start = async () => {
    if (starting() || !props.path.trim()) return;
    setStarting(true);
    try {
      await props.start();
    } finally {
      setStarting(false);
    }
  };
  return (
    <View class="min-h-0 flex-1 p-6 flex items-center justify-center bg-canvas">
      <Card class="w-full max-w-xl">
        <CardHeader class="items-center text-center">
          <View class="w-12 h-12 rounded-xl bg-selected flex items-center justify-center">
            <Icon source={bot} size={24} class="text-accent" />
          </View>
          <CardTitle>{i18n.message(m.setup_title, {})}</CardTitle>
          <CardDescription>{i18n.message(m.setup_detail, {})}</CardDescription>
        </CardHeader>
        <CardContent class="gap-4">
          <DirectoryPicker
            aria-label={i18n.message(m.workspace, {})}
            value={props.path}
            onValueChange={props.updatePath}
            placeholder={i18n.message(m.choose_repository, {})}
            browseLabel={i18n.message(m.browse, {})}
            browseAriaLabel={i18n.message(m.choose_repository, {})}
          />
          {props.error ? (
            <Text role="alert" class="text-sm text-danger whitespace-normal">
              {props.error}
            </Text>
          ) : null}
          <Button
            class="w-full"
            disabled={!props.path.trim() || starting()}
            onClick={() => void start()}
          >
            <Icon source={play} size={15} />
            {starting()
              ? i18n.message(m.starting, {})
              : i18n.message(m.start_agent, {})}
          </Button>
          <Text class="text-xs text-muted text-center whitespace-normal">
            {i18n.message(m.setup_safety, {})}
          </Text>
        </CardContent>
      </Card>
    </View>
  );
}
