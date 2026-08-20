import { renderComponent } from "@wabou/test/component";
import {
  Button,
  createNotifications,
  NotificationRegion,
  View,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("headless NotificationRegion remains static and removes items directly", () => {
  const screen = renderComponent(() => {
    const notifications = createNotifications({ defaultDuration: 0 });
    return (
      <View>
        <Button
          onClick={() =>
            notifications.show({
              "aria-label": "Static notification",
              content: ({ dismiss }) => (
                <Button onClick={dismiss}>Dismiss static notification</Button>
              ),
            })
          }
        >
          Show static notification
        </Button>
        <NotificationRegion notifications={notifications} motion={false} />
      </View>
    );
  });

  screen.getByRole("button", { name: "Show static notification" }).click();
  expect(
    screen.getByRole("status", { name: "Static notification" }).transform,
  ).toBeNull();
  screen.getByRole("button", { name: "Dismiss static notification" }).click();
  expect(
    screen.queryByRole("status", { name: "Static notification" }),
  ).toBeNull();
});
