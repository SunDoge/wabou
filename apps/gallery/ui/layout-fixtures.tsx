import { defineLayoutFixtures } from "@wabou/test/layout/fixtures";
import { Button, Text, View } from "@wabou/ui";
import { createEffect, createSignal, onCleanup } from "solid-js";
import { initialAgentState } from "../../pi-agent/ui/agent-state";
import { ModelControls } from "../../pi-agent/ui/model-controls";
import { Sidebar as PiAgentSidebar } from "../../pi-agent/ui/sidebar";
import { galleryLayoutFixtures } from "./layout-fixture-pages";

let activeOwners = 0;

function TrackedFixture(props: { name: string; width: string }) {
  activeOwners++;
  onCleanup(() => activeOwners--);
  return (
    <View
      aria-label={props.name}
      class="flex flex-col gap-2 p-4"
      style={{ width: props.width }}
    >
      <Text aria-label={`${props.name} owner count`}>{activeOwners}</Text>
      <Text>{props.name}</Text>
    </View>
  );
}

function EffectFixture() {
  const [status, setStatus] = createSignal("pending");
  createEffect(
    () => "ready",
    (value) => {
      setStatus(value);
    },
  );
  return <Text aria-label="effect status">{status()}</Text>;
}

const fixtureAgents = [
  {
    id: "agent-1",
    name: "Agent 1",
    cwd: "/work/agent-one",
    proxy: "",
    noProxy: "",
    provider: "",
    model: "",
    state: { ...initialAgentState, connection: "ready" as const, items: [] },
  },
  {
    id: "agent-2",
    name: "Agent 2",
    cwd: "/work/a-repository-with-a-long-name",
    proxy: "",
    noProxy: "",
    provider: "",
    model: "",
    state: {
      ...initialAgentState,
      connection: "running" as const,
      sessionId: "session-2",
      items: [],
    },
  },
];

function PiAgentToolbarFixture() {
  return (
    <View
      aria-label="Pi agent toolbar fixture"
      class="w-full h-14 px-5 flex flex-row items-center gap-3 overflow-hidden border-b border-subtle"
    >
      <View class="min-w-0 flex-1 overflow-hidden">
        <Text class="truncate">Agent 2 · a-repository-with-a-long-name</Text>
      </View>
      <View class="min-w-0 flex-none overflow-hidden flex flex-row items-center gap-1">
        <Button size="icon" aria-label="Fixture terminal" />
        <Button size="icon" aria-label="Fixture files" />
        <ModelControls
          models={[
            {
              provider: "openai",
              id: "gpt-5.2-codex-with-a-long-model-id",
              name: "GPT 5.2 Codex with a long display name",
              reasoning: true,
            },
          ]}
          modelProvider="openai"
          modelId="gpt-5.2-codex-with-a-long-model-id"
          thinking="xhigh"
          thinkingLevels={["off", "medium", "xhigh"]}
          chooseModel={() => {}}
          chooseThinking={() => {}}
        />
        <Button size="icon" aria-label="Fixture search" />
      </View>
    </View>
  );
}

defineLayoutFixtures({
  narrow: {
    width: 640,
    height: 480,
    render: () => <TrackedFixture name="narrow" width="120px" />,
  },
  wide: {
    width: 960,
    height: 720,
    render: () => <TrackedFixture name="wide" width="320px" />,
  },
  "effect/synchronous": () => <EffectFixture />,
  "pi-agent/toolbar": {
    width: 892,
    height: 120,
    render: () => <PiAgentToolbarFixture />,
  },
  "pi-agent/sidebar": {
    width: 288,
    height: 620,
    render: () => (
      <PiAgentSidebar
        agents={fixtureAgents}
        activeId="agent-2"
        select={() => {}}
        add={() => {}}
        newSession={() => {}}
        canCreateSession
        openSettings={() => {}}
        sessions={[
          {
            agentId: "agent-1",
            sessionId: "hidden-session",
            sessionFile: "hidden.jsonl",
            name: "Hidden inactive session",
            cwd: "/work/agent-one",
            updatedAt: 1,
          },
          {
            agentId: "agent-2",
            sessionId: "session-2",
            sessionFile: "active.jsonl",
            name: "A long active session name that must truncate",
            cwd: "/work/a-repository-with-a-long-name",
            updatedAt: 2,
          },
        ]}
        selectSession={() => {}}
      />
    ),
  },
  ...galleryLayoutFixtures,
});
