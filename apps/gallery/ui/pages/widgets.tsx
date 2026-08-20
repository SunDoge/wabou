import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Avatar,
  AvatarGroup,
  AvatarGroupCount,
  Breadcrumb,
  BreadcrumbEllipsis,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
  Button,
  ButtonGroup,
  ButtonGroupText,
  CalendarDate,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  Combobox,
  Command,
  ContextMenu,
  createToasts,
  DatePicker,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DropdownMenu,
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
  Icon,
  Input,
  InputGroup,
  InputGroupButton,
  InputGroupInput,
  InputGroupText,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  Popover,
  PopoverDescription,
  PopoverFooter,
  PopoverHeader,
  PopoverTitle,
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
  Select,
  Sheet,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  Text,
  Toaster,
  Tooltip,
  View,
} from "@wabou/ui";
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

export function AlertDialogPage() {
  const [result, setResult] = createSignal("No decision yet");
  return (
    <Preview title="Explicit confirmation">
      <View class="flex flex-col items-start gap-4">
        <AlertDialog
          aria-label="Delete deployment"
          trigger={(trigger) => (
            <Button variant="destructive" {...trigger}>
              Delete deployment
            </Button>
          )}
        >
          <AlertDialogHeader>
            <AlertDialogTitle>Delete this deployment?</AlertDialogTitle>
            <AlertDialogDescription>
              Its logs and generated artifacts will be permanently removed. This
              action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setResult("Cancelled")}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => setResult("Deployment deleted")}
            >
              Delete deployment
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialog>
        <Text role="status" class="text-sm text-secondary">
          {result()}
        </Text>
      </View>
    </Preview>
  );
}

export function SheetPage() {
  return (
    <Preview title="Modal edge panel">
      <Sheet
        aria-label="Edit profile"
        trigger={(trigger) => <Button {...trigger}>Open sheet</Button>}
      >
        {(controls) => (
          <>
            <SheetHeader>
              <SheetTitle>Edit profile</SheetTitle>
              <SheetDescription>
                Update settings without leaving the current page.
              </SheetDescription>
            </SheetHeader>
            <View class="min-h-0 flex-1 py-4">
              <Field>
                <FieldLabel>Name</FieldLabel>
                <Input aria-label="Profile name" value="Wabou" />
              </Field>
            </View>
            <SheetFooter>
              <Button variant="outline" onClick={controls.close}>
                Cancel
              </Button>
              <Button onClick={controls.close}>Save changes</Button>
            </SheetFooter>
          </>
        )}
      </Sheet>
    </Preview>
  );
}

export function ToastPage() {
  const toasts = createToasts();
  return (
    <>
      <Preview title="Transient feedback">
        <View class="flex flex-wrap items-center gap-3">
          <Button
            onClick={() =>
              toasts.success("Project saved", {
                description: "Your changes were written to disk.",
              })
            }
          >
            Success toast
          </Button>
          <Button
            variant="outline"
            onClick={() =>
              toasts.error("Download failed", {
                description: "The connection closed before completion.",
                action: { label: "Retry", onAction: () => {} },
              })
            }
          >
            Action toast
          </Button>
        </View>
      </Preview>
      <Toaster toasts={toasts} />
    </>
  );
}

export function TooltipPage() {
  return (
    <Preview title="Pointer and keyboard hints">
      <View class="flex items-center gap-3">
        <Tooltip
          trigger={(trigger) => (
            <Button variant="outline" {...trigger}>
              Hover or focus
            </Button>
          )}
        >
          Opens after a short pointer delay and immediately on keyboard focus.
        </Tooltip>
        <Tooltip
          placement="bottom"
          openDelay={0}
          trigger={(trigger) => (
            <Button variant="ghost" {...trigger}>
              Immediate hint
            </Button>
          )}
        >
          Placement is resolved from the completed native layout.
        </Tooltip>
      </View>
    </Preview>
  );
}

