import { resolve } from "node:path";
import { getLayoutNode, queryLayoutNodes } from "@wabou/test/layout";
import {
  type LayoutFixtureCase,
  renderLayoutFixtures,
} from "@wabou/test/layout/node";

const command = process.env.WABOU_LAYOUT_COMMAND
  ? process.env.WABOU_LAYOUT_COMMAND.split(" ").filter(Boolean)
  : [resolve("target/release/wabou")];
const selected = process.argv.slice(2).filter(Boolean);
const selectCases = (...cases: LayoutFixtureCase[]): LayoutFixtureCase[] =>
  selected.length === 0
    ? cases
    : cases.filter(({ id }) => selected.includes(id));

function assertSetupWorkspace(
  fixture: Parameters<typeof getLayoutNode>[0],
  viewportWidth: number,
  viewportHeight: number,
): void {
  const shell = getLayoutNode(fixture, {
    role: "region",
    name: "Timestow setup workspace",
  });
  const navigation = getLayoutNode(fixture, {
    role: "group",
    name: "Primary navigation",
  });
  const form = getLayoutNode(fixture, {
    role: "group",
    name: "Backup repository setup",
  });
  const submit = getLayoutNode(fixture, {
    role: "button",
    name: "Create backup",
  });

  if (Math.abs(shell.rect.width - viewportWidth) > 0.5) {
    throw new Error(`setup shell width drifted: ${shell.rect.width}`);
  }
  if (Math.abs(navigation.rect.width - 224) > 0.5) {
    throw new Error(`profile sidebar width drifted: ${navigation.rect.width}`);
  }
  if (form.rect.x < navigation.rect.width - 0.5) {
    throw new Error(
      `setup form overlaps the profile sidebar: x=${form.rect.x}`,
    );
  }
  if (form.rect.x + form.rect.width > viewportWidth + 0.5) {
    throw new Error("setup form escapes the horizontal viewport");
  }
  if (form.rect.y + form.rect.height > viewportHeight + 0.5) {
    throw new Error("setup form escapes the declared minimum viewport");
  }
  if (submit.rect.x + submit.rect.width > form.rect.x + form.rect.width + 0.5) {
    throw new Error("primary setup action escapes its form surface");
  }
}

function assertWorkspaceHeader(
  fixture: Parameters<typeof getLayoutNode>[0],
): void {
  const toolbar = getLayoutNode(fixture, {
    role: "toolbar",
    name: "Backup workspace actions",
  });
  const backup = getLayoutNode(fixture, {
    role: "button",
    name: "Back up now",
  });
  const folders = getLayoutNode(fixture, {
    role: "button",
    name: "Manage backup folders",
  });
  const toolbarRight = toolbar.rect.x + toolbar.rect.width;
  for (const [name, control] of [
    ["folder control", folders],
    ["backup action", backup],
  ] as const) {
    if (
      control.rect.x < toolbar.rect.x - 0.5 ||
      control.rect.x + control.rect.width > toolbarRight + 0.5
    ) {
      throw new Error(`${name} escapes the minimum workspace toolbar`);
    }
  }
}

function assertFullWorkspace(
  fixture: Parameters<typeof getLayoutNode>[0],
  viewportWidth: number,
): void {
  const navigation = getLayoutNode(fixture, {
    role: "group",
    name: "Primary navigation",
  });
  const history = getLayoutNode(fixture, {
    role: "region",
    name: "Snapshot history",
  });
  const browser = getLayoutNode(fixture, {
    role: "region",
    name: "Snapshot browser",
  });
  const files = getLayoutNode(fixture, {
    role: "group",
    name: "Snapshot file workspace",
  });
  getLayoutNode(fixture, { role: "table", name: "Snapshot files" });

  const selectedTitleCount = queryLayoutNodes(fixture, {
    text: "Before photo cleanup",
  }).length;
  if (selectedTitleCount < 2) {
    throw new Error("the selected snapshot label is not the workspace title");
  }

  if (Math.abs(navigation.rect.width - 224) > 0.5) {
    throw new Error(`profile sidebar width drifted: ${navigation.rect.width}`);
  }
  if (Math.abs(history.rect.width - 256) > 0.5) {
    throw new Error(`snapshot rail width drifted: ${history.rect.width}`);
  }
  if (browser.rect.x + browser.rect.width > viewportWidth + 0.5) {
    throw new Error("snapshot browser escapes the application viewport");
  }
  if (Math.abs(files.rect.width - browser.rect.width) > 0.5) {
    throw new Error(
      `file workspace lost available width: files=${files.rect.width}, browser=${browser.rect.width}`,
    );
  }
  if (
    queryLayoutNodes(fixture, { role: "region", name: "File details" }).length >
    0
  ) {
    throw new Error("empty file selection still reserves a details rail");
  }
}

