import {
  Attachment,
  AttachmentAction,
  AttachmentActions,
  AttachmentContent,
  AttachmentDescription,
  AttachmentGroup,
  AttachmentMedia,
  AttachmentTitle,
  Button,
  Icon,
  useDialog,
} from "@wabou/ui";
import image from "lucide-static/icons/image.svg?raw";
import paperclip from "lucide-static/icons/paperclip.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { For, Show } from "solid-js";
import { i18n, m } from "./i18n";

function fileName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

export interface ComposerImagesProps {
  paths: readonly string[];
  change(paths: readonly string[]): void;
}

export function ComposerImages(props: ComposerImagesProps) {
  return (
    <Show when={props.paths.length > 0}>
      <AttachmentGroup
        role="group"
        aria-label={i18n.message(m.attached_images, {})}
      >
        <For each={props.paths}>
          {(path) => (
            <Attachment size="sm" class="max-w-64">
              <AttachmentMedia>
                <Icon source={image} size={14} />
              </AttachmentMedia>
              <AttachmentContent>
                <AttachmentTitle>{fileName(path)}</AttachmentTitle>
                <AttachmentDescription>
                  {i18n.message(m.image_attachment, {})}
                </AttachmentDescription>
              </AttachmentContent>
              <AttachmentActions>
                <AttachmentAction
                  size="icon"
                  aria-label={i18n.message(m.remove_attachment, {
                    name: fileName(path),
                  })}
                  onClick={() =>
                    props.change(props.paths.filter((item) => item !== path))
                  }
                >
                  <Icon source={x} size={12} />
                </AttachmentAction>
              </AttachmentActions>
            </Attachment>
          )}
        </For>
      </AttachmentGroup>
    </Show>
  );
}

export function ComposerImagePicker(props: ComposerImagesProps) {
  const dialog = useDialog();
  const choose = async () => {
    const paths = await dialog.open({
      title: i18n.message(m.attach_images, {}),
      multiple: true,
      filters: [
        {
          name: i18n.message(m.images, {}),
          extensions: ["png", "jpg", "jpeg", "webp", "gif"],
        },
      ],
    });
    if (!paths?.length) return;
    props.change([...new Set([...props.paths, ...paths])]);
  };

  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label={i18n.message(m.attach_images, {})}
      onClick={() => void choose()}
    >
      <Icon source={paperclip} size={14} />
    </Button>
  );
}
