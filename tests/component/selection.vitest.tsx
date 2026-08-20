import { renderComponent } from "@wabou/test/component";
import {
  Checkbox,
  RadioGroup,
  RadioGroupItem,
  Text,
  Toggle,
  ToggleGroup,
  ToggleGroupItem,
  View,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("supports uncontrolled, controlled, mixed, and disabled checkboxes", () => {
  const App = () => {
    const [checked, setChecked] = createSignal(false);
    return (
      <View>
        <Checkbox label="Uncontrolled" />
        <Checkbox
          checked={checked()}
          onCheckedChange={setChecked}
          label="Controlled"
        />
        <Checkbox indeterminate label="Mixed" />
        <Checkbox disabled label="Disabled" />
      </View>
    );
  };
  const screen = renderComponent(App);
  const uncontrolled = screen.getByRole("checkbox", {
    name: "Uncontrolled",
  });
  const controlled = screen.getByRole("checkbox", { name: "Controlled" });

  expect(uncontrolled.checked).toBe(false);
  uncontrolled.click();
  expect(uncontrolled.checked).toBe(true);
  controlled.click();
  expect(controlled.checked).toBe(true);
  expect(screen.getByRole("checkbox", { name: "Mixed" }).checked).toBe("mixed");
  expect(
    screen.getByRole("checkbox", { name: "Disabled", disabled: true }),
  ).not.toBeNull();
});

test("selects radio items and skips disabled choices during roving focus", () => {
  const App = () => {
    const [plan, setPlan] = createSignal("free");
    return (
      <View>
        <RadioGroup
          value={plan()}
          onValueChange={setPlan}
          aria-label="Subscription plan"
        >
          <RadioGroupItem value="free" label="Free" />
          <RadioGroupItem value="pro" label="Pro" disabled />
          <RadioGroupItem value="team" label="Team" />
        </RadioGroup>
        <Text role="status">{plan()}</Text>
      </View>
    );
  };
  const screen = renderComponent(App);
  const free = screen.getByRole("radio", { name: "Free" });
  const pro = screen.getByRole("radio", { name: "Pro", disabled: true });
  const team = screen.getByRole("radio", { name: "Team" });

  free.focus();
  free.press("ArrowDown");

  expect(pro.focused).toBe(false);
  expect(team.focused).toBe(true);
  expect(team.checked).toBe(true);
  expect(screen.getByRole("status").text).toBe("team");
});

test("keeps a controlled radio group stable until its owner accepts a change", () => {
  const changes: string[] = [];
  const screen = renderComponent(() => (
    <RadioGroup
      value="free"
      onValueChange={(value) => changes.push(value)}
      aria-label="Plan"
    >
      <RadioGroupItem value="free" label="Free" />
      <RadioGroupItem value="team" label="Team" />
    </RadioGroup>
  ));

  screen.getByRole("radio", { name: "Team" }).click();

  expect(changes).toEqual(["team"]);
  expect(screen.getByRole("radio", { name: "Free" }).checked).toBe(true);
});

test("toggles buttons and single-value groups through semantic state", () => {
  const App = () => {
    const [bold, setBold] = createSignal(false);
    const [alignment, setAlignment] = createSignal("left");
    return (
      <View>
        <Toggle pressed={bold()} onPressedChange={setBold} aria-label="Bold">
          B
        </Toggle>
        <ToggleGroup
          type="single"
          value={alignment()}
          onValueChange={setAlignment}
          aria-label="Alignment"
        >
          <ToggleGroupItem value="left">Left</ToggleGroupItem>
          <ToggleGroupItem value="center" disabled>
            Center
          </ToggleGroupItem>
          <ToggleGroupItem value="right">Right</ToggleGroupItem>
        </ToggleGroup>
      </View>
    );
  };
  const screen = renderComponent(App);
  const bold = screen.getByRole("button", { name: "Bold" });
  const left = screen.getByRole("button", { name: "Left" });
  const center = screen.getByRole("button", {
    name: "Center",
    disabled: true,
  });
  const right = screen.getByRole("button", { name: "Right" });

  bold.click();
  expect(bold.pressed).toBe(true);
  left.focus();
  left.press("ArrowRight");

  expect(center.focused).toBe(false);
  expect(right.focused).toBe(true);
  expect(right.pressed).toBe(true);
});
