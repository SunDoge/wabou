import type { Dialog } from "@wabou/core";
import { renderComponent } from "@wabou/test/component";
import { DirectoryPicker, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";

function dialogWith(pickDirectory: Dialog["pickDirectory"]): Dialog {
  return {
    open: async () => null,
    save: async () => null,
    pickDirectory,
    message: async () => "ok",
  };
}

test("edits a controlled path and commits a native directory selection", async () => {
  let completeSelection: (path: string | null) => void = () => {};
  const pickDirectory = vi.fn(
    () =>
      new Promise<string | null>((resolve) => {
        completeSelection = resolve;
      }),
  );
  const App = () => {
    const [path, setPath] = createSignal("/home/user/Downloads");
    return (
      <View>
        <DirectoryPicker
          value={path()}
          onValueChange={setPath}
          aria-label="Download directory"
          browseAriaLabel="Choose directory"
          pendingLabel="Opening picker"
          dialogOptions={{ title: "Download directory" }}
        />
        <Text role="status">{path()}</Text>
      </View>
    );
  };
  const screen = renderComponent(App, {
    platform: { dialog: dialogWith(pickDirectory) },
  });
  const input = screen.getByRole("textbox", { name: "Download directory" });
  const browse = screen.getByRole("button", { name: "Choose directory" });

  input.input("/tmp/manual");
  expect(screen.getByRole("status").text).toBe("/tmp/manual");

  browse.click();
  expect(pickDirectory).toHaveBeenCalledWith({
    directory: "/tmp/manual",
    title: "Download directory",
  });
  expect(browse.disabled).toBe(true);
  expect(browse.text).toBe("Opening picker");

  completeSelection("/tmp/native");
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("/tmp/native");
  });
  expect(browse.disabled).toBe(false);
});

test("preserves the current path when the native picker is cancelled", async () => {
  const screen = renderComponent(
    () => {
      const [path, setPath] = createSignal("/data");
      return (
        <View>
          <DirectoryPicker
            value={path()}
            onValueChange={setPath}
            browseAriaLabel="Browse"
          />
          <Text role="status">{path()}</Text>
        </View>
      );
    },
    { platform: { dialog: dialogWith(async () => null) } },
  );

  screen.getByRole("button", { name: "Browse" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("button", { name: "Browse" }).disabled).toBe(false);
  });
  expect(screen.getByRole("status").text).toBe("/data");
});

test("reports picker failures and restores the browse action", async () => {
  const failure = new Error("portal unavailable");
  const onBrowseError = vi.fn();
  const screen = renderComponent(
    () => (
      <DirectoryPicker
        value="/data"
        onValueChange={() => {}}
        browseAriaLabel="Browse"
        onBrowseError={onBrowseError}
      />
    ),
    {
      platform: {
        dialog: dialogWith(async () => {
          throw failure;
        }),
      },
    },
  );
  const browse = screen.getByRole("button", { name: "Browse" });

  browse.click();
  await screen.waitFor(() => {
    expect(onBrowseError).toHaveBeenCalledWith(failure);
  });
  expect(browse.disabled).toBe(false);
});
