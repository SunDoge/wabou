import { Button, Column, Icon, Row, Text, useNavigate, View } from "@wabou/ui";
import check from "lucide-static/icons/check.svg?raw";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
import layout from "lucide-static/icons/panel-right.svg?raw";
import pencil from "lucide-static/icons/pencil.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import trash from "lucide-static/icons/trash-2.svg?raw";

function Box(props: {
  name: string;
  size: string;
  color: string;
  layout?:
    | "flex-1"
    | "h-10"
    | "h-14"
    | "h-16"
    | "h-20"
    | "min-h-0 flex-1"
    | "min-w-0 flex-1"
    | "mt-3 min-h-0 flex-1"
    | "w-14"
    | "w-16"
    | "w-20"
    | "w-64";
  children?: import("solid-js").JSX.Element;
}) {
  return (
    <Column
      class="min-w-0 min-h-0 p-2 overflow-hidden border border-[#ffffff4d]"
      classList={{
        "flex-1": props.layout === "flex-1",
        "h-10": props.layout === "h-10",
        "h-14": props.layout === "h-14",
        "h-16": props.layout === "h-16",
        "h-20": props.layout === "h-20",
        "min-h-0 flex-1": props.layout === "min-h-0 flex-1",
        "min-w-0 flex-1": props.layout === "min-w-0 flex-1",
        "mt-3 min-h-0 flex-1": props.layout === "mt-3 min-h-0 flex-1",
        "w-14": props.layout === "w-14",
        "w-16": props.layout === "w-16",
        "w-20": props.layout === "w-20",
        "w-64": props.layout === "w-64",
      }}
      style={{ "background-color": props.color }}
    >
      <Row>
        <Text class="truncate text-xs font-medium">{props.name}</Text>
        <View class="flex-1" />
        <Text class="text-xs opacity-70">{props.size}</Text>
      </Row>
      {props.children}
    </Column>
  );
}

function Treemap() {
  return (
    <Column
      class="w-full min-h-0 flex-1 gap-1"
      role="img"
      aria-label="Disk usage treemap"
    >
      <Row class="min-h-0 flex-1 gap-1">
        <Column class="min-w-0 flex-1 gap-1">
          <Box name="work" size="35 GB" color="#7ab9ee" layout="h-16" />
          <Row class="min-h-0 flex-1 gap-1">
            <Box
              name="gpu-kit-worktrees"
              size="19 GB"
              color="#a98be0"
              layout="min-w-0 flex-1"
            >
              <Row class="mt-3 min-h-0 flex-1 gap-1">
                <Box
                  name="review-pr-3496"
                  size="8.4 GB"
                  color="#b394e3"
                  layout="min-w-0 flex-1"
                />
                <Column class="min-w-0 flex-1 gap-1">
                  <Box
                    name="review-pr-3501"
                    size="4.9 GB"
                    color="#a98ada"
                    layout="min-h-0 flex-1"
                  />
                  <Box
                    name="review-pr-3499"
                    size="4.8 GB"
                    color="#a486d7"
                    layout="min-h-0 flex-1"
                  />
                  <Box
                    name="review-pr-3500"
                    size="646 MB"
                    color="#9677ca"
                    layout="h-10"
                  />
                </Column>
              </Row>
            </Box>
            <Column class="w-44 gap-1">
              <Box
                name=".local"
                size="9.2 GB"
                color="#65c8aa"
                layout="min-h-0 flex-1"
              />
              <Box
                name="share"
                size="8.3 GB"
                color="#6acbb0"
                layout="min-h-0 flex-1"
              >
                <Text class="mt-3 text-xs opacity-70">Trash</Text>
                <Text class="text-xs opacity-70">7.2 GB</Text>
              </Box>
              <Row class="h-20 gap-1">
                <Box
                  name="longbridge"
                  size=""
                  color="#5fc6ae"
                  layout="flex-1"
                />
                <Box name="zed.app" size="" color="#54bfa5" layout="flex-1" />
                <Box name="bin" size="" color="#4eb79f" layout="w-14" />
              </Row>
            </Column>
          </Row>
        </Column>
        <Column class="w-52 gap-1">
          <Box name=".cargo" size="2.4 GB" color="#62c7a9" layout="h-20" />
          <Row class="min-h-0 flex-1 gap-1">
            <Column class="min-w-0 flex-1 gap-1">
              <Box name="registry" size="" color="#60c3a4" layout="h-14" />
              <Box name="src" size="" color="#58bca0" layout="min-h-0 flex-1" />
            </Column>
            <Column class="w-20 gap-1">
              <Box
                name="toolchains"
                size=""
                color="#5bbfa2"
                layout="min-h-0 flex-1"
              />
              <Box name="1.98.1" size="" color="#4db496" layout="h-16" />
              <Box name="nightly" size="" color="#45a98d" layout="h-16" />
            </Column>
          </Row>
          <Row class="h-28 gap-1">
            <Box
              name=".cache"
              size="1.8 GB"
              color="#74a9cd"
              layout="min-w-0 flex-1"
            />
            <Box name=".config" size="" color="#7ab1d4" layout="w-20" />
            <Box name=".codex" size="" color="#9b7bd1" layout="w-16" />
          </Row>
        </Column>
      </Row>
      <Row class="h-56 gap-1">
        <Box
          name="gpu-kit"
          size="8.4 GB"
          color="#55a9e0"
          layout="min-w-0 flex-1"
        >
          <Box
            name="target"
            size="8.3 GB"
            color="#f1bd55"
            layout="mt-3 min-h-0 flex-1"
          >
            <Text class="mt-2 text-xs opacity-70">Reclaimable</Text>
          </Box>
        </Box>
        <Box name="allsum-io" size="4.5 GB" color="#75b8dc" layout="w-64">
          <Box
            name="target"
            size="4.3 GB"
            color="#f1bc52"
            layout="mt-3 min-h-0 flex-1"
          />
        </Box>
        <Column class="w-64 gap-1">
          <Row class="min-h-0 flex-1 gap-1">
            <Box
              name="gpu-fast"
              size="2.8 GB"
              color="#e5ad47"
              layout="min-w-0 flex-1"
            />
            <Box name="zed" size="" color="#68a8d2" layout="w-20" />
          </Row>
          <Row class="h-20 gap-1">
            <Box
              name="target"
              size="2.8 GB"
              color="#e9b34f"
              layout="min-w-0 flex-1"
            />
            <Box name="bun" size="" color="#75a8d1" layout="w-20" />
          </Row>
        </Column>
      </Row>
    </Column>
  );
}

