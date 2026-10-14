import { Button, Column, Icon, Row, Text, useNavigate, View } from "@wabou/ui";
import arrowLeft from "lucide-static/icons/arrow-left.svg?raw";
import arrowRight from "lucide-static/icons/arrow-right.svg?raw";
import chevronUp from "lucide-static/icons/chevron-up.svg?raw";
import columns from "lucide-static/icons/columns-2.svg?raw";
import copy from "lucide-static/icons/copy.svg?raw";
import more from "lucide-static/icons/ellipsis.svg?raw";
import file from "lucide-static/icons/file.svg?raw";
import fileCode from "lucide-static/icons/file-code.svg?raw";
import fileImage from "lucide-static/icons/file-image.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import grid from "lucide-static/icons/grid-2x2.svg?raw";
import home from "lucide-static/icons/house.svg?raw";
import list from "lucide-static/icons/list.svg?raw";
import monitor from "lucide-static/icons/monitor.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import refresh from "lucide-static/icons/refresh-cw.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import star from "lucide-static/icons/star.svg?raw";
import trash from "lucide-static/icons/trash-2.svg?raw";
import { For as ForValue } from "solid-js";

const files = [
  { name: "Desktop", kind: "folder", detail: "Folder" },
  { name: "Documents", kind: "folder", detail: "Folder" },
  { name: "Downloads", kind: "folder", detail: "Folder" },
  { name: "Pictures", kind: "folder", detail: "Folder" },
  { name: "Projects", kind: "folder", detail: "Folder" },
  { name: ".config", kind: "folder", detail: "Hidden folder" },
  { name: "notes.md", kind: "code", detail: "Markdown · 8 KB" },
  { name: "wallpaper.png", kind: "image", detail: "PNG image · 2.4 MB" },
  { name: "todo.txt", kind: "file", detail: "Text · 1 KB" },
] as const;

const iconFor = { folder, code: fileCode, image: fileImage, file } as const;

function Place(props: { source: string; label: string; active?: boolean }) {
  return (
    <Row
      class="h-9 px-3 items-center gap-3"
      classList={{
        "bg-[#28334d] text-[#c9d6f2]": props.active,
        "text-[#a9b1c5]": !props.active,
      }}
    >
      <Icon source={props.source} size={15} />
      <Text class="text-sm">{props.label}</Text>
    </Row>
  );
}

