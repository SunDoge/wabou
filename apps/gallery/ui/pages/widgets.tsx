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
  AspectRatio,
  Attachment,
  AttachmentAction,
  AttachmentActions,
  AttachmentContent,
  AttachmentDescription,
  AttachmentGroup,
  AttachmentMedia,
  AttachmentTitle,
  Avatar,
  AvatarGroup,
  AvatarGroupCount,
  Badge,
  Breadcrumb,
  BreadcrumbEllipsis,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
  Bubble,
  BubbleContent,
  BubbleGroup,
  BubbleReactions,
  Button,
  ButtonGroup,
  ButtonGroupSeparator,
  ButtonGroupText,
  CalendarDate,
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselNext,
  CarouselPrevious,
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
  Drawer,
  DrawerClose,
  DrawerDescription,
  DrawerFooter,
  DrawerHandle,
  DrawerHeader,
  DrawerTitle,
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
  FieldLegend,
  FieldSeparator,
  FieldSet,
  FieldTitle,
  HoverCard,
  Icon,
  Input,
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  InputGroupText,
  InputGroupTextArea,
  InputOTP,
  InputOTPGroup,
  InputOTPSeparator,
  InputOTPSlot,
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemMedia,
  ItemSeparator,
  ItemTitle,
  Marker,
  MarkerContent,
  Message,
  MessageAvatar,
  MessageContent,
  MessageFooter,
  MessageGroup,
  MessageHeader,
  MessageScroller,
  MessageScrollerButton,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerViewport,
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  Pagination,
  PaginationContent,
  PaginationItems,
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
  Spinner,
  Text,
  Toaster,
  Tooltip,
  View,
} from "@wabou/ui";
import file from "lucide-static/icons/file.svg?raw";
import rocket from "lucide-static/icons/rocket.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import triangleAlert from "lucide-static/icons/triangle-alert.svg?raw";
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

