import {
  Button,
  createFormDraft,
  createStandardSchemaValidator,
  Field,
  FieldError,
  FieldLabel,
  Input,
  mergeClasses,
  ScrollArea,
  Text,
  View,
} from "@wabou/ui";
import { Button as PrimitiveButton } from "@wabou/ui/primitives";
import { createMemo, createSignal, For as ForValue, Show } from "solid-js";
import { TaskPage } from "../shared";
import { personSchema } from "./validation";

interface Person {
  id: number;
  first: string;
  last: string;
}

export function CrudTask() {
  const [people, setPeople] = createSignal<Person[]>([
    { id: 1, first: "Hans", last: "Emil" },
    { id: 2, first: "Max", last: "Mustermann" },
    { id: 3, first: "Roman", last: "Tisch" },
  ]);
  const [filter, setFilter] = createSignal("");
  const [selected, setSelected] = createSignal<number | null>(null);
  const draft = createFormDraft(
    { first: "", last: "" },
    { validate: createStandardSchemaValidator(personSchema) },
  );
  const [validationVisible, setValidationVisible] = createSignal(false);
  let nextId = 4;
  const visible = createMemo(() => {
    const query = filter().trim().toLowerCase();
    return query
      ? people().filter((person) =>
          `${person.last}, ${person.first}`.toLowerCase().includes(query),
        )
      : people();
  });
  const choose = (person: Person) => {
    setSelected(person.id);
    draft.resetTo({ first: person.first, last: person.last });
    setValidationVisible(false);
  };
  const create = () => {
    setValidationVisible(true);
    if (!draft.valid()) return;
    const value = draft.value();
    setPeople((items) => [
      ...items,
      { id: nextId++, first: value.first.trim(), last: value.last.trim() },
    ]);
    draft.resetTo({ first: "", last: "" });
    setValidationVisible(false);
    setSelected(null);
  };
  const update = () => {
    const id = selected();
    setValidationVisible(true);
    if (id === null || !draft.valid()) return;
    const value = draft.value();
    setPeople((items) =>
      items.map((person) =>
        person.id === id
          ? { ...person, first: value.first.trim(), last: value.last.trim() }
          : person,
      ),
    );
    draft.commit();
    setValidationVisible(false);
  };
  const remove = () => {
    const id = selected();
    if (id === null) return;
    setPeople((items) => items.filter((person) => person.id !== id));
    setSelected(null);
    draft.resetTo({ first: "", last: "" });
    setValidationVisible(false);
  };
  return (
    <TaskPage
      number={5}
      title="CRUD"
      summary="Filtering, selection and mutations share one small explicit state model."
    >
      <View class="flex gap-5">
        <View class="w-72 flex flex-col gap-3">
          <View class="flex flex-col gap-2">
            <FieldLabel>Filter prefix</FieldLabel>
            <Input
              aria-label="Filter people"
              value={filter()}
              onInput={(e) => setFilter(e.currentTarget.value)}
            />
          </View>
          <ScrollArea
            class="h-64 rounded-lg border border-subtle bg-surface-muted"
            contentClass="p-2 gap-1"
          >
            <ForValue each={visible()}>
              {(person) => (
                <PrimitiveButton
                  unstyled
                  aria-label={`${person.last}, ${person.first}`}
                  selected={selected() === person.id}
                  class={(state) =>
                    mergeClasses(
                      "w-full h-9 px-3 justify-start rounded-md text-sm",
                      selected() === person.id
                        ? "bg-selected text-primary"
                        : state.hovered
                          ? "bg-control-hover text-primary"
                          : "bg-transparent text-secondary",
                    )
                  }
                  onClick={() => choose(person)}
                >
                  {person.last}, {person.first}
                </PrimitiveButton>
              )}
            </ForValue>
          </ScrollArea>
        </View>
        <View class="flex-1 flex flex-col gap-4">
          <Field
            invalid={validationVisible() && Boolean(draft.fieldError("first"))}
          >
            <FieldLabel>First name</FieldLabel>
            <Input
              aria-label="First name"
              aria-invalid={
                validationVisible() && Boolean(draft.fieldError("first"))
              }
              value={draft.field("first")}
              onInput={(e) => draft.set("first", e.currentTarget.value)}
            />
            <Show
              when={validationVisible() ? draft.fieldError("first") : undefined}
            >
              {(error) => <FieldError>{error()}</FieldError>}
            </Show>
          </Field>
          <Field
            invalid={validationVisible() && Boolean(draft.fieldError("last"))}
          >
            <FieldLabel>Surname</FieldLabel>
            <Input
              aria-label="Surname"
              aria-invalid={
                validationVisible() && Boolean(draft.fieldError("last"))
              }
              value={draft.field("last")}
              onInput={(e) => draft.set("last", e.currentTarget.value)}
            />
            <Show
              when={validationVisible() ? draft.fieldError("last") : undefined}
            >
              {(error) => <FieldError>{error()}</FieldError>}
            </Show>
          </Field>
          <View class="flex items-center gap-2">
            <Button aria-label="Create person" onClick={create}>
              Create
            </Button>
            <Button
              aria-label="Update person"
              variant="secondary"
              disabled={selected() === null}
              onClick={update}
            >
              Update
            </Button>
            <Button
              aria-label="Delete person"
              variant="destructive"
              disabled={selected() === null}
              onClick={remove}
            >
              Delete
            </Button>
          </View>
          <Text class="text-xs text-muted">
            {people().length} records · {visible().length} visible
          </Text>
        </View>
      </View>
    </TaskPage>
  );
}
