import { Badge, Button, Card, CardContent, Text, useNavigate, View } from "@wabou/ui";
import arrowLeft from "lucide-static/icons/arrow-left.svg?raw";
import cpu from "lucide-static/icons/cpu.svg?raw";
import image from "lucide-static/icons/image.svg?raw";
import route from "lucide-static/icons/route.svg?raw";
import { Icon } from "@wabou/ui";

const features = [
  {
    icon: image,
    title: "Explicit image resources",
    detail: "Rust owns decoded source images while the viewport controls presentation and annotations.",
  },
  {
    icon: cpu,
    title: "Dedicated OCR worker",
    detail: "A named background thread owns the OCR engine and processes queued inference requests serially.",
  },
  {
    icon: route,
    title: "Native data router",
    detail: "Application screens use Wabou's TanStack Router Core adapter without a browser DOM.",
  },
] as const;

export function About() {
  const navigate = useNavigate();
  return (
    <View class="w-full h-full min-w-0 min-h-0 flex flex-col bg-canvas text-primary">
      <View class="h-14 flex-none px-5 flex flex-row items-center gap-3 border-b border-subtle bg-surface shadow-sm">
        <Button size="sm" variant="ghost" aria-label="Back to reader" onClick={() => void navigate({ to: "/" })}>
          <Icon source={arrowLeft} size={16} />
          Reader
        </Button>
        <Text class="font-semibold">About Manga OCR</Text>
        <View class="flex-1" />
        <Badge variant="secondary">Wabou experiment</Badge>
      </View>
      <View class="flex-1 min-h-0 p-8 items-center justify-center">
        <View class="w-full max-w-5xl flex flex-col gap-5">
          <View class="flex flex-col gap-2">
            <Text class="text-3xl font-semibold">A native manga workflow</Text>
            <Text class="text-muted">OCR, editable image-space annotations, and LLM translation in one predictable pipeline.</Text>
          </View>
          <View class="grid grid-cols-3 gap-4">
            {features.map((feature) => (
              <Card class="min-w-0">
                <CardContent class="p-5 flex flex-col gap-3">
                  <View class="w-10 h-10 rounded-lg bg-selected items-center justify-center">
                    <Icon source={feature.icon} size={20} class="text-accent" />
                  </View>
                  <Text class="font-semibold">{feature.title}</Text>
                  <Text maxLines={4} class="text-sm text-muted">{feature.detail}</Text>
                </CardContent>
              </Card>
            ))}
          </View>
        </View>
      </View>
    </View>
  );
}
