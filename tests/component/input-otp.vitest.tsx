import { renderComponent } from "@wabou/test/component";
import { InputOTP, InputOTPGroup, InputOTPSlot } from "@wabou/ui";
import { expect, test } from "vitest";

test("disabled OTP keeps its value while exposing the shared unavailable state", () => {
  const screen = renderComponent(() => (
    <InputOTP aria-label="Security code" maxLength={2} value="12" disabled>
      <InputOTPGroup>
        <InputOTPSlot index={0} />
        <InputOTPSlot index={1} />
      </InputOTPGroup>
    </InputOTP>
  ));
  const group = screen.getByRole("group", { name: "Security code" });
  const input = screen.getByRole("textbox", {
    name: "Security code",
    disabled: true,
  });

  expect(group.className).toContain("cursor-not-allowed");
  expect(group.className).toContain("opacity-60");
  expect(input.value).toBe("12");
  expect(() => input.input("34")).toThrow("cannot input into disabled");
});
