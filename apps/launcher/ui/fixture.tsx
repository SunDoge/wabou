import "@wabou/ui";
import "virtual:wabou-stylesheet";
import { ColorThemeProvider, ComponentsProvider, mount } from "@wabou/ui";
import { LauncherSurface } from "./launcher";

const entries = [
  {
    id: "code.desktop",
    label: "Visual Studio Code",
    description: "Code Editing. Redefined.",
    keywords: ["editor", "development", "typescript"],
  },
  {
    id: "firefox.desktop",
    label: "Firefox",
    description: "Web Browser",
    keywords: ["internet", "web", "browser"],
  },
  {
    id: "terminal.desktop",
    label: "Terminal",
    description: "Command line",
    keywords: ["shell", "console"],
  },
  {
    id: "files.desktop",
    label: "Files",
    description: "Access and organize files",
    keywords: ["folder", "documents"],
  },
  {
    id: "settings.desktop",
    label: "Settings",
    description: "System configuration",
    keywords: ["preferences", "control"],
  },
];

mount(() => (
  <ColorThemeProvider theme="dark" transition={false}>
    <ComponentsProvider theme="dark">
      <LauncherSurface
        entries={entries}
        onLaunch={() => {}}
        onDismiss={() => {}}
      />
    </ComponentsProvider>
  </ColorThemeProvider>
));
