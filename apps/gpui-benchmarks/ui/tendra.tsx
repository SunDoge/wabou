import { Button, Column, Icon, Row, Text, useNavigate, View } from "@wabou/ui";
import bell from "lucide-static/icons/bell.svg?raw";
import bookOpen from "lucide-static/icons/book-open.svg?raw";
import briefcase from "lucide-static/icons/briefcase-business.svg?raw";
import chart from "lucide-static/icons/chart-no-axes-column-increasing.svg?raw";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import circleHelp from "lucide-static/icons/circle-help.svg?raw";
import layout from "lucide-static/icons/layout-dashboard.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import settings from "lucide-static/icons/settings-2.svg?raw";
import users from "lucide-static/icons/users.svg?raw";
import { For as ForValue } from "solid-js";

const stages = [
  {
    title: "Go / no-go",
    value: "$1.9M",
    bids: "2 bids",
    color: "#858585",
    bars: 8,
  },
  {
    title: "Drafting",
    value: "$6.5M",
    bids: "3 bids",
    color: "#315edb",
    bars: 18,
  },
  {
    title: "In review",
    value: "$9.6M",
    bids: "4 bids",
    color: "#d47b06",
    bars: 22,
  },
  {
    title: "Submitted",
    value: "$6.1M",
    bids: "4 bids",
    color: "#252525",
    bars: 15,
  },
  {
    title: "Awarded",
    value: "$3.9M",
    bids: "2 bids",
    color: "#21854c",
    bars: 13,
  },
] as const;

const columns = [
  {
    title: "Go / no-go",
    tone: "#d7a740",
    cards: ["City of Johnson City", "Sevier County"],
  },
  {
    title: "Drafting",
    tone: "#5d7ed9",
    cards: ["Chattanooga Authority", "Town of Greeneville"],
  },
  {
    title: "In review",
    tone: "#d99b31",
    cards: ["City of Kingsport, TN", "Tennessee DOT"],
  },
  {
    title: "Submitted",
    tone: "#454545",
    cards: ["City of Bristol, TN", "Sullivan County"],
  },
  {
    title: "Awarded",
    tone: "#4f9a69",
    cards: ["City of Johnson City", "Washington County"],
  },
] as const;

function BidCard(props: { name: string; index: number; tone: string }) {
  const values = [
    "$640K",
    "$1.3M",
    "$1.1M",
    "$1.4M",
    "$2.4M",
    "$5.8M",
    "$1.6M",
    "$920K",
    "$2.8M",
  ];
  return (
    <Column class="w-full h-44 p-3 gap-2 rounded-lg border border-[#dedede] bg-white shadow-sm">
      <Row class="items-center gap-2">
        <View class="px-1.5 py-1 rounded bg-[#f2f2f2]">
          <Text class="text-xs font-medium">
            {props.index % 2 ? "DRAFTING" : "DECIDE BY"}
          </Text>
        </View>
        <View class="flex-1" />
        <Text class="text-xs text-[#929292]">◷ {props.index + 3} days</Text>
      </Row>
      <Text maxLines={1} class="text-sm font-medium">
        {props.name}
      </Text>
      <Text class="text-xs text-[#777777]">
        ITB {1140 + props.index} · Water infrastructure
      </Text>
      <Row class="items-center gap-2">
        <Text class="text-xs text-[#8a8a8a]">Bid started</Text>
        <View class="flex-1 h-1 rounded-full bg-[#ededed]">
          <View
            class="h-full rounded-full"
            style={{
              width: `${35 + props.index * 7}%`,
              "background-color": props.tone,
            }}
          />
        </View>
      </Row>
      <View class="flex-1" />
      <Row class="items-center">
        <View class="w-6 h-6 rounded-full bg-[#ececff] flex items-center justify-center">
          <Text class="text-xs">LS</Text>
        </View>
        <View class="flex-1" />
        <Text class="text-sm font-semibold">
          {values[props.index % values.length]}
        </Text>
      </Row>
    </Column>
  );
}

