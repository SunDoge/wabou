import {
  Alert,
  Button,
  CalendarDate,
  DatePicker,
  Select,
  Text,
  View,
} from "@wabou/ui";
import { createMemo, createSignal, Show } from "solid-js";
import { FieldLabel, TaskPage } from "../shared";

export function FlightBookerTask() {
  const [kind, setKind] = createSignal("one-way");
  const [depart, setDepart] = createSignal(new CalendarDate(2026, 9, 1));
  const [returnDate, setReturnDate] = createSignal(
    new CalendarDate(2026, 9, 8),
  );
  const [confirmation, setConfirmation] = createSignal("");
  const returnEnabled = () => kind() === "return";
  const valid = createMemo(
    () => !returnEnabled() || returnDate().compare(depart()) >= 0,
  );
  const book = () => {
    if (!valid()) return;
    setConfirmation(
      returnEnabled()
        ? `Return flight booked: ${depart()} to ${returnDate()}.`
        : `One-way flight booked for ${depart()}.`,
    );
  };
  return (
    <TaskPage
      number={3}
      title="Flight Booker"
      summary="Conditional controls and cross-field validation stay explicit and inspectable."
    >
      <View class="w-80 mx-auto flex flex-col gap-4">
        <View class="flex flex-col gap-2">
          <FieldLabel>Trip</FieldLabel>
          <Select
            aria-label="Trip type"
            class="w-full"
            options={[
              { value: "one-way", label: "One-way flight" },
              { value: "return", label: "Return flight" },
            ]}
            value={kind()}
            onValueChange={setKind}
          />
        </View>
        <View class="flex flex-col gap-2">
          <FieldLabel>Departure date</FieldLabel>
          <DatePicker
            aria-label="Departure date"
            value={depart()}
            onValueChange={setDepart}
          />
        </View>
        <View class="flex flex-col gap-2">
          <FieldLabel>Return date</FieldLabel>
          <DatePicker
            aria-label="Return date"
            disabled={!returnEnabled()}
            value={returnDate()}
            minValue={depart()}
            onValueChange={setReturnDate}
          />
        </View>
        <Show when={!valid()}>
          <Text role="alert" class="text-xs text-danger-primary">
            Return must not precede departure.
          </Text>
        </Show>
        <Button aria-label="Book flight" disabled={!valid()} onClick={book}>
          Book
        </Button>
        <Show when={confirmation()}>
          {(message) => (
            <Alert
              title="Booked"
              class="border-success-primary bg-success-surface"
            >
              {message()}
            </Alert>
          )}
        </Show>
      </View>
    </TaskPage>
  );
}
