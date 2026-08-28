import "@wabou/ui";
import "virtual:wabou-stylesheet";
import { defineLayoutFixtures } from "@wabou/test/layout/fixtures";
import { DiffViewer } from "@wabou/ui";
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
  "workspace/diff-viewer": {
    width: 640,
    height: 520,
    render: () => (
      <DiffViewer
        defaultExpanded={["src/main.ts"]}
        files={[
          {
            path: "src/main.ts",
            status: "modified",
            additions: 2,
            deletions: 1,
            patch: "@@ -1,2 +1,3 @@\n-old\n+new\n+line",
          },
          {
            path: "README.md",
            status: "added",
            additions: 4,
            deletions: 0,
            patch: "@@ -0,0 +1,4 @@\n+# Wabou\n+\n+Native UI\n+",
          },
        ]}
      />
    ),
  },
});