await renderLayoutFixtures({
  app: "apps/timestow",
  command,
  cases: selectCases(
    {
      id: "timestow/setup-wide",
      width: 1_200,
      height: 760,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => assertSetupWorkspace(fixture, 1_200, 760),
    },
    {
      id: "timestow/setup-minimum",
      width: 900,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => assertSetupWorkspace(fixture, 900, 620),
    },
    {
      id: "timestow/unlock-minimum",
      width: 900,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, { role: "group", name: "Unlock backup" });
        getLayoutNode(fixture, {
          role: "textbox",
          name: "Repository password",
        });
        getLayoutNode(fixture, { role: "button", name: "Unlock backup" });
      },
    },
    {
      id: "timestow/workspace-header-minimum",
      width: 676,
      height: 176,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: assertWorkspaceHeader,
    },
    {
      id: "timestow/workspace-wide",
      width: 1_440,
      height: 900,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => assertFullWorkspace(fixture, 1_440),
    },
    {
      id: "timestow/workspace-minimum",
      width: 900,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => assertFullWorkspace(fixture, 900),
    },
    {
      id: "timestow/workspace-paged-directory",
      width: 900,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        assertFullWorkspace(fixture, 900);
        getLayoutNode(fixture, { role: "button", name: "Load more files" });
        if (
          !fixture.nodes.some((node) => node.text?.includes("3 of 300 items"))
        ) {
          throw new Error(
            "paged file count does not expose the full directory size",
          );
        }
      },
    },
    {
      id: "timestow/workspace-many-snapshots",
      width: 900,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, {
          role: "textbox",
          name: "Filter snapshots",
        });
        getLayoutNode(fixture, {
          role: "region",
          name: "Snapshot history",
        });
        getLayoutNode(fixture, { role: "table", name: "Snapshot files" });
      },
    },
    {
      id: "timestow/workspace-empty",
      width: 900,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, { role: "region", name: "Snapshot history" });
        getLayoutNode(fixture, { text: "No snapshots yet" });
        getLayoutNode(fixture, {
          text: "Run your first backup to create a snapshot.",
        });
        getLayoutNode(fixture, { text: "Create your first snapshot" });
        getLayoutNode(fixture, { role: "button", name: "Back up now" });
        if (queryLayoutNodes(fixture, { text: "Select a snapshot" }).length) {
          throw new Error("empty repository asks the user to select nothing");
        }
        if (queryLayoutNodes(fixture, { role: "table" }).length > 0) {
          throw new Error("empty repository renders a stale file table");
        }
      },
    },
    {
      id: "timestow/workspace-error",
      width: 900,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, {
          text: "Repository index could not be read.",
        });
        getLayoutNode(fixture, {
          role: "alert",
          name: "History unavailable",
        });
        if (queryLayoutNodes(fixture, { text: "No snapshots yet" }).length) {
          throw new Error("repository failure is presented as empty history");
        }
        getLayoutNode(fixture, {
          role: "alert",
          name: "Snapshot history unavailable",
        });
      },
    },
    {
      id: "timestow/workspace-file-error",
      width: 900,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, {
          role: "alert",
          name: "Snapshot files unavailable",
        });
        getLayoutNode(fixture, { role: "button", name: "Retry" });
        getLayoutNode(fixture, {
          role: "button",
          name: "Open snapshot Before photo cleanup",
        });
        if (
          queryLayoutNodes(fixture, { text: "Before photo cleanup" }).length < 2
        ) {
          throw new Error("file failure clears the selected snapshot");
        }
        if (
          queryLayoutNodes(fixture, {
            role: "alert",
            name: "History unavailable",
          }).length
        ) {
          throw new Error("file failure invalidates the snapshot history");
        }
        if (queryLayoutNodes(fixture, { role: "table" }).length) {
          throw new Error("file failure renders an empty file table");
        }
      },
    },
    {
      id: "timestow/backup-progress-narrow",
      width: 420,
      height: 88,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, { role: "status" });
        getLayoutNode(fixture, {
          role: "progressbar",
          name: "Backup progress",
        });
        getLayoutNode(fixture, { text: "128.0 MB of 512.0 MB" });
      },
    },
    {
      id: "timestow/session-error-narrow",
      width: 420,
      height: 96,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, { role: "alert", name: "Timestow error" });
        getLayoutNode(fixture, {
          role: "button",
          name: "Dismiss Timestow error",
        });
      },
    },
    {
      id: "timestow/repository-check",
      width: 520,
      height: 340,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, {
          role: "dialog",
          name: "Check repository",
        });
        getLayoutNode(fixture, { text: "Check repository" });
        getLayoutNode(fixture, { role: "button", name: "Check now" });
        if (
          !fixture.nodes.some((node) =>
            node.text?.includes("does not read every stored data byte"),
          )
        ) {
          throw new Error("repository check overstates its verification scope");
        }
      },
    },
    {
      id: "timestow/repository-check-result",
      width: 480,
      height: 300,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, {
          role: "alert",
          name: "Repository structure is healthy",
        });
        getLayoutNode(fixture, {
          role: "group",
          name: "Repository statistics",
        });
        for (const label of [
          "Repository size",
          "Unique data",
          "Snapshots",
          "Active packs",
        ]) {
          getLayoutNode(fixture, { text: label });
        }
      },
    },
    {
      id: "timestow/changes-wide",
      width: 960,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, {
          role: "combobox",
          name: "Comparison snapshot",
        });
        getLayoutNode(fixture, { role: "table", name: "Snapshot changes" });
        const headers = queryLayoutNodes(fixture, { role: "columnheader" });
        if (headers.length !== 5) {
          throw new Error(
            `wide changes table exposes ${headers.length} columns`,
          );
        }
      },
    },
    {
      id: "timestow/changes-minimum",
      width: 420,
      height: 480,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        const table = getLayoutNode(fixture, {
          role: "table",
          name: "Snapshot changes",
        });
        if (table.rect.x + table.rect.width > 420.5) {
          throw new Error("changes table escapes its compact viewport");
        }
        const headers = queryLayoutNodes(fixture, { role: "columnheader" });
        if (headers.length !== 2) {
          throw new Error(
            `compact changes table exposes ${headers.length} columns`,
          );
        }
        if (
          !fixture.nodes.some((node) =>
            node.text?.includes("822.4 KB → 826.2 KB"),
          )
        ) {
          throw new Error("compact changes table lost its size transition");
        }
      },
    },
    {
      id: "timestow/file-details-rail",
      width: 288,
      height: 620,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, { role: "region", name: "File details" });
        if (!fixture.nodes.some((node) => node.text === "File")) {
          throw new Error("file details expose the internal file kind");
        }
        if (
          !fixture.nodes.some((node) => node.text === "2026-09-08 00:41:00 UTC")
        ) {
          throw new Error("file details omit the timestamp time zone");
        }
        getLayoutNode(fixture, {
          role: "button",
          name: "Open preview",
        });
        getLayoutNode(fixture, { role: "button", name: "Extract…" });
      },
    },
    {
      id: "timestow/restore-plan",
      width: 480,
      height: 300,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        getLayoutNode(fixture, { role: "region", name: "Restore plan" });
        getLayoutNode(fixture, {
          role: "alert",
          name: "Existing content will change",
        });
      },
    },
    {
      id: "timestow/backup-sources-narrow",
      width: 360,
      height: 320,
      checks: ["visible-overflow", "text-collision", "visual-quality"],
      assert: (fixture) => {
        const panel = getLayoutNode(fixture, {
          role: "group",
          name: "Backup folder selection",
        });
        getLayoutNode(fixture, {
          role: "textbox",
          name: "Backup folder",
        });
        const remove = getLayoutNode(fixture, {
          role: "button",
          name: "Remove /data/users/me/Documents/Long project name/Reference material",
        });
        if (
          remove.rect.x + remove.rect.width >
          panel.rect.x + panel.rect.width
        ) {
          throw new Error("long backup path pushes its remove action out");
        }
      },
    },
  ),
});
