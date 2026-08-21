import {
  Badge,
  Button,
  createTanStackDataTable,
  DataTable,
  Input,
  type TanStackDataTableColumn,
  Text,
  View,
} from "@wabou/ui";
import { Show } from "solid-js";

interface Project {
  id: string;
  name: string;
  owner: string;
  status: "Active" | "Review" | "Paused";
  score: number;
}

const data: Project[] = [
  { id: "router", name: "Router", owner: "Mina", status: "Active", score: 98 },
  {
    id: "cache",
    name: "Resource cache",
    owner: "Arun",
    status: "Review",
    score: 94,
  },
  {
    id: "table",
    name: "Data table",
    owner: "Iris",
    status: "Active",
    score: 91,
  },
  {
    id: "virtual",
    name: "Virtual lists",
    owner: "Noah",
    status: "Active",
    score: 88,
  },
  {
    id: "a11y",
    name: "Accessibility",
    owner: "Mina",
    status: "Review",
    score: 84,
  },
  {
    id: "editor",
    name: "Editor widget",
    owner: "Arun",
    status: "Paused",
    score: 76,
  },
];

const columns: TanStackDataTableColumn<Project>[] = [
  { accessorKey: "name", header: "Project" },
  { accessorKey: "owner", header: "Owner" },
  { accessorKey: "status", header: "Status" },
  { accessorKey: "score", header: "Score" },
];

function statusVariant(status: Project["status"]) {
  if (status === "Active") return "success" as const;
  if (status === "Paused") return "destructive" as const;
  return "secondary" as const;
}

export function DataTablePage() {
  const model = createTanStackDataTable<Project>({
    data,
    columns,
    getRowId: (row) => row.id,
    enableRowSelection: true,
  });

  return (
    <View class="flex flex-col gap-5">
      <View class="rounded-xl border border-subtle bg-surface overflow-hidden">
        <View class="p-4 flex items-center gap-3 border-b border-subtle">
          <Input
            aria-label="Filter projects"
            class="w-72"
            placeholder="Filter projects…"
            value={model.globalFilter()}
            onInput={(event) =>
              model.setGlobalFilter(event.currentTarget.value)
            }
          />
          <Text
            role="status"
            aria-label="Visible project count"
            class="text-sm text-muted"
          >
            {model.rows().length} visible
          </Text>
          <Show when={model.selectedCount() > 0}>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => model.setRowSelection({})}
            >
              Clear {model.selectedCount()} selected
            </Button>
          </Show>
        </View>

        <DataTable
          model={model}
          aria-label="Project table"
          selectable
          emptyMessage="No matching projects"
          renderCell={({ value, columnId }) => (
            <Show
              when={columnId === "status"}
              fallback={
                <Text class="w-full truncate text-sm text-primary">
                  {String(value)}
                </Text>
              }
            >
              <Badge variant={statusVariant(value as Project["status"])}>
                {String(value)}
              </Badge>
            </Show>
          )}
        />
      </View>

      <View class="rounded-lg border border-subtle bg-surface-muted p-4 flex flex-col gap-2">
        <Text class="text-sm font-semibold text-primary">
          Experiment boundary
        </Text>
        <Text class="whitespace-normal text-sm text-secondary">
          TanStack Table owns row models, sorting, filtering, and selection.
          Wabou owns rendering, semantics, input routing, and styling. This page
          uses Wabou's thin Solid adapter instead of duplicating the core state
          machine in the application.
        </Text>
      </View>
    </View>
  );
}