export function DrawerPage() {
  const [open, setOpen] = createSignal(false);
  const [status, setStatus] = createSignal("Drawer has not opened");
  return (
    <Preview title="Drag-to-dismiss drawer">
      <View class="flex flex-col items-start gap-3">
        <Drawer
          aria-label="Create task"
          open={open()}
          onOpenChange={(next, reason) => {
            setOpen(next);
            setStatus(next ? "Drawer opened" : `Drawer closed by ${reason}`);
          }}
          direction="bottom"
          contentClass="h-[320px]"
          trigger={(trigger) => <Button {...trigger}>Open drawer</Button>}
        >
          <DrawerHandle />
          <DrawerHeader>
            <DrawerTitle>Create a download task</DrawerTitle>
            <DrawerDescription>
              Drag the handle downward, press Escape, or use an explicit action.
            </DrawerDescription>
          </DrawerHeader>
          <View class="min-h-0 flex-1 px-5">
            <Input
              aria-label="Download URL"
              placeholder="https://example.com/archive.zip"
            />
          </View>
          <DrawerFooter>
            <Button onClick={() => setStatus("Task created")}>
              Create task
            </Button>
            <DrawerClose variant="outline">Cancel</DrawerClose>
          </DrawerFooter>
        </Drawer>
        <Text role="status" class="text-sm text-muted">
          {status()}
        </Text>
      </View>
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

export function HoverCardPage() {
  return (
    <Preview title="Delayed preview surface">
      <View class="flex items-start gap-4">
        <HoverCard
          aria-label="Wabou project preview"
          trigger={(trigger) => (
            <Button variant="outline" aria-label="Preview Wabou" {...trigger}>
              Preview Wabou
            </Button>
          )}
        >
          <View class="flex flex-col gap-2">
            <Text class="text-sm font-semibold text-primary">Wabou</Text>
            <Text
              role="status"
              aria-label="Project summary"
              class="whitespace-normal text-sm text-secondary"
            >
              Native desktop applications composed with Solid and rendered by
              Rust.
            </Text>
            <Badge variant="outline">Native UI</Badge>
          </View>
        </HoverCard>
        <Text class="text-sm text-muted">
          Hover the trigger, or focus it with the keyboard.
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
        <Pagination
          count={24}
          page={page()}
          onPageChange={setPage}
          aria-label="Results pages"
        >
          <PaginationContent>
            <PaginationPrevious aria-label="Previous page" />
            <PaginationItems />
            <PaginationNext aria-label="Next page" />
          </PaginationContent>
        </Pagination>
        <Text
          role="status"
          aria-label="Selected pagination page"
          class="text-sm text-secondary"
        >
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
        <Accordion
          class="w-[520px]"
          collapsible
          defaultValue="native"
          role="group"
          aria-label="Wabou questions"
        >
          <AccordionItem value="native">
            <AccordionTrigger>
              Is Wabou rendered by the browser?
            </AccordionTrigger>
            <AccordionContent role="region" aria-label="Native rendering">
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
            <AccordionContent role="region" aria-label="CSS support">
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
            <AccordionContent role="region" aria-label="Theme support">
              <Text class="w-full whitespace-normal text-sm text-muted">
                Yes. Semantic tokens allow runtime themes without coupling
                widgets to a palette.
              </Text>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </Preview>
      <Preview title="Collapsible">
        <Collapsible
          class="w-[520px] rounded-lg border border-subtle p-4"
          role="group"
          aria-label="Advanced settings"
        >
          <CollapsibleTrigger>
            <Text class="text-sm font-medium text-primary">
              Advanced options
            </Text>
          </CollapsibleTrigger>
          <CollapsibleContent
            class="pt-3"
            role="region"
            aria-label="Advanced options"
          >
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

export function AspectRatioPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Video 16:9">
        <AspectRatio
          ratio={16 / 9}
          class="w-[560px] rounded-lg border border-subtle bg-control shadow-xs"
        >
          <View class="w-full h-full flex items-center justify-center bg-control-hover">
            <View class="flex flex-col items-center gap-1">
              <Text class="text-lg font-semibold text-primary">16:9</Text>
              <Text class="text-sm text-muted">Native layout constraint</Text>
            </View>
          </View>
        </AspectRatio>
      </Preview>
      <Preview title="Square and 4:3">
        <AspectRatio class="w-40 rounded-lg border border-subtle bg-control">
          <View class="w-full h-full flex items-center justify-center">
            <Text class="font-medium text-secondary">1:1</Text>
          </View>
        </AspectRatio>
        <AspectRatio
          ratio={4 / 3}
          class="w-56 rounded-lg border border-subtle bg-control"
        >
          <View class="w-full h-full flex items-center justify-center">
            <Text class="font-medium text-secondary">4:3</Text>
          </View>
        </AspectRatio>
      </Preview>
    </View>
  );
}

export function AttachmentPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Transfer states">
        <AttachmentGroup class="w-[680px]">
          <Attachment state="done">
            <AttachmentMedia>
              <Icon source={file} aria-hidden="true" size={18} />
            </AttachmentMedia>
            <AttachmentContent>
              <AttachmentTitle>release-notes.pdf</AttachmentTitle>
              <AttachmentDescription>2.4 MB - Complete</AttachmentDescription>
            </AttachmentContent>
            <AttachmentActions>
              <AttachmentAction aria-label="Open release notes">
                Open
              </AttachmentAction>
            </AttachmentActions>
          </Attachment>
          <Attachment state="uploading">
            <AttachmentMedia>
              <Spinner label="Uploading design archive" />
            </AttachmentMedia>
            <AttachmentContent>
              <AttachmentTitle>design-assets.zip</AttachmentTitle>
              <AttachmentDescription>Uploading - 68%</AttachmentDescription>
            </AttachmentContent>
          </Attachment>
          <Attachment state="error">
            <AttachmentMedia>
              <Icon source={triangleAlert} aria-hidden="true" size={18} />
            </AttachmentMedia>
            <AttachmentContent>
              <AttachmentTitle>recording.mov</AttachmentTitle>
              <AttachmentDescription>Upload failed</AttachmentDescription>
            </AttachmentContent>
            <AttachmentActions>
              <AttachmentAction aria-label="Retry recording upload">
                Retry
              </AttachmentAction>
            </AttachmentActions>
          </Attachment>
        </AttachmentGroup>
      </Preview>
      <Preview title="Vertical media">
        <Attachment orientation="vertical" class="w-32">
          <AttachmentMedia>
            <Icon source={file} aria-hidden="true" size={28} />
          </AttachmentMedia>
          <AttachmentContent>
            <AttachmentTitle>preview.png</AttachmentTitle>
            <AttachmentDescription>1280 x 720</AttachmentDescription>
          </AttachmentContent>
        </Attachment>
      </Preview>
    </View>
  );
}

