import {
  Button,
  Column,
  Icon,
  Row,
  Svg,
  Text,
  useNavigate,
  View,
} from "@wabou/ui";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import fileUp from "lucide-static/icons/file-up.svg?raw";
import mousePointer from "lucide-static/icons/mouse-pointer-2.svg?raw";
import penLine from "lucide-static/icons/pen-line.svg?raw";
import save from "lucide-static/icons/save.svg?raw";
import textCursor from "lucide-static/icons/text-cursor-input.svg?raw";
import trash from "lucide-static/icons/trash-2.svg?raw";

const signature = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 240 70"><path d="M5 43c16-35 3 18 22-12s-6 31 17 2 3 29 26-3c18-25 3 32 19 5 18-28 9 23 30-2 13-15 4 16 24 1 18-14 11 16 32-3 13-12 8 13 28 0" fill="none" stroke="#3f51a2" stroke-width="2" stroke-linecap="round"/><path d="M96 48c20 5 47 5 79 0" fill="none" stroke="#3f51a2" stroke-width="1.5"/></svg>`;

function ContractPage() {
  return (
    <Column class="w-[680px] h-[820px] flex-none px-20 py-20 bg-white shadow-lg text-black">
      <Text class="w-full text-center text-xl font-semibold">
        EQUIPMENT LOAN AGREEMENT
      </Text>
      <Text class="mt-10 text-xs" style={{ "line-height": "18px" }}>
        This agreement is made between Northwind Studio (“Lender”) and the
        person signing below (“Borrower”) for the loan of one laptop computer,
        serial number NW-2026-0417, together with its charger.
      </Text>
      <Text class="mt-4 text-xs font-semibold">1. Term.</Text>
      <Text class="mt-1 text-xs" style={{ "line-height": "18px" }}>
        The equipment is lent from the date of signing until December 31, 2026,
        and shall be returned on request or at the end of the term.
      </Text>
      <Text class="mt-3 text-xs font-semibold">2. Care.</Text>
      <Text class="mt-1 text-xs" style={{ "line-height": "18px" }}>
        Borrower shall keep the equipment in good condition and report any loss
        or damage within two business days.
      </Text>
      <Text class="mt-3 text-xs font-semibold">3. Return.</Text>
      <Text class="mt-1 text-xs" style={{ "line-height": "18px" }}>
        Borrower shall return the equipment in the condition received,
        reasonable wear excepted.
      </Text>
      <View class="flex-1" />
      <Row class="w-full gap-10">
        <Column class="flex-1">
          <View class="h-px bg-[#777777]" />
          <Text class="mt-2 text-xs">Borrower signature</Text>
        </Column>
        <Column class="w-40">
          <View class="h-px bg-[#777777]" />
          <Text class="mt-2 text-xs">Date</Text>
        </Column>
      </Row>
      <Row class="w-full mt-10 gap-10">
        <Column class="flex-1 relative">
          <View class="h-px bg-[#777777]" />
          <Svg
            source={signature}
            aria-label="Signature"
            class="absolute w-48 h-14 left-12 -top-9"
          />
          <Text class="mt-2 text-xs">Borrower printed name</Text>
        </Column>
        <Column class="w-40">
          <View class="h-px bg-[#777777]" />
          <Text class="mt-2 text-xs">10/13/2026</Text>
        </Column>
      </Row>
      <Row class="w-full mt-10 gap-10">
        <Column class="flex-1">
          <View class="h-px bg-[#777777]" />
          <Text class="mt-2 text-xs">Lender signature</Text>
        </Column>
        <Column class="w-40">
          <View class="h-px bg-[#777777]" />
          <Text class="mt-2 text-xs">Date</Text>
        </Column>
      </Row>
    </Column>
  );
}

export function Signed() {
  const navigate = useNavigate();
  return (
    <Column
      role="group"
      aria-label="Signed PDF workspace"
      class="w-full h-full overflow-hidden bg-[#ededed] text-[#222222] font-sans"
    >
      <Row class="h-14 px-3 flex-none items-center gap-3 border-b border-[#d5d5d5] bg-[#fafafa]">
        <Button
          variant="ghost"
          size="icon"
          aria-label="Back to benchmarks"
          class="w-8 h-8 bg-[#e8e8e6]"
          onClick={() => navigate({ to: "/" })}
        >
          <Icon source={penLine} size={15} />
        </Button>
        <Text class="text-sm font-medium">sample-agreement.pdf</Text>
        <View class="w-1.5 h-1.5 rounded-full bg-[#e2a62a]" />
        <View class="flex-1" />
        <Row class="h-9 items-center rounded-lg border border-[#dddddd] bg-[#eeeeed]">
          <Button variant="ghost" class="h-8 px-3 gap-2 rounded-none">
            <Icon source={penLine} size={14} />
            Signature
          </Button>
          <View class="w-px h-5 bg-[#d2d2d2]" />
          <Button
            variant="ghost"
            size="icon"
            aria-label="Add text"
            class="rounded-none"
          >
            <Icon source={textCursor} size={14} />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Select"
            class="rounded-none"
          >
            <Icon source={mousePointer} size={14} />
          </Button>
        </Row>
        <Button variant="outline" class="h-8 px-3 gap-2">
          <Icon source={fileUp} size={14} />
          Open
          <Icon source={chevronDown} size={12} />
        </Button>
        <Button variant="outline" class="h-8 px-3 gap-2">
          <Icon source={save} size={14} />
          Save
          <Icon source={chevronDown} size={12} />
        </Button>
      </Row>
      <Row class="min-h-0 flex-1">
        <Column class="w-24 h-full flex-none items-center pt-5 border-r border-[#d2d2d2] bg-[#e6e6e6]">
          <View class="w-16 h-24 p-1 rounded border-2 border-[#4b88ec] bg-white">
            <Column class="w-full h-full px-2 py-3 gap-1">
              <View class="h-1 bg-[#bbbbbb]" />
              <View class="h-px bg-[#dddddd]" />
              <View class="h-px bg-[#dddddd]" />
              <View class="flex-1" />
              <View class="h-px bg-[#bbbbbb]" />
              <View class="h-px bg-[#bbbbbb]" />
            </Column>
          </View>
          <Text class="mt-2 text-xs">1</Text>
          <View class="flex-1" />
          <Button
            variant="ghost"
            size="icon"
            aria-label="Delete page"
            class="mb-3 text-[#888888]"
          >
            <Icon source={trash} size={14} />
          </Button>
        </Column>
        <View class="min-w-0 min-h-0 flex-1 overflow-scroll bg-[#dcdcdc]">
          <Row class="min-w-full min-h-full p-8 items-start justify-center">
            <ContractPage />
          </Row>
        </View>
      </Row>
      <Row class="h-7 px-3 flex-none items-center border-t border-[#d5d5d5] bg-[#fafafa]">
        <Text class="text-xs text-[#777777]">Page 1 of 1</Text>
        <View class="flex-1" />
        <Text class="text-xs text-[#777777]">?</Text>
      </Row>
    </Column>
  );
}