export function PopoverPage() {
  const [saved, setSaved] = createSignal("No changes saved");
  return (
    <Preview title="Composed floating panel">
      <View class="flex flex-col items-start gap-4">
        <Popover
          aria-label="Workspace options"
          placement="bottom-start"
          contentClass="w-72"
          trigger={(trigger) => (
            <Button variant="outline" {...trigger}>
              Workspace options
            </Button>
          )}
        >
          <PopoverHeader>
            <PopoverTitle>Workspace options</PopoverTitle>
            <PopoverDescription>
              Configure this workspace without leaving the current view.
            </PopoverDescription>
          </PopoverHeader>
          <Field>
            <FieldLabel>Display name</FieldLabel>
            <Input aria-label="Workspace display name" value="Wabou" />
          </Field>
          <PopoverFooter>
            <Button size="sm" onClick={() => setSaved("Workspace saved")}>
              Save
            </Button>
          </PopoverFooter>
        </Popover>
        <Text role="status" class="text-sm text-secondary">
          {saved()}
        </Text>
      </View>
    </Preview>
  );
}

const workspacePanels = [
  { id: "navigation", defaultSize: 32, minSize: 20, maxSize: 60 },
  { id: "content", defaultSize: 68, minSize: 40, maxSize: 80 },
] as const;

export function ResizablePage() {
  const [sizes, setSizes] = createSignal({ navigation: 32, content: 68 });
  return (
    <Preview title="Explicit split-panel composition">
      <View class="w-full flex flex-col gap-4">
        <ResizablePanelGroup
          panels={workspacePanels}
          value={sizes()}
          onValueChange={(next) =>
            setSizes({
              navigation: next.navigation,
              content: next.content,
            })
          }
          aria-label="Workspace panels"
          class="h-64 rounded-lg border border-subtle bg-surface"
        >
          <ResizablePanel id="navigation" class="p-4 bg-surface-muted">
            <View class="flex flex-col gap-2">
              <Text class="font-semibold">Navigation</Text>
              <Text maxLines={1} class="text-sm text-secondary">
                Drag or use arrow keys.
              </Text>
            </View>
          </ResizablePanel>
          <ResizableHandle
            before="navigation"
            after="content"
            aria-label="Resize navigation panel"
          />
          <ResizablePanel id="content" class="p-4">
            <View class="flex flex-col gap-2">
              <Text class="font-semibold">Content</Text>
              <Text class="text-sm text-secondary">
                Both panels keep explicit min and max constraints.
              </Text>
            </View>
          </ResizablePanel>
        </ResizablePanelGroup>
        <Text
          role="status"
          aria-label="Panel sizes"
          class="text-sm text-secondary"
        >
          {`${Math.round(sizes().navigation)}% navigation · ${Math.round(sizes().content)}% content`}
        </Text>
      </View>
    </Preview>
  );
}

export function BreadcrumbPageDemo() {
  const [destination, setDestination] = createSignal("Current page");
  return (
    <Preview title="Application-owned navigation">
      <View class="flex flex-col gap-4">
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem>
              <BreadcrumbLink onClick={() => setDestination("Workspace")}>
                Workspace
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
              <BreadcrumbEllipsis />
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
              <BreadcrumbLink onClick={() => setDestination("Components")}>
                Components
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
              <BreadcrumbPage>Breadcrumb</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
        <Text role="status" class="text-sm text-secondary">
          {destination()}
        </Text>
      </View>
    </Preview>
  );
}