export function MessagePage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Conversation">
        <MessageGroup
          aria-label="Conversation preview"
          class="w-[640px] rounded-lg border border-subtle bg-surface p-5 shadow-xs"
        >
          <Marker variant="separator">
            <MarkerContent>Today</MarkerContent>
          </Marker>
          <Message>
            <MessageAvatar>
              <Avatar fallback="WA" size="sm" />
            </MessageAvatar>
            <MessageContent>
              <MessageHeader>Wabou team - 09:41</MessageHeader>
              <BubbleGroup>
                <Bubble variant="secondary">
                  <BubbleContent>
                    <Text class="whitespace-normal text-sm">
                      The native component capture is ready. It passed both
                      scale factors.
                    </Text>
                  </BubbleContent>
                </Bubble>
                <Bubble variant="outline">
                  <BubbleContent>
                    <Text class="whitespace-normal text-sm">
                      We can now reuse the same anatomy in chat and activity
                      views.
                    </Text>
                  </BubbleContent>
                  <BubbleReactions>
                    <Text class="text-xs text-secondary">2 likes</Text>
                  </BubbleReactions>
                </Bubble>
              </BubbleGroup>
              <MessageFooter>Delivered</MessageFooter>
            </MessageContent>
          </Message>
          <Message align="end">
            <MessageAvatar>
              <Avatar fallback="ME" size="sm" />
            </MessageAvatar>
            <MessageContent>
              <MessageHeader>You - 09:43</MessageHeader>
              <Bubble variant="default">
                <BubbleContent>
                  <Text class="whitespace-normal text-sm">
                    Great. Keep the state explicit and avoid hidden DOM
                    behavior.
                  </Text>
                </BubbleContent>
              </Bubble>
              <MessageFooter>Read</MessageFooter>
            </MessageContent>
          </Message>
          <Marker variant="border">
            <MarkerContent>1 unread message</MarkerContent>
          </Marker>
          <Message>
            <MessageContent>
              <Bubble variant="destructive">
                <BubbleContent>
                  <Text class="whitespace-normal text-sm">
                    Delivery failed. Retry from the action menu.
                  </Text>
                </BubbleContent>
              </Bubble>
            </MessageContent>
          </Message>
        </MessageGroup>
      </Preview>
    </View>
  );
}

export function MessageScrollerPage() {
  const messages = [
    "Download started.",
    "Metadata verified.",
    "Segment completed.",
    "Limit unchanged.",
    "Transfer halfway.",
    "Segments connected.",
    "Checksum verified.",
    "New event received.",
  ];
  const [messageCount, setMessageCount] = createSignal(messages.length - 1);
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Follow-at-end scroller">
        <View class="w-[640px] flex flex-col items-end gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={messageCount() === messages.length}
            onClick={() => setMessageCount(messages.length)}
          >
            Append message
          </Button>
          <MessageScroller
            aria-label="Scrollable conversation"
            class="w-full h-64 rounded-lg border border-subtle bg-surface shadow-xs"
          >
            <MessageScrollerViewport aria-label="Message history">
              <MessageScrollerContent class="p-4">
                {messages.slice(0, messageCount()).map((content, index) => (
                  <MessageScrollerItem>
                    <Message align={index % 2 === 0 ? "start" : "end"}>
                      <MessageContent>
                        <Bubble
                          class="w-3/5"
                          variant={index % 2 === 0 ? "secondary" : "default"}
                        >
                          <BubbleContent class="w-full">
                            <Text class="whitespace-normal text-sm">
                              {content}
                            </Text>
                          </BubbleContent>
                        </Bubble>
                      </MessageContent>
                    </Message>
                  </MessageScrollerItem>
                ))}
              </MessageScrollerContent>
            </MessageScrollerViewport>
            <MessageScrollerButton direction="start" />
            <MessageScrollerButton direction="end" />
          </MessageScroller>
        </View>
      </Preview>
    </View>
  );
}

