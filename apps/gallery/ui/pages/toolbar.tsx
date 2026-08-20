import {
  Icon,
  Text,
  Toolbar,
  ToolbarButton,
  ToolbarGroup,
  ToolbarSeparator,
  ToolbarToggle,
  View,
} from "@wabou/ui";
import alignCenter from "lucide-static/icons/align-center.svg?raw";
import alignLeft from "lucide-static/icons/align-left.svg?raw";
import alignRight from "lucide-static/icons/align-right.svg?raw";
import boldIcon from "lucide-static/icons/bold.svg?raw";
import italicIcon from "lucide-static/icons/italic.svg?raw";
import redo from "lucide-static/icons/redo-2.svg?raw";
import undo from "lucide-static/icons/undo-2.svg?raw";
import { createSignal } from "solid-js";
import { Preview } from "../preview";

function CommandIcon(props: { source: string }) {
  return <Icon source={props.source} size={14} aria-hidden="true" />;
}

export function ToolbarPage() {
  const [bold, setBold] = createSignal(false);
  const [italic, setItalic] = createSignal(false);
  const [alignment, setAlignment] = createSignal("left");

  return (
    <View class="flex flex-col gap-5">
      <Preview title="Formatting commands">
        <View class="flex flex-col items-center gap-3">
          <Toolbar aria-label="Document formatting">
            <ToolbarGroup aria-label="History">
              <ToolbarButton aria-label="Undo">
                <CommandIcon source={undo} />
              </ToolbarButton>
              <ToolbarButton aria-label="Redo" disabled>
                <CommandIcon source={redo} />
              </ToolbarButton>
            </ToolbarGroup>
            <ToolbarSeparator />
            <ToolbarGroup aria-label="Text style">
              <ToolbarToggle
                aria-label="Bold"
                pressed={bold()}
                onPressedChange={setBold}
              >
                <CommandIcon source={boldIcon} />
              </ToolbarToggle>
              <ToolbarToggle
                aria-label="Italic"
                pressed={italic()}
                onPressedChange={setItalic}
              >
                <CommandIcon source={italicIcon} />
              </ToolbarToggle>
            </ToolbarGroup>
            <ToolbarSeparator />
            <ToolbarGroup aria-label="Text alignment">
              <ToolbarToggle
                aria-label="Align left"
                pressed={alignment() === "left"}
                onPressedChange={() => setAlignment("left")}
              >
                <CommandIcon source={alignLeft} />
              </ToolbarToggle>
              <ToolbarToggle
                aria-label="Align center"
                pressed={alignment() === "center"}
                onPressedChange={() => setAlignment("center")}
              >
                <CommandIcon source={alignCenter} />
              </ToolbarToggle>
              <ToolbarToggle
                aria-label="Align right"
                pressed={alignment() === "right"}
                onPressedChange={() => setAlignment("right")}
              >
                <CommandIcon source={alignRight} />
              </ToolbarToggle>
            </ToolbarGroup>
          </Toolbar>
          <Text
            role="status"
            aria-label="Formatting state"
            class="text-xs text-muted"
          >
            {`${bold() ? "Bold" : "Regular"}, ${italic() ? "italic" : "upright"}, ${alignment()} aligned`}
          </Text>
        </View>
      </Preview>

      <Preview title="Vertical tools">
        <Toolbar aria-label="Drawing tools" orientation="vertical">
          <ToolbarButton aria-label="Select tool">Select</ToolbarButton>
          <ToolbarButton aria-label="Frame tool">Frame</ToolbarButton>
          <ToolbarSeparator />
          <ToolbarButton aria-label="Text tool">Text</ToolbarButton>
        </Toolbar>
      </Preview>
    </View>
  );
}
