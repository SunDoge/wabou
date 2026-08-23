import {
  Badge,
  Button,
  Card,
  CardContent,
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
  Icon,
  ScrollArea,
  Text,
  useNavigate,
  View,
} from "@wabou/ui";
import clock from "lucide-static/icons/clock-3.svg?raw";
import fileImage from "lucide-static/icons/file-image.svg?raw";
import folderOpen from "lucide-static/icons/folder-open.svg?raw";
import images from "lucide-static/icons/images.svg?raw";
import { For, Show } from "solid-js";
import { useMangaSession } from "./session";

export function Starter() {
  const session = useMangaSession();
  const navigate = useNavigate();
  const openFiles = async () => {
    await session.openFiles();
    if (session.pages().length) await navigate({ to: "/reader" });
  };
  const openFolder = async () => {
    await session.openFolder();
    if (session.pages().length) await navigate({ to: "/reader" });
  };

  return (
    <ScrollArea class="w-full h-full">
      <View class="w-full min-h-full p-8 items-center">
        <View class="w-full max-w-5xl flex flex-col gap-6">
          <View class="flex flex-col gap-2">
            <Text class="text-3xl font-semibold">Start a manga workspace</Text>
            <Text class="text-muted">
              Open loose pages or a directory. OCR results and edits remain
              available while you move between pages.
            </Text>
          </View>
          <View class="grid grid-cols-2 gap-4">
            <Card
              role="button"
              aria-label="Open manga pages"
              onClick={() => void openFiles()}
            >
              <CardContent class="p-6 flex flex-row items-center gap-4">
                <View class="w-12 h-12 flex-none rounded-lg bg-selected items-center justify-center">
                  <Icon source={fileImage} size={23} class="text-accent" />
                </View>
                <View class="min-w-0 flex flex-col gap-1">
                  <Text class="font-semibold">Open pages</Text>
                  <Text class="text-sm text-muted">
                    Choose one or several image files.
                  </Text>
                </View>
              </CardContent>
            </Card>
            <Card
              role="button"
              aria-label="Open manga directory"
              onClick={() => void openFolder()}
            >
              <CardContent class="p-6 flex flex-row items-center gap-4">
                <View class="w-12 h-12 flex-none rounded-lg bg-selected items-center justify-center">
                  <Icon source={folderOpen} size={23} class="text-accent" />
                </View>
                <View class="min-w-0 flex flex-col gap-1">
                  <Text class="font-semibold">Open directory</Text>
                  <Text class="text-sm text-muted">
                    Load a naturally sorted chapter.
                  </Text>
                </View>
              </CardContent>
            </Card>
          </View>
          <View class="flex flex-row items-center gap-2">
            <Icon source={clock} size={17} />
            <Text class="text-lg font-semibold">Recent</Text>
            <View class="flex-1" />
            <Badge variant={session.modelInstalled() ? "success" : "secondary"}>
              {session.modelInstalled()
                ? "OCR model ready"
                : "OCR model required"}
            </Badge>
          </View>
          <Show
            when={session.recentEntries().length > 0}
            fallback={
              <Empty class="min-h-64">
                <EmptyHeader>
                  <EmptyMedia variant="icon">
                    <Icon source={images} size={20} />
                  </EmptyMedia>
                  <EmptyTitle>No recent manga</EmptyTitle>
                  <EmptyDescription>
                    Your opened files and directories will appear here.
                  </EmptyDescription>
                </EmptyHeader>
                <EmptyContent>
                  <Button onClick={() => void openFiles()}>
                    Open your first page
                  </Button>
                </EmptyContent>
              </Empty>
            }
          >
            <View class="grid grid-cols-2 gap-3">
              <For each={session.recentEntries()}>
                {(entry) => (
                  <Button
                    variant="outline"
                    class="h-16 min-w-0 justify-start px-4"
                    onClick={async () => {
                      await session.openRecent(entry);
                      if (session.pages().length)
                        await navigate({ to: "/reader" });
                    }}
                  >
                    <Icon
                      source={
                        entry.kind === "directory" ? folderOpen : fileImage
                      }
                      size={18}
                    />
                    <View class="min-w-0 flex flex-col items-start">
                      <Text maxLines={1} class="w-full text-sm font-medium">
                        {entry.label}
                      </Text>
                      <Text maxLines={1} class="w-full text-xs text-muted">
                        {entry.path.replaceAll("\n", ", ")}
                      </Text>
                    </View>
                  </Button>
                )}
              </For>
            </View>
          </Show>
        </View>
      </View>
    </ScrollArea>
  );
}