export function Omanta() {
  const navigate = useNavigate();
  return (
    <Column
      role="group"
      aria-label="Omanta file browser"
      class="w-full h-full overflow-hidden bg-[#161821] text-[#c0caf5] font-sans"
    >
      <Row class="h-12 flex-none items-center border-b border-[#2b3040] bg-[#1a1d28]">
        <Button
          variant="ghost"
          size="icon"
          aria-label="Back to benchmarks"
          class="ml-2 rounded-none text-[#7aa2f7]"
          onClick={() => navigate({ to: "/" })}
        >
          <Text class="text-lg font-bold">O</Text>
        </Button>
        <Row class="h-full items-center px-3 gap-2 border-r border-[#2b3040]">
          <Icon source={folder} size={14} />
          <Text class="text-sm font-medium">Home</Text>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Close tab"
            class="w-6 h-6"
          >
            <Text>×</Text>
          </Button>
        </Row>
        <Button
          variant="ghost"
          size="icon"
          aria-label="New tab"
          class="rounded-none"
        >
          <Icon source={plus} size={14} />
        </Button>
        <View class="flex-1" />
        <Button
          variant="ghost"
          size="icon"
          aria-label="Split view"
          class="rounded-none"
        >
          <Icon source={columns} size={15} />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          aria-label="More options"
          class="rounded-none"
        >
          <Icon source={more} size={15} />
        </Button>
      </Row>
      <Row class="h-12 px-3 flex-none items-center gap-2 border-b border-[#2b3040] bg-[#1f2330]">
        <Button variant="ghost" size="icon" aria-label="Back">
          <Icon source={arrowLeft} size={15} />
        </Button>
        <Button variant="ghost" size="icon" aria-label="Forward" disabled>
          <Icon source={arrowRight} size={15} />
        </Button>
        <Button variant="ghost" size="icon" aria-label="Parent folder">
          <Icon source={chevronUp} size={15} />
        </Button>
        <Button variant="ghost" size="icon" aria-label="Refresh">
          <Icon source={refresh} size={14} />
        </Button>
        <Row class="min-w-0 flex-1 h-8 px-3 items-center gap-2 border border-[#343a4d] bg-[#171a24]">
          <Icon source={folder} size={13} />
          <Text class="text-sm">/home/jason</Text>
        </Row>
        <Row class="w-56 h-8 px-3 items-center gap-2 border border-[#343a4d] bg-[#171a24]">
          <Icon source={search} size={13} />
          <Text class="text-sm text-[#747d98]">Search this folder</Text>
        </Row>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Grid view"
          class="bg-[#2b3142]"
        >
          <Icon source={grid} size={14} />
        </Button>
        <Button variant="ghost" size="icon" aria-label="List view">
          <Icon source={list} size={14} />
        </Button>
      </Row>
      <Row class="min-h-0 flex-1">
        <Column class="w-52 h-full flex-none border-r border-[#2b3040] bg-[#1a1d27]">
          <Text class="px-4 pt-4 pb-2 text-xs text-[#676f87]">PLACES</Text>
          <Place source={home} label="Home" active />
          <Place source={monitor} label="Desktop" />
          <Place source={folder} label="Documents" />
          <Place source={folder} label="Downloads" />
          <Place source={fileImage} label="Pictures" />
          <Text class="px-4 pt-5 pb-2 text-xs text-[#676f87]">BOOKMARKS</Text>
          <Place source={star} label="Projects" />
          <Place source={star} label="Design" />
          <View class="flex-1" />
          <Place source={trash} label="Trash" />
        </Column>
        <Column class="min-w-0 h-full flex-1">
          <Row class="h-10 px-4 items-center border-b border-[#2b3040]">
            <Text class="text-xs text-[#798198]">9 items</Text>
            <View class="flex-1" />
            <Text class="text-xs text-[#798198]">Name ↑</Text>
          </Row>
          <View class="min-h-0 flex-1 overflow-y-scroll p-5">
            <Row
              role="list"
              aria-label="Home folder files"
              class="w-full flex-wrap gap-4"
            >
              <ForValue each={files}>
                {(entry, index) => (
                  <Column
                    role="listitem"
                    aria-label={entry.name}
                    class="w-36 h-32 p-3 items-center gap-2 border"
                    classList={{
                      "border-[#7aa2f7] bg-[#27324a]": index() === 4,
                      "border-transparent bg-[#1c202c]": index() !== 4,
                    }}
                  >
                    <Icon
                      source={iconFor[entry.kind]}
                      size={38}
                      class={
                        entry.kind === "folder"
                          ? "text-[#e0af68]"
                          : "text-[#7aa2f7]"
                      }
                    />
                    <Text class="w-full truncate text-center text-sm">
                      {entry.name}
                    </Text>
                    <Text class="w-full truncate text-center text-xs text-[#6e7690]">
                      {entry.detail}
                    </Text>
                  </Column>
                )}
              </ForValue>
            </Row>
          </View>
        </Column>
        <Column class="w-72 h-full flex-none border-l border-[#2b3040] bg-[#191c26]">
          <Row class="h-10 px-4 items-center border-b border-[#2b3040]">
            <Text class="text-xs text-[#747d94]">PREVIEW</Text>
            <View class="flex-1" />
            <Button
              variant="ghost"
              size="icon"
              aria-label="Copy path"
              class="w-6 h-6"
            >
              <Icon source={copy} size={13} />
            </Button>
          </Row>
          <Column class="p-5 items-center gap-4">
            <View class="w-20 h-20 flex items-center justify-center">
              <Icon source={folder} size={64} class="text-[#e0af68]" />
            </View>
            <Text class="text-lg font-medium">Projects</Text>
            <Text class="text-sm text-[#798198]">Folder</Text>
            <View class="w-full h-px bg-[#2b3040]" />
            <Row class="w-full">
              <Text class="text-xs text-[#6f7890]">Location</Text>
              <View class="flex-1" />
              <Text class="text-xs">/home/jason</Text>
            </Row>
            <Row class="w-full">
              <Text class="text-xs text-[#6f7890]">Modified</Text>
              <View class="flex-1" />
              <Text class="text-xs">Today, 14:32</Text>
            </Row>
            <Row class="w-full">
              <Text class="text-xs text-[#6f7890]">Contents</Text>
              <View class="flex-1" />
              <Text class="text-xs">18 items</Text>
            </Row>
          </Column>
        </Column>
      </Row>
      <Row class="h-7 px-3 flex-none items-center gap-5 border-t border-[#2b3040] bg-[#171a23]">
        <Text class="text-xs text-[#6f7890]">Ctrl+L location</Text>
        <Text class="text-xs text-[#6f7890]">Ctrl+F search</Text>
        <Text class="text-xs text-[#6f7890]">Space preview</Text>
        <View class="flex-1" />
        <Text class="text-xs text-[#6f7890]">9 items · 2.4 MB</Text>
      </Row>
    </Column>
  );
}