export function DiskTree() {
  const navigate = useNavigate();
  return (
    <Column
      role="group"
      aria-label="DiskTree explorer"
      class="w-full h-full overflow-hidden bg-[#f5f2f8] text-[#31303a] font-sans"
    >
      <Row class="h-14 px-4 flex-none items-center gap-3 border-b border-[#ded9e6] bg-[#faf8fc]">
        <Button
          variant="ghost"
          size="icon"
          aria-label="Back to benchmarks"
          class="w-8 h-8 text-[#8a64ce]"
          onClick={() => navigate({ to: "/" })}
        >
          <View class="grid grid-cols-2 gap-0.5">▦</View>
        </Button>
        <Column>
          <Text class="text-sm font-semibold">DiskTree</Text>
          <Text class="text-xs text-[#8d8794]">DISK EXPLORER</Text>
        </Column>
        <Button variant="ghost" size="icon" aria-label="Back">
          <Icon source={chevronLeft} size={14} />
        </Button>
        <Text class="text-xs text-[#8d8794]">/</Text>
        <Text class="text-xs text-[#8d8794]">home</Text>
        <Text class="text-xs">jason</Text>
        <View class="flex-1" />
        <Button variant="ghost" size="icon" aria-label="Edit path">
          <Icon source={pencil} size={14} />
        </Button>
        <Button variant="secondary" size="sm" selected>
          Size
        </Button>
        <Button variant="ghost" size="sm">
          Files
        </Button>
        <Button variant="ghost" size="sm">
          Age
        </Button>
        <Button variant="outline" size="icon" aria-label="Toggle inspector">
          <Icon source={layout} size={14} />
        </Button>
      </Row>
      <Row class="h-12 px-4 flex-none items-center gap-3 border-b border-[#ded9e6] bg-white">
        <Row class="items-center gap-2">
          <View class="w-4 h-4 rounded bg-[#8a68ca] flex items-center justify-center">
            <Icon source={check} size={10} class="text-white" />
          </View>
          <Text class="text-xs">Hidden files</Text>
        </Row>
        <Row class="items-center gap-2">
          <View class="w-4 h-4 rounded border border-[#bbbbbb]" />
          <Text class="text-xs">Apparent size</Text>
        </Row>
        <Text class="text-xs text-[#777777]">Depth</Text>
        <Button
          variant="outline"
          size="icon"
          aria-label="Decrease depth"
          class="w-7 h-7"
        >
          <Icon source={minus} size={12} />
        </Button>
        <Text class="text-xs">3</Text>
        <Button
          variant="outline"
          size="icon"
          aria-label="Increase depth"
          class="w-7 h-7"
        >
          <Icon source={plus} size={12} />
        </Button>
        <View class="flex-1" />
        <Row class="w-60 h-8 px-3 items-center gap-2 rounded-lg border border-[#dddddd] bg-white">
          <Icon source={search} size={13} />
          <Text class="text-xs text-[#777777]">Filter folders and files</Text>
        </Row>
      </Row>
      <Row class="min-h-0 flex-1">
        <Column class="min-w-0 flex-1 p-4 gap-3">
          <Row class="items-center">
            <Text class="text-xs text-[#8a8490]">
              Scanning · 297582 entries · partial
            </Text>
            <View class="flex-1" />
            <Button variant="outline" size="sm">
              × Cancel
            </Button>
          </Row>
          <Treemap />
          <Row class="items-center gap-3">
            {[
              ["#55a9e0", "Code"],
              ["#a98be0", "Agent scratch"],
              ["#65c8aa", "Toolchains"],
              ["#72badb", "Synced"],
              ["#8a70cb", "Git"],
              ["#e57e91", "Media"],
              ["#77a9ce", "Documents"],
              ["#e9b34f", "Cache"],
            ].map(([color, label]) => (
              <Row class="items-center gap-1">
                <View class="w-2 h-2" style={{ "background-color": color }} />
                <Text class="text-xs text-[#777777]">{label}</Text>
              </Row>
            ))}
          </Row>
        </Column>
        <Column class="w-96 h-full flex-none border-l border-[#ded9e6] bg-white">
          <Row class="h-10 px-4 items-center border-b border-[#eeeeee]">
            <Text class="text-xs text-[#938d9a]">INSPECTOR</Text>
            <View class="flex-1" />
            <Text class="text-xs text-[#aaaaaa]">Overview</Text>
          </Row>
          <Column class="p-5 gap-5">
            <Row class="items-center gap-3">
              <View class="w-10 h-10 rounded-lg bg-[#efe8ff] text-[#8967cc] flex items-center justify-center">
                <Icon source={folder} size={19} />
              </View>
              <Text class="text-base font-medium">jason</Text>
            </Row>
            <Text class="text-xs text-[#8d8794]">Space used</Text>
            <Text class="text-3xl font-medium">52 GB</Text>
            <View class="h-2 rounded-full bg-[#e5dff0]">
              <View class="w-full h-full rounded-full bg-[#7d5ac3]" />
            </View>
            <Row>
              <Column class="flex-1">
                <Text class="text-xs text-[#8d8794]">OF SCAN</Text>
                <Text class="text-sm">100.0%</Text>
              </Column>
              <Column class="flex-1">
                <Text class="text-xs text-[#8d8794]">FILES</Text>
                <Text class="text-sm">247597</Text>
              </Column>
            </Row>
            <Row>
              <Column class="flex-1">
                <Text class="text-xs text-[#8d8794]">LAST MODIFIED</Text>
                <Text class="text-sm">10/14/2026</Text>
              </Column>
              <Column class="flex-1">
                <Text class="text-xs text-[#8d8794]">CATEGORY</Text>
                <Text class="text-sm">Documents</Text>
              </Column>
            </Row>
            <Row class="gap-2">
              <Button variant="outline" class="flex-1 gap-2">
                <Icon source={folder} size={13} />
                Open
              </Button>
              <Button variant="outline" class="flex-1">
                Browse
              </Button>
              <Button variant="secondary" class="flex-1" disabled>
                ✓ Mark
              </Button>
            </Row>
            <Button variant="ghost" class="w-full text-[#aaaaaa]" disabled>
              <Icon source={trash} size={13} />
              Review 0 marked
            </Button>
            <Text class="text-sm font-medium">Worth a look</Text>
            <Text class="text-xs text-[#8d8794]">
              Scan in progress · sizes are estimates
            </Text>
            {[
              "target · 8.4 GB",
              "target · 8.3 GB",
              "target · 4.9 GB",
              "target · 4.8 GB",
              "target · 4.3 GB",
              ".cache · 2.8 GB",
            ].map((item) => (
              <Row class="items-center gap-2">
                <Icon source={folder} size={12} class="text-[#888888]" />
                <Column class="min-w-0 flex-1">
                  <Text class="truncate text-xs">{item.split(" · ")[0]}</Text>
                  <Text class="truncate text-xs text-[#999999]">
                    work/gpu-kit-worktrees/review
                  </Text>
                </Column>
                <View class="w-1 h-1 rounded-full bg-[#7651ba]" />
                <Text class="text-xs">{item.split(" · ")[1]}</Text>
              </Row>
            ))}
          </Column>
        </Column>
      </Row>
      <Row class="h-7 px-3 flex-none items-center gap-4 border-t border-[#ded9e6] bg-white">
        <Text class="text-xs">Space mark</Text>
        <Text class="text-xs">Enter browse</Text>
        <Text class="text-xs">⌘F filter</Text>
        <Text class="text-xs">↑↓ depth</Text>
        <Text class="text-xs">M metric</Text>
        <Text class="text-xs">R rescan</Text>
        <Text class="text-xs">motion on</Text>
        <Text class="text-xs">I inspector</Text>
      </Row>
    </Column>
  );
}
