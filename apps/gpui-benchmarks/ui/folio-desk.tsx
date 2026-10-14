import { Button, Column, Icon, Row, Text, useNavigate, View } from "@wabou/ui";
import archive from "lucide-static/icons/archive.svg?raw";
import calendar from "lucide-static/icons/calendar-days.svg?raw";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import chevronUp from "lucide-static/icons/chevron-up.svg?raw";
import clock from "lucide-static/icons/clock-3.svg?raw";
import file from "lucide-static/icons/file-text.svg?raw";
import inbox from "lucide-static/icons/inbox.svg?raw";
import paperclip from "lucide-static/icons/paperclip.svg?raw";
import pen from "lucide-static/icons/pen-line.svg?raw";
import send from "lucide-static/icons/send.svg?raw";
import sparkle from "lucide-static/icons/sparkles.svg?raw";
import { For as ForValue } from "solid-js";

const messages = [
  {
    initials: "JM",
    name: "Jules Morgan",
    company: "Margin Press",
    subject: "Your spring catalogue, on paper",
    color: "#efd9c8",
    active: true,
  },
  {
    initials: "OR",
    name: "Owen Reed",
    company: "Still House",
    subject: "Stockists for late autumn",
    color: "#c9e0eb",
  },
  {
    initials: "CF",
    name: "Clara Ellis",
    company: "Paper & Pine",
    subject: "The quiet shelf",
    color: "#d9d5ef",
  },
  {
    initials: "EG",
    name: "Elliot Gray",
    company: "Daybreak Co.",
    subject: "A small run for June",
    color: "#c9e7df",
  },
  {
    initials: "NB",
    name: "Nina Bell",
    company: "Soft Assembly",
    subject: "Notes for the next edition",
    color: "#e8d8f2",
  },
  {
    initials: "LW",
    name: "Lena Woods",
    company: "Quiet Objects",
    subject: "Thank you for the proofs",
    color: "#f1d7c8",
  },
] as const;

function Avatar(props: { initials: string; color: string }) {
  return (
    <View
      class="w-9 h-9 flex-none rounded-lg flex items-center justify-center"
      style={{ "background-color": props.color }}
    >
      <Text class="text-xs font-semibold text-secondary">{props.initials}</Text>
    </View>
  );
}

