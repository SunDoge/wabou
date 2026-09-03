import { renderComponent } from "@wabou/test/component";
import { PromptSuggestion, PromptSuggestions } from "@wabou/ui";
import { expect, test, vi } from "vitest";

const sparkles = '<svg viewBox="0 0 24 24"><path d="M12 3v18" /></svg>';

test("prompt suggestions expose explicit application-owned actions", () => {
  const choose = vi.fn();
  const screen = renderComponent(() => (
    <PromptSuggestions role="group" aria-label="Starter prompts">
      <PromptSuggestion
        icon={sparkles}
        title="Review current changes"
        description="Find the highest-risk issue."
        onClick={() => choose("review")}
      />
      <PromptSuggestion
        title="Run project checks"
        onClick={() => choose("verify")}
      />
    </PromptSuggestions>
  ));

  screen.getByRole("button", { name: "Review current changes" }).click();
  expect(choose).toHaveBeenCalledWith("review");
  expect(
    screen.getByRole("button", { name: "Run project checks" }),
  ).toBeDefined();
});
