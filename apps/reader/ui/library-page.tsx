import {
  Badge,
  Button,
  Icon,
  Row,
  ScrollArea,
  Text,
  useNavigate,
  View,
  WorkbenchContent,
  WorkbenchHeader,
} from "@wabou/ui";
import bookOpen from "lucide-static/icons/book-open.svg?raw";
import fileText from "lucide-static/icons/file-text.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import { For as ForValue } from "solid-js";
import { useReader } from "./reader-state";

export function LibraryPage() {
  const reader = useReader();
  const navigate = useNavigate();
  const open = async () => {
    if (await reader.openPdf()) navigate({ to: "/" });
  };
  return (
    <View class="w-full h-full flex flex-col overflow-hidden">
      <WorkbenchHeader>
        <Text class="flex-1 font-semibold">Library</Text>
        <Button
          size="sm"
          disabled={reader.opening()}
          onClick={() => void open()}
        >
          <Icon source={plus} size={14} />
          Open PDF
        </Button>
      </WorkbenchHeader>
      <WorkbenchContent>
        <ScrollArea class="w-full h-full" contentClass="px-8 py-8">
          <View class="w-full max-w-5xl mx-auto">
            <Text class="text-2xl font-semibold tracking-tight">
              Your reading shelf
            </Text>
            <Text class="mt-2 text-sm text-muted">
              Documents stay on this device. Open one to return to the reading
              desk.
            </Text>
            <View
              role="list"
              aria-label="Library documents"
              class="mt-8 grid grid-cols-3 gap-4"
            >
              <ForValue each={reader.visibleDocuments()}>
                {(document) => (
                  <View role="listitem" aria-label={document.title}>
                    <Button
                      variant="outline"
                      aria-label={`Read ${document.title}`}
                      class="w-full min-w-0 h-48 p-5 flex flex-col items-start text-left rounded-xl border-subtle bg-surface shadow-sm"
                      onClick={() => {
                        reader.selectDocument(document.id);
                        navigate({ to: "/" });
                      }}
                    >
                      <View class="w-10 h-12 rounded-md border border-subtle bg-surface-muted flex items-center justify-center">
                        <Icon
                          source={document.kind === "pdf" ? fileText : bookOpen}
                          size={18}
                          class="text-accent"
                        />
                      </View>
                      <Text class="w-full mt-5 truncate text-sm font-semibold">
                        {document.title}
                      </Text>
                      <Text class="w-full mt-1 truncate text-xs text-muted">
                        {document.path ?? "Built-in reading guide"}
                      </Text>
                      <Row class="w-full mt-auto items-center">
                        <Badge variant="outline" size="sm">
                          {document.kind === "pdf" ? "PDF" : "Guide"}
                        </Badge>
                        <Text class="ml-auto text-xs text-muted">Open</Text>
                      </Row>
                    </Button>
                  </View>
                )}
              </ForValue>
            </View>
          </View>
        </ScrollArea>
      </WorkbenchContent>
    </View>
  );
}
