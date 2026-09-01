import { renderComponent } from "@wabou/test/component";
import {
  Button,
  Onboarding,
  OnboardingDescription,
  OnboardingFooter,
  OnboardingHeader,
  OnboardingHeading,
  OnboardingTitle,
  Text,
} from "@wabou/ui";
import { expect, test, vi } from "vitest";

test("Onboarding exposes one bounded first-run task and supporting copy", () => {
  const continueSetup = vi.fn();
  const screen = renderComponent(() => (
    <Onboarding aria-label="Set up the application">
      <OnboardingHeader>
        <OnboardingHeading>
          <OnboardingTitle>Prepare your first workspace</OnboardingTitle>
          <OnboardingDescription>
            Use the prepared default or choose a different project directory.
          </OnboardingDescription>
        </OnboardingHeading>
      </OnboardingHeader>
      <Button onClick={continueSetup}>Continue</Button>
      <OnboardingFooter>
        <Text>You can change this later.</Text>
      </OnboardingFooter>
    </Onboarding>
  ));

  expect(
    screen.getByRole("region", { name: "Set up the application" }),
  ).toBeDefined();
  expect(
    screen.getByRole("heading", { name: "Prepare your first workspace" }),
  ).toBeDefined();
  expect(JSON.stringify(screen.snapshot())).toContain(
    "Use the prepared default or choose a different project directory.",
  );

  screen.getByRole("button", { name: "Continue" }).click();
  expect(continueSetup).toHaveBeenCalledOnce();
});