export function PaginationPage() {
  const [page, setPage] = createSignal(2);
  return (
    <Preview title="Controlled page navigation">
      <View class="flex flex-col gap-4">
        <Pagination aria-label={`Page ${page()}`}>
          <PaginationContent>
            <PaginationItem>
              <PaginationPrevious
                disabled={page() === 1}
                onClick={() => setPage((value) => Math.max(1, value - 1))}
              />
            </PaginationItem>
            {[1, 2, 3, 4, 5].map((value) => (
              <PaginationItem>
                <PaginationLink
                  aria-label={`Page ${value}`}
                  active={page() === value}
                  onClick={() => setPage(value)}
                >
                  {String(value)}
                </PaginationLink>
              </PaginationItem>
            ))}
            <PaginationItem>
              <PaginationNext
                disabled={page() === 5}
                onClick={() => setPage((value) => Math.min(5, value + 1))}
              />
            </PaginationItem>
          </PaginationContent>
        </Pagination>
        <Text role="status" class="text-sm text-secondary">
          {`Selected page ${page()}`}
        </Text>
      </View>
    </Preview>
  );
}

export function DropdownMenuPage() {
  const [action, setAction] = createSignal("No action selected");
  return (
    <Preview title="Keyboard and pointer actions">
      <View class="flex flex-col items-start gap-4">
        <DropdownMenu
          aria-label="Project actions"
          onAction={setAction}
          items={[
            {
              id: "open",
              label: "Open project",
              description: "Open in the current window",
            },
            { id: "rename", label: "Rename project" },
            { id: "archive", label: "Archive project", disabled: true },
            {
              id: "delete",
              label: "Delete project",
              destructive: true,
              separatorBefore: true,
            },
          ]}
          trigger={(trigger) => (
            <Button variant="outline" {...trigger}>
              Project actions
            </Button>
          )}
        />
        <Text role="status" class="text-sm text-secondary">
          {action()}
        </Text>
      </View>
    </Preview>
  );
}

export function ContextMenuPage() {
  const [action, setAction] = createSignal("Right-click the target");
  return (
    <Preview title="Secondary-click actions">
      <View class="flex flex-col items-start gap-4">
        <ContextMenu
          aria-label="Canvas actions"
          onAction={setAction}
          items={[
            { id: "copy", label: "Copy" },
            { id: "duplicate", label: "Duplicate" },
            { id: "locked", label: "Locked action", disabled: true },
            {
              id: "delete",
              label: "Delete",
              destructive: true,
              separatorBefore: true,
            },
          ]}
          trigger={(trigger) => (
            <Button variant="outline" class="w-72 h-24" {...trigger}>
              Right-click this target
            </Button>
          )}
        />
        <Text role="status" class="text-sm text-secondary">
          {action()}
        </Text>
      </View>
    </Preview>
  );
}

export function CommandPage() {
  const [action, setAction] = createSignal("No command selected");
  return (
    <Preview title="Searchable command list">
      <View class="w-96 flex flex-col gap-3 rounded-lg border border-subtle bg-surface p-3 shadow-sm">
        <Command
          aria-label="Project commands"
          placeholder="Search commands"
          onAction={setAction}
          items={[
            {
              id: "open",
              label: "Open project",
              description: "Open a local Wabou project",
              keywords: ["folder", "workspace"],
            },
            {
              id: "theme",
              label: "Change theme",
              description: "Select a light or dark appearance",
              keywords: ["appearance", "dark", "light"],
            },
            { id: "admin", label: "Admin tools", disabled: true },
          ]}
        />
        <Text role="status" class="text-sm text-secondary">
          {action()}
        </Text>
      </View>
    </Preview>
  );
}

export function ComboboxPage() {
  const [value, setValue] = createSignal("Not selected");
  return (
    <Preview title="Searchable selection">
      <View class="flex flex-col items-start gap-4">
        <Combobox
          aria-label="Technology"
          placeholder="Choose technology"
          searchPlaceholder="Search technologies"
          onValueChange={setValue}
          options={[
            {
              id: "solid",
              value: "solid",
              label: "SolidJS",
              keywords: ["signals"],
            },
            { id: "rust", value: "rust", label: "Rust", keywords: ["native"] },
            { id: "quickjs", value: "quickjs", label: "QuickJS" },
            {
              id: "future",
              value: "future",
              label: "Future option",
              disabled: true,
            },
          ]}
        />
        <Text role="status" class="text-sm text-secondary">
          {value()}
        </Text>
      </View>
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
