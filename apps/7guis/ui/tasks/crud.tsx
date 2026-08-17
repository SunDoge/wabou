import { Button, Input } from "@wabou/components";
import {
  Button as PrimitiveButton,
  ScrollArea,
  Text,
  View,
} from "@wabou/primitives";
import { createMemo, createSignal, For } from "solid-js";
import { FieldLabel, TaskPage } from "../shared";

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
  const [first, setFirst] = createSignal("");
  const [last, setLast] = createSignal("");
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
    setFirst(person.first);
    setLast(person.last);
  };
  const create = () => {
    if (!first().trim() || !last().trim()) return;
    setPeople((items) => [
      ...items,
      { id: nextId++, first: first().trim(), last: last().trim() },
    ]);
    setFirst("");
    setLast("");
    setSelected(null);
  };
  const update = () => {
    const id = selected();
    if (id === null || !first().trim() || !last().trim()) return;
    setPeople((items) =>
      items.map((person) =>
        person.id === id
          ? { ...person, first: first().trim(), last: last().trim() }
          : person,
      ),
    );
  };
  const remove = () => {
    const id = selected();
    if (id === null) return;
    setPeople((items) => items.filter((person) => person.id !== id));
    setSelected(null);
    setFirst("");
    setLast("");
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
            <For each={visible()}>
              {(person) => (
                <PrimitiveButton
                  unstyled
                  aria-label={`${person.last}, ${person.first}`}
                  selected={selected() === person.id}
                  class="w-full h-9 px-3 justify-start rounded-md text-sm"
                  style={(state) => ({
                    "background-color":
                      selected() === person.id
                        ? "#15395d"
                        : state.hovered
                          ? "#202b3b"
                          : "transparent",
                    color: selected() === person.id ? "#e0f2fe" : "#c4cfdd",
                  })}
                  onClick={() => choose(person)}
                >
                  {person.last}, {person.first}
                </PrimitiveButton>
              )}
            </For>
          </ScrollArea>
        </View>
        <View class="flex-1 flex flex-col gap-4">
          <View class="flex flex-col gap-2">
            <FieldLabel>First name</FieldLabel>
            <Input
              aria-label="First name"
              value={first()}
              onInput={(e) => setFirst(e.currentTarget.value)}
            />
          </View>
          <View class="flex flex-col gap-2">
            <FieldLabel>Surname</FieldLabel>
            <Input
              aria-label="Surname"
              value={last()}
              onInput={(e) => setLast(e.currentTarget.value)}
            />
          </View>
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
