import {
  Button,
  Column,
  Icon,
  Row,
  SearchField,
  Text,
  useNavigate,
  View,
} from "@wabou/ui";
import arrowDownUp from "lucide-static/icons/arrow-down-up.svg?raw";
import externalLink from "lucide-static/icons/external-link.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import filter from "lucide-static/icons/list-filter.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import refresh from "lucide-static/icons/refresh-cw.svg?raw";
import trash from "lucide-static/icons/trash-2.svg?raw";
import upload from "lucide-static/icons/upload-cloud.svg?raw";
import { For as ForValue, type JSX } from "solid-js";
import bolt from "./icons/bolt.svg?raw";
import skills from "./icons/skills.svg?raw";

/**
 * Visual benchmark brief
 * User + job: an agent-tool user scans and installs skills.
 * Primary action: add a skill; discovery remains the dominant content surface.
 * Density + viewport: calm desktop density, 960x640 minimum, 1440x900 target.
 * Visual thesis: quiet neutral chrome around a precise three-column catalog.
 * Reference delta: static fixture data; native Wabou text, layout and SVG paint.
 * Proof: semantic layout checks plus a 1440x900 and 2x pixel comparison.
 */

const recommended = [
  {
    name: "frontend-design",
    description:
      "Guidance for distinctive, intentional visual design when building new UI or reshaping an existing one.",
    icon: skills,
  },
  {
    name: "mcp-builder",
    description:
      "Guide for creating high-quality MCP (Model Context Protocol) servers that enable LLMs to work with external tools.",
    icon: skills,
  },
  {
    name: "skill-creator",
    description:
      "Create new skills, modify and improve existing skills, and measure skill performance.",
    icon: skills,
  },
] as const;

const newSkills = [
  {
    name: "academy-guide",
    description:
      "Stop and check this skill before finishing any reply to a question about how to use Claude or its tools.",
    icon: bolt,
  },
  {
    name: "algorithmic-art",
    description:
      "Creating algorithmic art using p5.js with seeded randomness and interactive parameters.",
    icon: bolt,
  },
  {
    name: "brand-guidelines",
    description:
      "Applies Anthropic's official brand colors and typography to any sort of artifact that may benefit from them.",
    icon: bolt,
  },
  {
    name: "canvas-design",
    description:
      "Create beautiful visual art in PNG and PDF documents using an intentional design system.",
    icon: bolt,
  },
  {
    name: "claude-api",
    description:
      "Build applications with the Claude API using reliable patterns and practical examples.",
    icon: bolt,
  },
  {
    name: "theme-factory",
    description:
      "A toolkit for styling artifacts with a cohesive theme, type scale, and color palette.",
    icon: bolt,
  },
] as const;

function RailButton(props: {
  label: string;
  source: string;
  selected?: boolean;
}): JSX.Element {
  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label={props.label}
      selected={props.selected}
      class="w-9 h-9 rounded-lg"
      classList={{
        "border border-subtle bg-sidebar-raised shadow-sm text-sidebar-primary":
          props.selected,
        "text-sidebar-muted": !props.selected,
      }}
    >
      <Icon source={props.source} size={16} />
    </Button>
  );
}

function BrandMark(props: { onClick: () => void }): JSX.Element {
  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label="Back to benchmarks"
      onClick={props.onClick}
      class="w-7 h-7 p-0 flex-none rounded-md bg-[#c9f31d]"
    >
      <View class="w-3.5 h-3.5 rounded-full border-2 border-[#242422] flex items-center justify-center">
        <View class="w-1 h-full bg-[#242422]" />
      </View>
    </Button>
  );
}

function SkillCard(props: {
  name: string;
  description: string;
  icon: string;
}): JSX.Element {
  return (
    <View
      role="listitem"
      aria-label={props.name}
      class="min-w-64 flex-1 p-4 flex flex-col rounded-xl border border-subtle bg-surface"
      style={{ height: "164px" }}
    >
      <Row class="w-full min-w-0 items-center gap-3">
        <Icon source={props.icon} size={16} class="flex-none text-primary" />
        <Text class="min-w-0 flex-1 truncate text-base font-medium text-primary">
          {props.name}
        </Text>
        <Button
          variant="outline"
          size="icon"
          aria-label={`Install ${props.name}`}
          class="w-7 h-7 flex-none rounded-md border-subtle text-primary"
        >
          <Icon source={plus} size={15} />
        </Button>
      </Row>
      <Text
        maxLines={2}
        lineOverflow="clip"
        class="w-full h-12 mt-3 text-sm leading-6 text-muted"
      >
        {props.description}
      </Text>
      <View class="flex-1" />
      <Row class="items-center gap-2">
        <Text class="text-sm text-muted">by</Text>
        <View class="w-5 h-5 flex-none flex items-center justify-center rounded bg-[#b85338]">
          <Text class="text-xs font-semibold text-white">A</Text>
        </View>
        <Text class="text-sm text-primary">Anthropic</Text>
      </Row>
    </View>
  );
}

