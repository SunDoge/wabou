import {
  Button,
  Icon,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuIcon,
  SidebarMenuLabel,
  SidebarMenuSuffix,
  SidebarSearch,
  Text,
  useLocation,
  useNavigate,
  View,
  Workbench,
  WorkbenchMain,
  WorkbenchSidebar,
} from "@wabou/ui";
import bookOpen from "lucide-static/icons/book-open.svg?raw";
import fileText from "lucide-static/icons/file-text.svg?raw";
import library from "lucide-static/icons/library.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import type { JSX } from "solid-js";
import { For as ForValue, Show } from "solid-js";
import { ReaderProvider, useReader } from "./reader-state";

function Shell(props: { children?: JSX.Element }) {
  const reader = useReader();
  const navigate = useNavigate();
  const location = useLocation();
  const open = async () => {
    if (await reader.openPdf()) navigate({ to: "/" });
  };
  return (
    <Workbench aria-label="Wabou Reader workspace">
      <WorkbenchSidebar class="w-64">
        <SidebarHeader class="px-3">
          <View class="w-7 h-7 rounded-md bg-accent flex items-center justify-center">
            <Icon source={bookOpen} size={15} class="text-on-accent" />
          </View>
          <Text class="font-semibold tracking-tight">Reader</Text>
          <View class="flex-1" />
          <Button
            variant="ghost"
            size="icon"
            aria-label="Open PDF"
            disabled={reader.opening()}
            onClick={() => void open()}
          >
            <Icon source={plus} size={15} />
          </Button>
        </SidebarHeader>
        <SidebarSearch
          variant="quiet"
          value={reader.query()}
          placeholder="Find in library"
          aria-label="Find in library"
          onValueChange={reader.setQuery}
        />
        <SidebarContent contentClass="px-2 py-2">
          <SidebarGroup>
            <SidebarGroupLabel>Workspace</SidebarGroupLabel>
            <SidebarMenu
              value={location().pathname === "/library" ? "library" : "read"}
            >
              <SidebarMenuButton
                value="read"
                onClick={() => navigate({ to: "/" })}
              >
                <SidebarMenuIcon>
                  <Icon source={bookOpen} size={14} />
                </SidebarMenuIcon>
                <SidebarMenuLabel>Reading desk</SidebarMenuLabel>
              </SidebarMenuButton>
              <SidebarMenuButton
                value="library"
                onClick={() => navigate({ to: "/library" })}
              >
                <SidebarMenuIcon>
                  <Icon source={library} size={14} />
                </SidebarMenuIcon>
                <SidebarMenuLabel>Library</SidebarMenuLabel>
                <SidebarMenuSuffix>
                  <Text class="text-xs text-muted">
                    {reader.documents().length}
                  </Text>
                </SidebarMenuSuffix>
              </SidebarMenuButton>
            </SidebarMenu>
          </SidebarGroup>
          <SidebarGroup>
            <SidebarGroupLabel>Documents</SidebarGroupLabel>
            <SidebarMenu value={reader.activeId()}>
              <ForValue each={reader.visibleDocuments()}>
                {(document) => (
                  <SidebarMenuButton
                    value={document.id}
                    onClick={() => {
                      reader.selectDocument(document.id);
                      navigate({ to: "/" });
                    }}
                  >
                    <SidebarMenuIcon>
                      <Icon source={fileText} size={14} />
                    </SidebarMenuIcon>
                    <SidebarMenuLabel>{document.title}</SidebarMenuLabel>
                    <Show when={document.kind === "guide"}>
                      <SidebarMenuSuffix>
                        <Text class="text-xs text-secondary">Guide</Text>
                      </SidebarMenuSuffix>
                    </Show>
                  </SidebarMenuButton>
                )}
              </ForValue>
            </SidebarMenu>
          </SidebarGroup>
        </SidebarContent>
        <SidebarFooter class="p-3">
          <Button
            class="w-full"
            disabled={reader.opening()}
            onClick={() => void open()}
          >
            <Icon source={plus} size={14} />
            {reader.opening() ? "Opening…" : "Open PDF"}
          </Button>
        </SidebarFooter>
      </WorkbenchSidebar>
      <WorkbenchMain>{props.children}</WorkbenchMain>
    </Workbench>
  );
}

export function AppShell(props: { children?: JSX.Element }) {
  return (
    <ReaderProvider>
      <Shell>{props.children}</Shell>
    </ReaderProvider>
  );
}
