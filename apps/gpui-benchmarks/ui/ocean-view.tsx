import {
  Button,
  Column,
  Icon,
  Row,
  Svg,
  Text,
  useNavigate,
  View,
} from "@wabou/ui";
import bell from "lucide-static/icons/bell.svg?raw";
import clock from "lucide-static/icons/clock.svg?raw";
import compass from "lucide-static/icons/compass.svg?raw";
import expand from "lucide-static/icons/expand.svg?raw";
import file from "lucide-static/icons/file-chart-column.svg?raw";
import fuel from "lucide-static/icons/fuel.svg?raw";
import gauge from "lucide-static/icons/gauge.svg?raw";
import lifeBuoy from "lucide-static/icons/life-buoy.svg?raw";
import mapIcon from "lucide-static/icons/map.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
import navigation from "lucide-static/icons/navigation.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import ship from "lucide-static/icons/ship.svg?raw";
import waves from "lucide-static/icons/waves.svg?raw";
import { For as ForValue } from "solid-js";

const mapSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 560">
<rect width="1000" height="560" fill="#0a2d61"/>
<g stroke="#19477f" stroke-width="1" opacity=".45"><path d="M0 80H1000M0 160H1000M0 240H1000M0 320H1000M0 400H1000M0 480H1000"/><path d="M100 0V560M200 0V560M300 0V560M400 0V560M500 0V560M600 0V560M700 0V560M800 0V560M900 0V560"/></g>
<g fill="#0d356e" stroke="#2f68ac" stroke-width="3">
<path d="M0 35 52 18 96 45 134 41 152 78 210 72 232 110 291 103 311 140 282 168 222 154 183 191 124 180 89 212 34 190 0 203Z"/>
<path d="M355 0 401 50 380 82 425 125 493 140 511 190 486 230 438 204 411 250 372 236 342 192 309 177 322 129 292 96Z"/>
<path d="M520 66 574 51 626 74 683 57 723 93 778 84 817 121 864 115 905 148 949 134 1000 159V0H526Z"/>
<path d="M655 190 699 179 731 204 775 199 811 231 846 221 869 257 839 288 790 275 759 302 704 287 680 252 641 239Z"/>
<path d="M845 363 886 341 927 359 956 391 939 431 891 445 854 421 823 391Z"/>
<path d="M92 372 142 354 184 379 217 418 196 467 152 482 113 454 71 436 58 401Z"/>
</g>
<g fill="none" stroke="#3d8df0" stroke-width="2" stroke-dasharray="6 6" opacity=".85"><path d="M207 291C330 238 457 278 548 348S743 395 883 378"/><path d="M411 427C472 362 520 315 606 277"/></g>
</svg>`;

const vessels = [
  { name: "MV Serenity", left: "25%", top: "32%", color: "#f0b53c" },
  { name: "MV Polaris", left: "49%", top: "52%", color: "#38d7ef" },
  { name: "MV Voyager", left: "39%", top: "61%", color: "#54c7ff" },
  { name: "MV Horizon", left: "82%", top: "44%", color: "#f4ca45" },
] as const;

function DarkNav(props: {
  source: string;
  label: string;
  active?: boolean;
  dot?: boolean;
}) {
  return (
    <Row
      class="h-9 px-3 items-center gap-3 rounded-md"
      classList={{
        "bg-[#19498e] text-white": props.active,
        "text-[#b7c8df]": !props.active,
      }}
    >
      <Icon source={props.source} size={15} />
      <Text class="text-sm">{props.label}</Text>
      <View class="flex-1" />
      {props.dot ? (
        <View class="w-1.5 h-1.5 rounded-full bg-[#3d8cff]" />
      ) : null}
    </Row>
  );
}

export function OceanView() {
  const navigate = useNavigate();
  return (
    <View
      role="group"
      aria-label="OceanView fleet command"
      class="w-full h-full overflow-hidden flex bg-[#071427] text-[#e9f2ff] font-sans"
    >
      <Column class="w-56 h-full flex-none border-r border-[#18304f] bg-[#08172b]">
        <Row class="h-14 px-4 flex-none items-center gap-3 border-b border-[#18304f]">
          <Button
            variant="ghost"
            size="icon"
            aria-label="Back to benchmarks"
            class="w-8 h-8 rounded-full bg-[#1e63dd] text-white"
            onClick={() => navigate({ to: "/" })}
          >
            <Icon source={waves} size={16} />
          </Button>
          <Text class="font-semibold">OceanView</Text>
        </Row>
        <Column class="p-3 gap-1">
          <DarkNav source={gauge} label="Overview" active />
          <DarkNav source={ship} label="Fleet" />
          <DarkNav source={mapIcon} label="Live Tracking" dot />
          <DarkNav source={file} label="Reports" />
          <DarkNav source={bell} label="Alerts" />
          <DarkNav source={settings} label="Settings" />
        </Column>
        <Text class="px-5 pt-2 pb-2 text-xs text-[#607998]">VESSELS</Text>
        <Column class="px-3 gap-1">
          <DarkNav source={ship} label="MV Aurora" dot />
          <DarkNav source={ship} label="MV Horizon" dot />
          <DarkNav source={ship} label="MV Serenity" dot />
          <DarkNav source={ship} label="MV Polaris" active dot />
          <DarkNav source={ship} label="MV Voyager" dot />
        </Column>
        <View class="flex-1" />
        <Text class="px-5 pb-2 text-xs text-[#607998]">QUICK ACTIONS</Text>
        <Column class="px-3 pb-4 gap-2">
          <Button
            variant="outline"
            class="h-8 border-[#1c3555] bg-[#0d2039] text-[#d3e1f3] gap-2"
          >
            <Icon source={navigation} size={13} />
            New Route
          </Button>
          <Button
            variant="outline"
            class="h-8 border-[#1c3555] bg-[#0d2039] text-[#d3e1f3] gap-2"
          >
            <Icon source={file} size={13} />
            Create Report
          </Button>
          <Button
            variant="outline"
            class="h-8 border-[#1c3555] bg-[#0d2039] text-[#d3e1f3] gap-2"
          >
            <Icon source={lifeBuoy} size={13} />
            Contact Support
          </Button>
        </Column>
      </Column>

      <Column class="min-w-0 h-full flex-1">
        <Row class="h-14 px-5 flex-none items-center gap-2 border-b border-[#18304f] bg-[#08172b]">
          <Text class="text-xs text-[#90a6c2]">Fleet</Text>
          <Text class="text-xs">/</Text>
          <Text class="text-xs text-[#90a6c2]">Live Tracking</Text>
          <Text class="text-xs">/</Text>
          <Text class="text-xs font-medium">MV Polaris</Text>
          <View class="flex-1" />
          <View class="px-2 py-1 rounded-full border border-[#24533e] bg-[#0c2c28]">
            <Text class="text-xs text-[#77d5a1]">● System Online · ×1</Text>
          </View>
          <Row class="h-8 w-64 px-3 items-center gap-2 rounded-lg border border-[#1a3150] bg-[#0b1d35]">
            <Icon source={search} size={13} />
            <Text class="text-xs text-[#7890ad]">
              Search ships, ports, or IDs...
            </Text>
          </Row>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Notifications"
            class="text-[#b8c8dc]"
          >
            <Icon source={bell} size={14} />
          </Button>
          <View class="w-8 h-8 rounded-full bg-[#20477d] flex items-center justify-center">
            <Text class="text-xs">JD</Text>
          </View>
        </Row>
        <View class="relative min-h-0 flex-1 overflow-hidden">
          <Svg
            source={mapSvg}
            aria-label="Ocean fleet route map"
            class="absolute inset-0 w-full h-full"
          />
          <Row class="absolute top-3 left-3 px-3 py-2 items-center gap-3 rounded-lg border border-[#285489] bg-[#081b33e8]">
            <View class="w-14 h-8 rounded bg-[#123d72] flex items-center justify-center">
              <Icon source={ship} size={26} class="text-[#ffb44f]" />
            </View>
            <Column>
              <Text class="text-sm font-semibold">MV Polaris</Text>
              <Text class="text-xs text-[#8fa9c7]">
                IMO: 9298113 · MMSI 563087612
              </Text>
              <Text class="text-xs text-[#8fa9c7]">Next: Colombo</Text>
            </Column>
          </Row>
          <Row class="absolute top-3 right-3 gap-2">
            <Button
              variant="outline"
              class="border-[#285489] bg-[#0b2341] text-[#dceaff] gap-2"
            >
              <Icon source={mapIcon} size={13} />
              Free View
            </Button>
            <Button
              variant="outline"
              class="border-[#285489] bg-[#0b2341] text-[#dceaff] gap-2"
            >
              <Icon source={clock} size={13} />
              Last 24 hours
            </Button>
            <Button
              variant="outline"
              size="icon"
              aria-label="Expand map"
              class="border-[#285489] bg-[#0b2341] text-[#dceaff]"
            >
              <Icon source={expand} size={14} />
            </Button>
          </Row>
          <Column class="absolute right-3 top-20 rounded-lg overflow-hidden border border-[#285489]">
            <Button
              variant="ghost"
              size="icon"
              aria-label="Zoom in"
              class="rounded-none bg-[#0b2341] text-white"
            >
              <Icon source={plus} size={14} />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              aria-label="Zoom out"
              class="rounded-none bg-[#0b2341] text-white"
            >
              <Icon source={minus} size={14} />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              aria-label="Locate"
              class="rounded-none bg-[#0b2341] text-white"
            >
              <Icon source={navigation} size={14} />
            </Button>
          </Column>
          <ForValue each={vessels}>
            {(vessel) => (
              <View
                class="absolute"
                style={{ left: vessel.left, top: vessel.top }}
              >
                <Column class="items-center">
                  <View class="px-2 py-1 rounded border border-[#2f669f] bg-[#071a31e8]">
                    <Text class="text-xs">{vessel.name}</Text>
                    <Text class="text-xs text-[#8fb0d6]">0.0 kn · 331°</Text>
                  </View>
                  <View
                    class="w-4 h-4 mt-1 rotate-45 border border-white"
                    style={{ "background-color": vessel.color }}
                  />
                </Column>
              </View>
            )}
          </ForValue>
          <View class="absolute bottom-3 left-3 px-3 py-2 rounded-lg border border-[#285489] bg-[#081b33e8]">
            <Row class="items-center gap-3">
              <View class="w-10 h-10 rounded-full border border-[#558bc6] flex items-center justify-center">
                <Icon source={compass} size={19} />
              </View>
              <Column>
                <Text class="text-xs">6° 56.400′ N</Text>
                <Text class="text-xs">79° 50.580′ E</Text>
              </Column>
            </Row>
          </View>
        </View>
        <Row class="h-44 flex-none p-3 gap-3 border-t border-[#18304f] bg-[#08172b]">
          <Column class="min-w-0 flex-1 p-3 rounded-lg border border-[#1b3557] bg-[#0a1c34]">
            <Row>
              <Icon source={navigation} size={14} />
              <Text class="ml-2 text-sm font-medium">Route & Position</Text>
              <View class="flex-1" />
              <Text class="text-xs text-[#4c88df]">Details ›</Text>
            </Row>
            <Row class="mt-4">
              <Column class="flex-1 gap-2">
                <Text class="text-xs text-[#829ab8]">From</Text>
                <Text class="text-xs text-[#6dd69b]">● Port Klang</Text>
                <Text class="text-xs text-[#829ab8]">ETA</Text>
                <Text class="text-xs">Nov 15, 2026</Text>
              </Column>
              <Column class="flex-1 gap-2">
                <Text class="text-xs text-[#829ab8]">To</Text>
                <Text class="text-xs text-[#6dd69b]">● Colombo</Text>
                <Text class="text-xs text-[#829ab8]">Remaining</Text>
                <Text class="text-xs">0 NM</Text>
              </Column>
            </Row>
            <View class="mt-3 h-1.5 rounded-full bg-[#153150]">
              <View class="w-full h-full rounded-full bg-[#26c46e]" />
            </View>
          </Column>
          <Column class="w-44 p-3 rounded-lg border border-[#1b3557] bg-[#0a1c34]">
            <Row>
              <Icon source={gauge} size={14} />
              <Text class="ml-2 text-sm">Speed & Course</Text>
            </Row>
            <Row class="flex-1 items-center justify-center">
              <View class="w-16 h-16 rounded-full border-4 border-[#233a59] flex flex-col items-center justify-center">
                <Text class="text-xl">0.0</Text>
                <Text class="text-xs text-[#7890ad]">Knots</Text>
              </View>
              <Column class="ml-5 gap-2">
                <Text class="text-xs text-[#7890ad]">Course</Text>
                <Text class="text-base">331° ↗</Text>
                <Text class="text-xs text-[#f1ba36]">● Arrived</Text>
              </Column>
            </Row>
          </Column>
          <Column class="min-w-0 flex-1 p-3 rounded-lg border border-[#1b3557] bg-[#0a1c34]">
            <Row>
              <Icon source={fuel} size={14} />
              <Text class="ml-2 text-sm">Fuel & Consumption</Text>
              <View class="flex-1" />
              <Text class="text-xs text-[#4c88df]">Details ›</Text>
            </Row>
            <View class="mt-5 h-8 border border-[#176943] bg-[#0f5738]">
              <View class="w-3/4 h-full bg-[#168b54]" />
            </View>
            <Row class="mt-2">
              <Text class="text-xs text-[#7890ad]">Fuel Level</Text>
              <View class="flex-1" />
              <Text class="text-xs">24%</Text>
            </Row>
            <View class="mt-2 h-1.5 bg-[#14304e]">
              <View class="w-1/4 h-full bg-[#29cb72]" />
            </View>
          </Column>
          <Column class="min-w-0 flex-1 p-3 rounded-lg border border-[#1b3557] bg-[#0a1c34]">
            <Row>
              <Icon source={bell} size={14} />
              <Text class="ml-2 text-sm">Recent Alerts</Text>
              <View class="flex-1" />
              <Text class="text-xs text-[#4c88df]">View All ›</Text>
            </Row>
            <Text class="mt-5 text-xs text-[#92a8c2]">
              No alerts yet. Rules check deviation, fuel and engine temperature
              every step.
            </Text>
          </Column>
        </Row>
      </Column>
    </View>
  );
}