function SkillGroup(props: {
  title: string;
  skills: readonly { name: string; description: string; icon: string }[];
}): JSX.Element {
  return (
    <Column class="w-full mt-11 gap-6">
      <Text
        role="heading"
        aria-level={2}
        style={{ "font-size": "15px" }}
        class="font-medium text-muted"
      >
        {props.title}
      </Text>
      <Row
        role="list"
        aria-label={props.title}
        class="w-full flex-wrap items-stretch gap-6"
      >
        <ForValue each={props.skills}>
          {(skill) => <SkillCard {...skill} />}
        </ForValue>
      </Row>
    </Column>
  );
}

export function SkillsManager(): JSX.Element {
  const navigate = useNavigate();
  return (
    <View class="w-full h-full overflow-hidden flex bg-canvas font-sans select-none text-primary">
      <Column
        role="group"
        aria-label="Skills navigation"
        class="w-16 h-full flex-none items-center gap-2 pt-4 pb-4 bg-sidebar"
      >
        <BrandMark onClick={() => navigate({ to: "/" })} />
        <View class="h-2 flex-none" />
        <RailButton label="Skills" source={skills} selected />
        <RailButton label="Trash" source={trash} />
        <RailButton label="Folders" source={folder} />
        <View class="w-5 h-px my-2 flex-none bg-subtle" />
        <RailButton label="Refresh" source={refresh} />
        <RailButton label="Open source repository" source={externalLink} />
        <View class="w-5 h-px my-2 flex-none bg-subtle" />
        <RailButton label="Add skill" source={plus} />
      </Column>

      <Column class="m-2 ml-0 min-w-0 min-h-0 flex-1 overflow-hidden rounded-xl border border-subtle bg-surface">
        <Row class="h-14 flex-none px-5 items-center gap-2 border-b border-subtle">
          <Icon source={skills} size={15} class="text-muted" />
          <Text class="text-sm text-primary">Skills</Text>
        </Row>

        <View class="min-h-0 flex-1 overflow-y-scroll">
          <Column
            role="group"
            aria-label="Skills catalog"
            class="w-full max-w-6xl mx-auto px-8 pt-14 pb-10"
          >
            <Row class="w-full flex-wrap items-start justify-between gap-5">
              <Column class="gap-2">
                <Text
                  role="heading"
                  aria-level={1}
                  class="text-xl font-semibold text-primary"
                >
                  Skills
                </Text>
                <Text style={{ "font-size": "15px" }} class="text-muted">
                  Manage all your agent skills
                </Text>
              </Column>
              <Row class="items-center gap-3">
                <Button variant="outline" class="h-8 px-3 gap-2 border-subtle">
                  <Icon source={upload} size={16} />
                  Import
                </Button>
                <Button class="h-8 px-3 gap-2">
                  <Icon source={plus} size={16} />
                  Add skill
                </Button>
              </Row>
            </Row>

            <Column class="w-full mt-7 gap-4">
              <Row class="w-full flex-wrap items-center justify-between gap-4">
                <Row
                  role="tablist"
                  aria-label="Skill source"
                  class="h-8 flex-none items-center gap-1 p-0.5 rounded-lg border border-subtle bg-control"
                >
                  <Button
                    variant="ghost"
                    size="sm"
                    role="tab"
                    aria-selected={false}
                    class="h-7 px-2 text-muted"
                  >
                    Installed 10
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    role="tab"
                    aria-selected={true}
                    selected
                    class="h-7 px-2 border-subtle bg-surface shadow-sm text-primary"
                  >
                    Discover
                  </Button>
                </Row>
                <Row class="min-w-0 flex-1 justify-end items-center gap-3">
                  <SearchField
                    aria-label="Search skills"
                    placeholder="Search"
                    class="w-72 h-8"
                  />
                  <Button
                    variant="outline"
                    class="h-8 px-3 gap-2 border-subtle text-muted"
                  >
                    <Icon source={filter} size={15} />
                    Filter
                  </Button>
                  <Button
                    variant="outline"
                    class="h-8 px-3 gap-2 border-subtle text-muted"
                  >
                    <Icon source={arrowDownUp} size={15} />
                    Name
                  </Button>
                </Row>
              </Row>
              <Row class="items-center gap-2">
                <Text class="text-xs text-muted">
                  19 skills from anthropics/skills, openai/skills,
                  vercel-labs/agent-skills · updated 10 min ago
                </Text>
                <Text class="text-xs font-medium text-primary">Refresh</Text>
              </Row>
            </Column>

            <SkillGroup title="Recommended" skills={recommended} />
            <SkillGroup title="New skills" skills={newSkills} />
          </Column>
        </View>
      </Column>
    </View>
  );
}
