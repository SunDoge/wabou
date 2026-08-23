import {
  Button,
  ColorThemeProvider,
  ComponentsProvider,
  Icon,
  IconFrame,
  Text,
  useNavigate,
  useRouteActive,
  View,
} from "@wabou/ui";
import bookOpen from "lucide-static/icons/book-open.svg?raw";
import image from "lucide-static/icons/image.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import type { JSX } from "solid-js";
import { useMangaSession } from "./session";

function NavButton(props: { to: string; label: string; icon: string }) {
  const navigate = useNavigate();
  const active = useRouteActive(props.to);
  return (
    <Button
      size="sm"
      variant="ghost"
      selected={active()}
      class={active() ? "bg-selected" : undefined}
      onClick={() => void navigate({ to: props.to })}
    >
      <Icon source={props.icon} size={15} />
      {props.label}
    </Button>
  );
}

export function AppShell(props: { children?: JSX.Element }) {
  const session = useMangaSession();
  return (
    <ColorThemeProvider
      theme={session.theme()}
      transition={{ duration: 0.18, easing: "ease-out" }}
    >
      <ComponentsProvider theme={session.theme()}>
        <View class="w-full h-full min-w-0 min-h-0 flex flex-col bg-canvas text-primary">
          <View class="h-14 flex-none px-5 flex flex-row items-center gap-3 border-b border-subtle bg-surface shadow-sm">
            <IconFrame
              source={image}
              size="sm"
              iconSize={18}
              variant="solid"
            />
            <View class="flex flex-col mr-3">
              <Text class="font-semibold">Manga OCR</Text>
              <Text class="text-xs text-muted">
                Native OCR and translation workspace
              </Text>
            </View>
            <NavButton to="/" label="Start" icon={bookOpen} />
            <NavButton to="/reader" label="Reader" icon={image} />
            <View class="flex-1" />
            <NavButton to="/settings" label="Settings" icon={settings} />
            <NavButton to="/about" label="About" icon={info} />
          </View>
          <View class="flex-1 min-h-0 min-w-0">{props.children}</View>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}
