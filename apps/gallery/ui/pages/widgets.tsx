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
} from "@wabou/components";
import { Text, View } from "@wabou/primitives";
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
            <Text class="text-xl">◇</Text>
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