export function FolioDesk() {
  const navigate = useNavigate();
  return (
    <View
      role="group"
      aria-label="Folio Desk correspondence"
      class="w-full h-full overflow-hidden flex bg-[#eeece7] text-[#272520] font-sans"
    >
      <Column class="w-12 h-full flex-none items-center py-4 gap-3 border-r border-[#dedbd3] bg-[#f6f4ef]">
        <Button
          variant="ghost"
          size="icon"
          aria-label="Back to benchmarks"
          class="w-8 h-8 rounded-lg bg-[#6659d8] text-white"
          onClick={() => navigate({ to: "/" })}
        >
          <Text class="font-semibold">F</Text>
        </Button>
        <View class="h-2" />
        <Button
          variant="ghost"
          size="icon"
          aria-label="Inbox"
          class="text-[#6f6b63] bg-white shadow-sm"
        >
          <Icon source={inbox} size={15} />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Drafts"
          class="text-[#9a968d]"
        >
          <Icon source={file} size={15} />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Archive"
          class="text-[#9a968d]"
        >
          <Icon source={archive} size={15} />
        </Button>
        <View class="h-px w-5 my-2 bg-[#dedbd3]" />
        <Button
          variant="ghost"
          size="icon"
          aria-label="Calendar"
          class="text-[#9a968d]"
        >
          <Icon source={calendar} size={15} />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Later"
          class="text-[#9a968d]"
        >
          <Icon source={clock} size={15} />
        </Button>
      </Column>

      <Column class="w-80 h-full flex-none border-r border-[#dedbd3] bg-[#f8f6f1]">
        <Row class="h-14 px-5 flex-none items-center border-b border-[#dedbd3]">
          <Text class="flex-1 text-sm font-semibold">Correspondence</Text>
          <Icon source={chevronDown} size={14} class="text-[#8e8a82]" />
        </Row>
        <Column class="px-5 pt-5 gap-4 min-h-0 flex-1 overflow-y-scroll">
          <Text class="text-xs text-[#8d887e]">Ready to send</Text>
          <ForValue each={messages.slice(0, 5)}>
            {(message) => (
              <View
                class="w-full p-3 rounded-lg"
                classList={{
                  "bg-[#eae4d7]": "active" in message && message.active,
                  "bg-transparent": !("active" in message && message.active),
                }}
              >
                <Row class="items-center gap-3">
                  <Avatar initials={message.initials} color={message.color} />
                  <Column class="min-w-0 flex-1 gap-1">
                    <Text class="truncate text-sm font-medium">
                      {message.name}
                    </Text>
                    <Text class="truncate text-xs text-[#777269]">
                      {message.company}
                    </Text>
                  </Column>
                  <View class="w-1.5 h-1.5 rounded-full bg-[#ae9367]" />
                </Row>
              </View>
            )}
          </ForValue>
          <Text class="mt-2 text-xs text-[#8d887e]">Saved for later</Text>
          <ForValue each={messages.slice(5)}>
            {(message) => (
              <Row class="w-full p-3 items-center gap-3">
                <Avatar initials={message.initials} color={message.color} />
                <Column class="min-w-0 flex-1 gap-1">
                  <Text class="truncate text-sm font-medium">
                    {message.name}
                  </Text>
                  <Text class="truncate text-xs text-[#777269]">
                    {message.company}
                  </Text>
                </Column>
              </Row>
            )}
          </ForValue>
        </Column>
      </Column>

      <Column class="min-w-0 h-full flex-1 bg-[#f5f2eb]">
        <Row class="h-14 px-5 flex-none items-center gap-3 border-b border-[#dedbd3]">
          <Text class="text-xs text-[#726e65]">Jules Morgan</Text>
          <View class="w-1.5 h-1.5 rounded-full bg-[#687cc5]" />
          <Text class="text-xs">Margin Press</Text>
          <View class="flex-1" />
          <Button
            variant="outline"
            size="icon"
            aria-label="Previous message"
            class="w-7 h-7 border-[#dedbd3]"
          >
            <Icon source={chevronUp} size={14} />
          </Button>
          <Button
            variant="outline"
            size="icon"
            aria-label="Next message"
            class="w-7 h-7 border-[#dedbd3]"
          >
            <Icon source={chevronDown} size={14} />
          </Button>
          <Button variant="outline" size="sm" class="border-[#dedbd3]">
            Dismiss
          </Button>
        </Row>
        <Column class="min-h-0 flex-1 overflow-y-scroll px-10 py-8 gap-5">
          <View class="w-full p-5 rounded-xl border border-[#e3ded4] bg-[#eee8de]">
            <Row class="items-center gap-3">
              <Icon source={sparkle} size={14} class="text-[#6f6a62]" />
              <Text class="text-xs font-medium">Draft context</Text>
              <View class="px-2 py-1 rounded border border-[#d9d3c8] bg-[#faf8f3]">
                <Text class="text-xs">Email draft</Text>
              </View>
            </Row>
            <Text class="mt-4 text-sm text-[#4b4842]">
              The catalogue layout is ready for review. Ask Jules to confirm the
              product order before the proof goes to print.
            </Text>
          </View>

          <Column class="w-full rounded-xl border border-[#dfdbd2] bg-[#fffefa] overflow-hidden">
            <Row class="h-11 px-5 items-center border-b border-[#e4e0d8]">
              <Text class="w-16 text-xs text-[#8a857c]">From</Text>
              <View class="w-5 h-5 rounded-full bg-[#d9d7fb] flex items-center justify-center">
                <Text class="text-xs">R</Text>
              </View>
              <Text class="ml-2 text-xs">robin@folio.example</Text>
            </Row>
            <Row class="h-11 px-5 items-center border-b border-[#e4e0d8]">
              <Text class="w-16 text-xs text-[#8a857c]">To</Text>
              <Avatar initials="JM" color="#efd9c8" />
              <Text class="ml-2 text-xs">jules@margin.example</Text>
            </Row>
            <Column class="px-7 py-6 gap-4">
              <Text class="text-base font-semibold">
                Your spring catalogue, on paper
              </Text>
              <Text class="text-sm">Hello Jules,</Text>
              <Text class="text-sm" style={{ "line-height": "22px" }}>
                The spring catalogue for Margin Press is taking shape. I've
                given each object a full page and kept the descriptions brief,
                so the materials and small details can do most of the talking.
              </Text>
              <Text class="text-sm" style={{ "line-height": "22px" }}>
                Could you confirm the product sequence by Friday? I'll then
                prepare a printed proof with the cover and two interior paper
                options.
              </Text>
              <Text class="text-sm">Warmly,</Text>
              <Text class="text-sm">Robin</Text>
              <Row class="mt-2 items-center">
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Attach file"
                  class="text-[#706b62]"
                >
                  <Icon source={paperclip} size={14} />
                </Button>
                <View class="flex-1" />
                <Row class="items-center gap-2 text-[#79746c]">
                  <Icon source={sparkle} size={14} />
                  <Text class="text-xs">Writing assistant</Text>
                </Row>
                <Button class="ml-4 h-8 px-3 gap-2">
                  <Text class="text-xs">Send email</Text>
                  <Icon source={send} size={13} />
                </Button>
              </Row>
            </Column>
          </Column>
          <Row class="items-center text-[#969086]">
            <Icon source={pen} size={12} />
            <Text class="ml-2 text-xs">
              A quieter place for your correspondence.
            </Text>
            <View class="flex-1" />
            <Text class="text-xs">FOLIO DESK</Text>
          </Row>
        </Column>
      </Column>
    </View>
  );
}
