import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
  Avatar,
  AvatarGroup,
  AvatarGroupCount,
  Button,
  ButtonGroup,
  ButtonGroupText,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  CalendarDate,
  DatePicker,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
  Field,
  FieldContent,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
  Input,
  InputGroup,
  InputGroupButton,
  InputGroupInput,
  InputGroupText,
  Select,
} from "@wabou/components";
import { Icon, Text, View } from "@wabou/primitives";
import rocket from "lucide-static/icons/rocket.svg?raw";
import { createSignal } from "solid-js";
import { Preview } from "../preview";

export function DialogPage() {
  return (
    <Preview title="Confirmation dialog">
      <Dialog
        aria-label="Delete project"
        trigger={(trigger) => (
          <Button variant="destructive" {...trigger}>
            Delete project
          </Button>
        )}
      >
        {(dialog) => (
          <>
            <DialogHeader>
              <DialogTitle>Delete this project?</DialogTitle>
              <DialogDescription>
                This permanently removes the project and all of its deployment
                history. This action cannot be undone.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant="ghost" onClick={dialog.close}>
                Cancel
              </Button>
              <Button variant="destructive" onClick={dialog.close}>
                Delete
              </Button>
            </DialogFooter>
          </>
        )}
      </Dialog>
    </Preview>
  );
}

export function AccordionPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Single selection">
        <Accordion class="w-[520px]" collapsible defaultValue="native">
          <AccordionItem value="native">
            <AccordionTrigger>
              Is Wabou rendered by the browser?
            </AccordionTrigger>
            <AccordionContent>
              <Text class="w-full whitespace-normal text-sm text-muted">
                No. Solid produces a native scene graph rendered by Rust and
                Vello.
              </Text>
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="css">
            <AccordionTrigger>
              Does it support arbitrary browser CSS?
            </AccordionTrigger>
            <AccordionContent>
              <Text class="w-full whitespace-normal text-sm text-muted">
                No. Utilities compile to explicit native style values with
                predictable semantics.
              </Text>
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="themes">
            <AccordionTrigger>
              Can applications provide multiple themes?
            </AccordionTrigger>
            <AccordionContent>
              <Text class="w-full whitespace-normal text-sm text-muted">
                Yes. Semantic tokens allow runtime themes without coupling
                widgets to a palette.
              </Text>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </Preview>
      <Preview title="Collapsible">
        <Collapsible class="w-[520px] rounded-lg border border-subtle p-4">
          <CollapsibleTrigger>
            <Text class="text-sm font-medium text-primary">
              Advanced options
            </Text>
          </CollapsibleTrigger>
          <CollapsibleContent class="pt-3">
            <Text class="text-sm text-muted">
              Tracing and renderer diagnostics are available here.
            </Text>
          </CollapsibleContent>
        </Collapsible>
      </Preview>
    </View>
  );
}

export function AvatarPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Sizes">
        <Avatar fallback="WA" size="sm" />
        <Avatar fallback="SO" />
        <Avatar fallback="UI" size="lg" />
      </Preview>
      <Preview title="Group">
        <AvatarGroup>
          <Avatar fallback="AL" />
          <Avatar fallback="BM" />
          <Avatar fallback="CK" />
          <AvatarGroupCount>+5</AvatarGroupCount>
        </AvatarGroup>
      </Preview>
    </View>
  );
}

export function FieldPage() {
  const [email, setEmail] = createSignal("");
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Form fields">
        <FieldGroup class="w-[440px]">
          <Field>
            <FieldLabel>Email address</FieldLabel>
            <FieldContent>
              <Input
                value={email()}
                placeholder="you@example.com"
                onInput={(event) => setEmail(event.currentTarget.value)}
              />
              <FieldDescription>
                Used for security alerts and account recovery.
              </FieldDescription>
            </FieldContent>
          </Field>
          <Field invalid>
            <FieldLabel>Workspace slug</FieldLabel>
            <FieldContent>
              <Input value="my workspace" />
              <FieldError>
                Use lowercase letters, numbers and hyphens only.
              </FieldError>
            </FieldContent>
          </Field>
        </FieldGroup>
      </Preview>
      <Preview title="Input group">
        <View class="w-[440px] flex flex-col gap-3">
          <InputGroup>
            <InputGroupText>https://</InputGroupText>
            <InputGroupInput placeholder="example.com" />
            <InputGroupButton>Copy</InputGroupButton>
          </InputGroup>
          <InputGroup>
            <InputGroupText>⌕</InputGroupText>
            <InputGroupInput placeholder="Search projects…" />
            <InputGroupText>⌘ K</InputGroupText>
          </InputGroup>
        </View>
      </Preview>
    </View>
  );
}

