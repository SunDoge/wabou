import "@wabou/ui";
import "virtual:wabou-stylesheet";
import { defineLayoutFixtures } from "@wabou/test/layout/fixtures";
import { WorkspacePanel } from "./workspace-panel";

defineLayoutFixtures({
  "workspace/files-panel": {
    width: 420,
    height: 720,
    render: () => (
      <WorkspacePanel
        cwd="/work/wabou"
        loadFiles={async () => ["README.md", "src/main.rs", "src/service.rs"]}
        readFile={async (_cwd, path) => ({
          path,
          text: "# Workspace preview\n\nA real layout fixture for the file inspector.",
        })}
        addContext={() => {}}
        close={() => {}}
      />
    ),
  },
});
