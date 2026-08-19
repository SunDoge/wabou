import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Path,
  PathBuilder,
  Text,
  View,
} from "@wabou/ui";
import { scaleLinear } from "d3-scale";
import { area, curveMonotoneX, line } from "d3-shape";

interface Sample {
  second: number;
  download: number;
  upload: number;
}

const samples: Sample[] = [
  { second: 0, download: 18, upload: 4 },
  { second: 1, download: 26, upload: 7 },
  { second: 2, download: 22, upload: 6 },
  { second: 3, download: 38, upload: 11 },
  { second: 4, download: 44, upload: 9 },
  { second: 5, download: 36, upload: 14 },
  { second: 6, download: 53, upload: 12 },
  { second: 7, download: 61, upload: 18 },
  { second: 8, download: 57, upload: 16 },
  { second: 9, download: 72, upload: 21 },
  { second: 10, download: 68, upload: 19 },
  { second: 11, download: 82, upload: 24 },
];

const width = 760;
const height = 300;
const margin = { top: 20, right: 24, bottom: 36, left: 48 };
const x = scaleLinear()
  .domain([0, samples.at(-1)?.second ?? 0])
  .range([margin.left, width - margin.right]);
const y = scaleLinear()
  .domain([0, 90])
  .nice()
  .range([height - margin.bottom, margin.top]);

function d3Context(path: PathBuilder) {
  // d3-shape only calls this path-method subset, but its public context type is
  // the full browser canvas interface. Keep that compatibility cast at the
  // adapter boundary instead of making Wabou emulate CanvasRenderingContext2D.
  return {
    moveTo: (x: number, y: number) => path.moveTo(x, y),
    lineTo: (x: number, y: number) => path.lineTo(x, y),
    quadraticCurveTo: (cx: number, cy: number, x: number, y: number) =>
      path.quadTo(cx, cy, x, y),
    bezierCurveTo: (
      c1x: number,
      c1y: number,
      c2x: number,
      c2y: number,
      x: number,
      y: number,
    ) => path.cubicTo(c1x, c1y, c2x, c2y, x, y),
    closePath: () => path.close(),
  } as unknown as CanvasRenderingContext2D;
}

const downloadLine = new PathBuilder();
line<Sample>()
  .x((sample) => x(sample.second))
  .y((sample) => y(sample.download))
  .curve(curveMonotoneX)
  .context(d3Context(downloadLine))(samples);
const uploadLine = new PathBuilder();
line<Sample>()
  .x((sample) => x(sample.second))
  .y((sample) => y(sample.upload))
  .curve(curveMonotoneX)
  .context(d3Context(uploadLine))(samples);
const downloadArea = new PathBuilder();
area<Sample>()
  .x((sample) => x(sample.second))
  .y0(y(0))
  .y1((sample) => y(sample.download))
  .curve(curveMonotoneX)
  .context(d3Context(downloadArea))(samples);

const grid = new PathBuilder();
for (const tick of y.ticks(5)) {
  grid.moveTo(margin.left, y(tick)).lineTo(width - margin.right, y(tick));
}

const gridPath = grid.build({ stroke: 0x33415573 });
const areaPath = downloadArea.build({ fill: 0x38bdf852 });
const downloadPath = downloadLine.build({
  stroke: 0x38bdf8ff,
  strokeWidth: 3,
  lineCap: "round",
  lineJoin: "round",
});
const uploadPath = uploadLine.build({
  stroke: 0xa78bfaff,
  strokeWidth: 2.5,
  lineCap: "round",
  lineJoin: "round",
});

export function ChartPage() {
  return (
    <View class="flex flex-col gap-5">
      <Card>
        <CardHeader class="flex flex-row items-center justify-between">
          <View class="flex flex-col gap-1">
            <CardTitle>Transfer speed</CardTitle>
            <Text class="text-sm text-muted">
              D3 scale and shape, typed native Vello paths
            </Text>
          </View>
          <View class="flex items-center gap-4">
            <Text class="text-sm text-accent">Download 82 MB/s</Text>
            <Text class="text-sm text-secondary">Upload 24 MB/s</Text>
          </View>
        </CardHeader>
        <CardContent>
          <View
            aria-label="Download and upload speed chart"
            role="img"
            class="relative w-full h-72"
          >
            <Path class="absolute inset-0 w-full h-full" source={gridPath} />
            <Path class="absolute inset-0 w-full h-full" source={areaPath} />
            <Path
              class="absolute inset-0 w-full h-full"
              source={downloadPath}
            />
            <Path class="absolute inset-0 w-full h-full" source={uploadPath} />
          </View>
          <View class="px-12 flex items-center justify-between">
            <Text class="text-xs text-muted">0 s</Text>
            <Text class="text-xs text-muted">11 s</Text>
          </View>
        </CardContent>
      </Card>
      <Text class="text-sm text-secondary">
        The library only computes scales, ticks, and path geometry. Wabou owns
        layout, accessibility, interaction, and native rendering.
      </Text>
    </View>
  );
}