export function EmptyPage() {
  return (
    <Preview title="Empty state">
      <Empty class="w-[560px]">
        <EmptyHeader>
          <EmptyMedia>
            <Icon source={rocket} aria-hidden="true" size={24} />
          </EmptyMedia>
          <EmptyTitle>No deployments yet</EmptyTitle>
          <EmptyDescription>
            Create a deployment to preview your application on another machine
            and share it with your team.
          </EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
          <Button>Create deployment</Button>
          <Button variant="outline">Documentation</Button>
        </EmptyContent>
      </Empty>
    </Preview>
  );
}

export function ButtonGroupPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Toolbar">
        <ButtonGroup>
          <Button variant="outline">Back</Button>
          <Button variant="outline">Forward</Button>
          <Button variant="outline">Reload</Button>
          <ButtonGroupText>3 selected</ButtonGroupText>
          <Button variant="destructive">Delete</Button>
        </ButtonGroup>
      </Preview>
      <Preview title="Vertical">
        <ButtonGroup orientation="vertical" class="w-40">
          <Button variant="outline">Profile</Button>
          <Button variant="outline">Settings</Button>
          <Button variant="outline">Sign out</Button>
        </ButtonGroup>
      </Preview>
    </View>
  );
}

export function SelectPage() {
  const [framework, setFramework] = createSignal<string>();
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Framework">
        <View class="w-[440px] flex flex-col gap-2">
          <Text class="text-sm font-medium text-primary">UI framework</Text>
          <Select
            aria-label="UI framework"
            value={framework()}
            onValueChange={setFramework}
            placeholder="Select a framework"
            options={[
              { value: "solid", label: "SolidJS" },
              { value: "react", label: "React" },
              { value: "vue", label: "Vue" },
              { value: "svelte", label: "Svelte", disabled: true },
            ]}
          />
          <Text class="text-xs text-muted">Selected: {framework() ?? "—"}</Text>
        </View>
      </Preview>
      <Preview title="Default value">
        <Select
          aria-label="Deployment region"
          defaultValue="hkg"
          options={[
            { value: "hkg", label: "Hong Kong" },
            { value: "nrt", label: "Tokyo" },
            { value: "fra", label: "Frankfurt" },
          ]}
        />
      </Preview>
      <Preview title="Long list and labels">
        <Select
          aria-label="Project"
          placeholder="Select a project"
          options={Array.from({ length: 16 }, (_, index) => ({
            value: `project-${index + 1}`,
            label:
              index === 7
                ? "Project 8 — a deliberately long label that should be truncated"
                : `Project ${index + 1}`,
          }))}
        />
      </Preview>
    </View>
  );
}

export function DatePickerPage() {
  const [date, setDate] = createSignal(new CalendarDate(2026, 8, 17));
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Date picker">
        <View class="w-[440px] flex flex-col gap-2">
          <Text class="text-sm font-medium text-primary">Deployment date</Text>
          <DatePicker
            aria-label="Deployment date"
            value={date()}
            minValue={new CalendarDate(2026, 1, 1)}
            maxValue={new CalendarDate(2027, 12, 31)}
            onValueChange={setDate}
          />
          <Text
            role="status"
            aria-label="Selected date"
            class="text-xs text-muted"
          >
            {date().toString()}
          </Text>
        </View>
      </Preview>
      <Preview title="Unavailable dates">
        <DatePicker
          aria-label="Weekday appointment"
          placeholder="Choose a weekday"
          isDateUnavailable={(value) => {
            const day = value.toDate("UTC").getUTCDay();
            return day === 0 || day === 6;
          }}
        />
      </Preview>
      <Preview title="Localized calendar">
        <DatePicker
          aria-label="本地化日期"
          locale="zh-CN"
          defaultValue={new CalendarDate(2026, 8, 17)}
          labels={{
            previousMonth: "上个月",
            nextMonth: "下个月",
            today: "今天",
            selectToday: "选择今天",
          }}
        />
      </Preview>
    </View>
  );
}