export function InputOTPPage() {
  const [code, setCode] = createSignal("");
  const [completed, setCompleted] = createSignal(false);
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Native input with visual slots">
        <View class="flex flex-col items-start gap-3">
          <InputOTP
            aria-label="Verification code"
            value={code()}
            maxLength={6}
            onValueChange={(value) => {
              setCode(value);
              setCompleted(false);
            }}
            onComplete={() => setCompleted(true)}
          >
            <InputOTPGroup>
              <InputOTPSlot index={0} />
              <InputOTPSlot index={1} />
              <InputOTPSlot index={2} />
            </InputOTPGroup>
            <InputOTPSeparator />
            <InputOTPGroup>
              <InputOTPSlot index={3} />
              <InputOTPSlot index={4} />
              <InputOTPSlot index={5} />
            </InputOTPGroup>
          </InputOTP>
          <Text class="text-sm text-muted">
            {completed()
              ? `Code complete: ${code()}`
              : `Code: ${code() || "empty"}`}
          </Text>
        </View>
      </Preview>
    </View>
  );
}

export function NavigationMenuPage() {
  const [selection, setSelection] = createSignal("No destination selected");
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Shared navigation viewport">
        <View class="min-h-56 flex flex-col items-start gap-4">
          <NavigationMenu aria-label="Product navigation">
            <NavigationMenuList>
              <NavigationMenuItem value="products">
                <NavigationMenuTrigger aria-label="Products">
                  Products
                </NavigationMenuTrigger>
                <NavigationMenuContent class="grid grid-cols-2 gap-2">
                  <NavigationMenuLink
                    aria-label="Wabou Runtime"
                    onClick={() => setSelection("Wabou Runtime")}
                  >
                    <Text class="font-medium text-primary">Wabou Runtime</Text>
                    <Text class="whitespace-normal text-xs text-muted">
                      Native rendering and QuickJS application hosting.
                    </Text>
                  </NavigationMenuLink>
                  <NavigationMenuLink
                    aria-label="Wabou UI"
                    onClick={() => setSelection("Wabou UI")}
                  >
                    <Text class="font-medium text-primary">Wabou UI</Text>
                    <Text class="whitespace-normal text-xs text-muted">
                      Composable application components for Solid.
                    </Text>
                  </NavigationMenuLink>
                </NavigationMenuContent>
              </NavigationMenuItem>
              <NavigationMenuItem value="resources">
                <NavigationMenuTrigger aria-label="Resources">
                  Resources
                </NavigationMenuTrigger>
                <NavigationMenuContent>
                  <NavigationMenuLink
                    aria-label="Guides"
                    onClick={() => setSelection("Guides")}
                  >
                    <Text class="font-medium text-primary">Guides</Text>
                    <Text class="whitespace-normal text-xs text-muted">
                      Architecture, components and native integration.
                    </Text>
                  </NavigationMenuLink>
                  <NavigationMenuLink
                    aria-label="Examples"
                    onClick={() => setSelection("Examples")}
                  >
                    <Text class="font-medium text-primary">Examples</Text>
                    <Text class="whitespace-normal text-xs text-muted">
                      Complete desktop applications built with Wabou.
                    </Text>
                  </NavigationMenuLink>
                </NavigationMenuContent>
              </NavigationMenuItem>
            </NavigationMenuList>
          </NavigationMenu>
          <Text class="text-sm text-muted">{selection()}</Text>
        </View>
      </Preview>
    </View>
  );
}

