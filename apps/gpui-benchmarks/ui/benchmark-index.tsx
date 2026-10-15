import { Button, Column, Icon, Row, Text, useNavigate, View } from "@wabou/ui";
import anchor from "lucide-static/icons/anchor.svg?raw";
import chart from "lucide-static/icons/chart-no-axes-combined.svg?raw";
import kanban from "lucide-static/icons/columns-3.svg?raw";
import files from "lucide-static/icons/folders.svg?raw";
import hardDrive from "lucide-static/icons/hard-drive.svg?raw";
import mail from "lucide-static/icons/mail.svg?raw";
import signature from "lucide-static/icons/signature.svg?raw";
import { For as ForValue } from "solid-js";
import skills from "./icons/skills.svg?raw";

const apps = [
  {
    path: "/skills",
    name: "Skills",
    detail: "Agent skill catalog",
    icon: skills,
    color: "#c8f11f",
  },
  {
    path: "/folio",
    name: "Folio Desk",
    detail: "Editorial correspondence",
    icon: mail,
    color: "#655bd7",
  },
  {
    path: "/tendra",
    name: "Tendra",
    detail: "Bid pipeline",
    icon: kanban,
    color: "#242422",
  },
  {
    path: "/ocean",
    name: "OceanView",
    detail: "Fleet command",
    icon: anchor,
    color: "#1656d8",
  },
  {
    path: "/signed",
    name: "Signed",
    detail: "PDF signing workspace",
    icon: signature,
    color: "#d8d8d6",
  },
  {
    path: "/omanta",
    name: "Omanta",
    detail: "Native file browser",
    icon: files,
    color: "#7aa2f7",
  },
  {
    path: "/disktree",
    name: "DiskTree",
    detail: "Disk usage treemap",
    icon: hardDrive,
    color: "#9277d3",
  },
] as const;

export function BenchmarkIndex() {
  const navigate = useNavigate();
  return (
    <View class="w-full h-full overflow-y-scroll bg-canvas text-primary font-sans">
      <Column class="w-full max-w-5xl mx-auto px-10 py-12 gap-8">
        <Column class="gap-2">
          <Row class="items-center gap-3">
            <View class="w-10 h-10 rounded-xl bg-accent text-on-accent flex items-center justify-center">
              <Icon source={chart} size={21} />
            </View>
            <Text role="heading" aria-level={1} class="text-3xl font-semibold">
              GPUI shell visual benchmarks
            </Text>
          </Row>
          <Text class="text-base text-muted">
            Seven native application shells rendered through Wabou's Solid,
            Style IR, Taffy, and Vello pipeline.
          </Text>
        </Column>
        <Row
          class="w-full flex-wrap gap-5"
          role="list"
          aria-label="Benchmark applications"
        >
          <ForValue each={apps}>
            {(app) => (
              <View role="listitem" aria-label={app.name}>
                <Button
                  variant="outline"
                  aria-label={`Open ${app.name}`}
                  class="w-72 h-28 p-4 flex-col items-start gap-3 rounded-xl border-subtle bg-surface shadow-sm"
                  onClick={() => navigate({ to: app.path })}
                >
                  <Row class="w-full items-center gap-3">
                    <View
                      class="w-9 h-9 rounded-lg flex items-center justify-center"
                      style={{ "background-color": app.color }}
                    >
                      <Icon source={app.icon} size={18} />
                    </View>
                    <Text class="text-base font-semibold">{app.name}</Text>
                  </Row>
                  <Text class="text-sm text-muted">{app.detail}</Text>
                </Button>
              </View>
            )}
          </ForValue>
        </Row>
      </Column>
    </View>
  );
}