export function Tendra() {
  const navigate = useNavigate();
  return (
    <View
      role="group"
      aria-label="Tendra bid pipeline"
      class="w-full h-full overflow-hidden flex bg-[#eeeeed] text-[#202020] font-sans"
    >
      <Column class="w-60 h-full flex-none border-r border-[#dedede] bg-[#f7f7f6]">
        <Row class="h-14 px-4 flex-none items-center gap-3">
          <Button
            variant="ghost"
            size="icon"
            aria-label="Back to benchmarks"
            class="w-8 h-8 rounded-lg bg-[#262626] text-white"
            onClick={() => navigate({ to: "/" })}
          >
            <Text class="text-xs font-bold">T</Text>
          </Button>
          <Text class="font-semibold">Tendra</Text>
          <View class="flex-1" />
          <Button variant="ghost" size="icon" aria-label="Collapse sidebar">
            <Icon source={layout} size={15} />
          </Button>
        </Row>
        <View class="mx-3 p-3 rounded-lg border border-[#dddddd] bg-white">
          <Row class="items-center gap-2">
            <View class="w-8 h-8 rounded-md bg-[#29313b] text-white flex items-center justify-center">
              <Text class="text-xs">H&P</Text>
            </View>
            <Column class="min-w-0 flex-1">
              <Text class="truncate text-xs font-semibold">
                Harbor & Pike Civil
              </Text>
              <Text class="text-xs text-[#777777]">Capture team</Text>
            </Column>
            <Icon source={chevronDown} size={13} />
          </Row>
        </View>
        <Row class="mx-3 mt-3 h-8 px-2 items-center gap-2 rounded-md bg-[#ededec] text-[#777777]">
          <Icon source={search} size={14} />
          <Text class="text-xs">Search</Text>
          <View class="flex-1" />
          <Text class="text-xs">⌘K</Text>
        </Row>
        <Column class="px-3 pt-4 gap-1">
          <Text class="px-2 pb-2 text-xs text-[#999999]">WORKSPACE</Text>
          {[
            [layout, "Today"],
            [briefcase, "Opportunities"],
            [layout, "Bids"],
            [bookOpen, "Compliance"],
            [bookOpen, "Answer library"],
            [users, "Partners"],
          ].map(([icon, label], index) => (
            <Row
              class="h-9 px-2 items-center gap-3 rounded-md"
              classList={{ "bg-white border border-[#dddddd]": index === 2 }}
            >
              <Icon source={icon} size={14} />
              <Text class="text-sm">{label}</Text>
              <View class="flex-1" />
              {index === 1 ? (
                <View class="px-1.5 py-0.5 rounded bg-[#ffe5e5]">
                  <Text class="text-xs text-[#bd3c3c]">3 NEW</Text>
                </View>
              ) : index === 2 ? (
                <Text class="text-xs">9</Text>
              ) : null}
            </Row>
          ))}
          <Text class="px-2 pt-4 pb-2 text-xs text-[#999999]">INSIGHTS</Text>
          {[
            [chart, "Win and loss"],
            [settings, "Settings"],
            [circleHelp, "Help"],
          ].map(([icon, label]) => (
            <Row class="h-9 px-2 items-center gap-3">
              <Icon source={icon} size={14} />
              <Text class="text-sm">{label}</Text>
            </Row>
          ))}
        </Column>
        <View class="flex-1" />
        <Column class="m-3 p-3 gap-2 rounded-lg border border-[#dddddd] bg-white">
          <Row class="items-center gap-2">
            <View class="w-2 h-2 rounded-full bg-[#31b66b]" />
            <Text class="text-xs font-medium">Watching 15 portals</Text>
          </Row>
          <Text class="text-xs text-[#777777]">
            SAM.gov and 14 state and city portals checked 6 min ago
          </Text>
        </Column>
        <Row class="h-14 px-4 items-center gap-3 border-t border-[#dddddd]">
          <View class="w-8 h-8 rounded-full bg-[#e9ebf8] flex items-center justify-center">
            <Text class="text-xs">DO</Text>
          </View>
          <Column>
            <Text class="text-xs font-medium">Dana Ortiz</Text>
            <Text class="text-xs text-[#777777]">Capture lead</Text>
          </Column>
        </Row>
      </Column>

      <Column class="min-w-0 h-full flex-1 bg-white">
        <Row class="h-14 px-5 flex-none items-center gap-2 border-b border-[#e2e2e2]">
          <Icon source={briefcase} size={13} class="text-[#858585]" />
          <Text class="text-xs">Bids</Text>
          <Text class="text-xs text-[#aaaaaa]">›</Text>
          <Text class="text-xs">Pipeline</Text>
          <View class="flex-1" />
          <Text class="text-xs text-[#777777]">
            Tue, Oct 13, 2026 · 10:46 AM ET
          </Text>
          <Button variant="ghost" size="icon" aria-label="Notifications">
            <Icon source={bell} size={14} />
          </Button>
        </Row>
        <Column class="min-h-0 flex-1 overflow-y-scroll px-6 pt-5 pb-6 gap-4">
          <Row class="items-end">
            <Column class="gap-1">
              <Text
                role="heading"
                aria-level={1}
                class="text-2xl font-semibold"
              >
                Bid pipeline
              </Text>
              <Text class="text-sm">
                9 active bids worth $18.0M. 2 are stuck and need you today.
              </Text>
            </Column>
            <View class="flex-1" />
            <Button variant="secondary" size="sm">
              Team
            </Button>
            <Button variant="ghost" size="sm">
              Mine
            </Button>
            <Button variant="outline" size="icon" aria-label="Board view">
              <Icon source={layout} size={14} />
            </Button>
            <Button class="ml-2 h-8 px-3 gap-2">
              <Icon source={plus} size={14} />
              Add a bid
            </Button>
          </Row>
          <Row class="w-full gap-3" role="list" aria-label="Pipeline summary">
            <ForValue each={stages}>
              {(stage) => (
                <Column
                  role="listitem"
                  aria-label={stage.title}
                  class="min-w-0 flex-1 h-32 p-3 gap-2 rounded-xl border border-[#dddddd] bg-[#fafafa]"
                >
                  <Row>
                    <Text class="text-xs text-[#777777]">
                      {stage.title.toUpperCase()}
                    </Text>
                    <View class="flex-1" />
                    <Text class="text-xs text-[#999999]">{stage.bids}</Text>
                  </Row>
                  <Text
                    class="text-xl font-medium"
                    style={{ color: stage.color }}
                  >
                    {stage.value}
                  </Text>
                  <Row class="h-4 items-end gap-px">
                    <ForValue each={Array.from({ length: 24 })}>
                      {(_, i) => (
                        <View
                          class="w-0.5"
                          style={{
                            height: `${3 + ((i() * 7 + stage.bars) % 12)}px`,
                            "background-color": stage.color,
                          }}
                        />
                      )}
                    </ForValue>
                  </Row>
                  <Text class="text-xs text-[#777777]">
                    2 waiting for a decision
                  </Text>
                </Column>
              )}
            </ForValue>
          </Row>
          <Row
            class="w-full min-h-0 flex-1 gap-3"
            role="list"
            aria-label="Bid columns"
          >
            <ForValue each={columns}>
              {(column, ci) => (
                <Column
                  role="listitem"
                  aria-label={column.title}
                  class="min-w-40 flex-1 p-2 gap-3 rounded-xl border border-[#dedede] bg-[#f7f7f7]"
                >
                  <Row class="px-1 items-center">
                    <View
                      class="w-2 h-2 rounded-full"
                      style={{ "background-color": column.tone }}
                    />
                    <Text class="ml-2 text-xs font-medium">{column.title}</Text>
                    <View class="flex-1" />
                    <Text class="text-xs text-[#888888]">
                      {column.cards.length}
                    </Text>
                    <Text class="ml-2 text-xs text-[#aaaaaa]">•••</Text>
                  </Row>
                  <ForValue each={column.cards}>
                    {(name, i) => (
                      <BidCard
                        name={name}
                        index={ci() * 2 + i()}
                        tone={column.tone}
                      />
                    )}
                  </ForValue>
                </Column>
              )}
            </ForValue>
          </Row>
        </Column>
      </Column>
    </View>
  );
}