export function CarouselPage() {
  const [index, setIndex] = createSignal(0);
  const [verticalIndex, setVerticalIndex] = createSignal(0);
  const slides = [
    {
      eyebrow: "Native rendering",
      title: "Retained scenes",
      description: "Update only the nodes whose state actually changed.",
      tone: "selected",
    },
    {
      eyebrow: "Solid 2",
      title: "Fine-grained UI",
      description:
        "Keep application state and native views in one reactive tree.",
      tone: "control",
    },
    {
      eyebrow: "Desktop input",
      title: "Captured dragging",
      description: "Pointer, keyboard and button navigation share one index.",
      tone: "success",
    },
  ] as const;
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Snapping carousel">
        <View class="w-[560px] flex flex-col items-center gap-3">
          <Carousel
            aria-label="Framework highlights"
            index={index()}
            onIndexChange={setIndex}
            class="w-full"
          >
            <CarouselContent
              role="group"
              aria-label="Feature slides"
              class="w-full h-52 rounded-xl border border-subtle bg-surface"
            >
              {slides.map((slide, slideIndex) => (
                <CarouselItem
                  aria-label={`Slide ${slideIndex + 1} of ${slides.length}`}
                >
                  <View
                    class="w-full h-full p-8 flex flex-col items-start justify-center gap-2"
                    classList={{
                      "bg-selected": slide.tone === "selected",
                      "bg-control": slide.tone === "control",
                      "bg-success-surface": slide.tone === "success",
                    }}
                  >
                    <Text class="text-xs font-medium text-muted">
                      {slide.eyebrow}
                    </Text>
                    <Text class="text-xl font-semibold text-primary">
                      {slide.title}
                    </Text>
                    <Text class="max-w-sm whitespace-normal text-sm text-secondary">
                      {slide.description}
                    </Text>
                  </View>
                </CarouselItem>
              ))}
            </CarouselContent>
            <View class="flex items-center gap-3">
              <CarouselPrevious />
              <Text role="status" class="w-24 text-center text-sm text-muted">
                {`Slide ${index() + 1} of ${slides.length}`}
              </Text>
              <CarouselNext />
            </View>
          </Carousel>
        </View>
      </Preview>
      <Preview title="Vertical orientation">
        <Carousel
          aria-label="Vertical highlights"
          orientation="vertical"
          index={verticalIndex()}
          onIndexChange={setVerticalIndex}
          class="w-[420px]"
        >
          <CarouselContent
            role="group"
            aria-label="Vertical feature slides"
            class="w-full h-40 rounded-xl border border-subtle"
          >
            {slides.map((slide, slideIndex) => (
              <CarouselItem
                aria-label={`Vertical slide ${slideIndex + 1} of ${slides.length}`}
              >
                <View class="w-full h-full p-6 flex flex-col items-start justify-center gap-1 bg-control">
                  <Text class="text-xs text-muted">{slide.eyebrow}</Text>
                  <Text class="text-lg font-semibold text-primary">
                    {slide.title}
                  </Text>
                </View>
              </CarouselItem>
            ))}
          </CarouselContent>
          <View class="flex items-center gap-3">
            <CarouselPrevious aria-label="Previous vertical slide" />
            <Text role="status" class="w-32 text-center text-sm text-muted">
              {`Vertical slide ${verticalIndex() + 1} of ${slides.length}`}
            </Text>
            <CarouselNext aria-label="Next vertical slide" />
          </View>
        </Carousel>
      </Preview>
    </View>
  );
}

export function FieldPage() {
  const [email, setEmail] = createSignal("");
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Form fields">
        <FieldSet class="w-[440px]">
          <FieldLegend>Workspace details</FieldLegend>
          <FieldGroup>
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
                <FieldError
                  errors={[
                    {
                      message:
                        "Use lowercase letters, numbers and hyphens only.",
                    },
                    {
                      message:
                        "Use lowercase letters, numbers and hyphens only.",
                    },
                  ]}
                />
              </FieldContent>
            </Field>
          </FieldGroup>
          <FieldSeparator>Optional</FieldSeparator>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldTitle>Project visibility</FieldTitle>
              <FieldDescription>
                Visibility is controlled by the application state.
              </FieldDescription>
            </FieldContent>
            <Badge variant="secondary">Private</Badge>
          </Field>
        </FieldSet>
      </Preview>
      <Preview title="Input group">
        <View class="w-[440px] flex flex-col gap-3">
          <InputGroup>
            <InputGroupAddon
              align="inline-start"
              aria-label="Focus project hostname"
            >
              <InputGroupText>https://</InputGroupText>
            </InputGroupAddon>
            <InputGroupInput
              aria-label="Project hostname"
              placeholder="example.com"
            />
            <InputGroupButton>Copy</InputGroupButton>
          </InputGroup>
          <InputGroup>
            <InputGroupAddon align="inline-start" class="px-2.5">
              <Icon source={search} aria-hidden="true" size={14} />
            </InputGroupAddon>
            <InputGroupInput placeholder="Search projects…" />
            <InputGroupAddon align="inline-end">
              <InputGroupText>Ctrl K</InputGroupText>
            </InputGroupAddon>
          </InputGroup>
          <InputGroup orientation="vertical">
            <InputGroupAddon align="block-start">
              <InputGroupText>Request headers</InputGroupText>
            </InputGroupAddon>
            <InputGroupTextArea
              aria-label="Request headers"
              placeholder="Authorization: Bearer …"
            />
            <InputGroupAddon align="block-end">
              <InputGroupText>One header per line</InputGroupText>
            </InputGroupAddon>
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
          <EmptyMedia variant="icon">
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

export function ItemPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Project list">
        <ItemGroup class="w-[560px] rounded-lg border border-subtle bg-surface shadow-xs">
          <Item>
            <ItemMedia variant="icon">
              <Icon source={rocket} aria-hidden="true" size={16} />
            </ItemMedia>
            <ItemContent>
              <ItemTitle>Wabou Gallery</ItemTitle>
              <ItemDescription>
                Native component fixtures rendered through the Rust host.
              </ItemDescription>
            </ItemContent>
            <ItemActions>
              <Badge variant="success">Running</Badge>
              <Button size="sm" variant="outline">
                Open
              </Button>
            </ItemActions>
          </Item>
          <ItemSeparator />
          <Item size="sm" variant="muted">
            <ItemContent>
              <ItemTitle>Component contract checks</ItemTitle>
              <ItemDescription>
                Surface, focus and overlay ownership are validated before
                capture.
              </ItemDescription>
            </ItemContent>
            <ItemActions>
              <Badge variant="outline">18 checks</Badge>
            </ItemActions>
          </Item>
        </ItemGroup>
      </Preview>
      <Preview title="Standalone outline">
        <Item class="w-[440px]" variant="outline">
          <ItemContent>
            <ItemTitle>Download complete</ItemTitle>
            <ItemDescription>
              The row remains shrinkable and truncates long descriptions at two
              lines.
            </ItemDescription>
          </ItemContent>
          <ItemActions>
            <Button size="sm">Reveal</Button>
          </ItemActions>
        </Item>
      </Preview>
    </View>
  );
}

export function ButtonGroupPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Toolbar">
        <ButtonGroup>
          <Button variant="outline">Back</Button>
          <ButtonGroupSeparator />
          <Button variant="outline">Forward</Button>
          <ButtonGroupSeparator />
          <Button variant="outline">Reload</Button>
          <ButtonGroupSeparator />
          <ButtonGroupText>3 selected</ButtonGroupText>
          <ButtonGroupSeparator />
          <Button variant="destructive">Delete</Button>
        </ButtonGroup>
      </Preview>
      <Preview title="Vertical">
        <ButtonGroup orientation="vertical" class="w-40">
          <Button variant="outline">Profile</Button>
          <ButtonGroupSeparator orientation="horizontal" />
          <Button variant="outline">Settings</Button>
          <ButtonGroupSeparator orientation="horizontal" />
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
