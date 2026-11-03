import { createAnimatedValue, mount, Scroll, signal, Text, View } from "nsh";
import { Button, createNavStack, NavPage, NavStack } from "nsh/base";

function Action(props: { label: string; onPress: () => void }) {
  return <Button onPress={props.onPress} style={{ padding: 12, backgroundColor: "#e2e8f0", borderRadius: 8 }}>
    <Text style={{ color: "#0f172a", fontSize: 16 }}>{props.label}</Text>
  </Button>;
}

function App() {
  const nav = createNavStack<"root" | "chart" | "cover">(["root"]);
  const enabled = signal(true);
  const hidden = signal(false);
  const scrubbing = signal(false);
  const cursor = createAnimatedValue(0);
  const bars = [40, 65, 90, 55, 115, 80, 140, 95, 120, 70, 100, 60];
  const finish = () => scrubbing.set(false);
  return <View style={{ flex: 1, paddingTop: 60, backgroundColor: "#ffffff" }}>
    <Text style={{ color: "#0f172a", padding: 8 }}>{`Depth: ${nav.routes().length}`}</Text>
    <NavStack state={nav} style={{ flex: 1 }}>
      {route => <NavPage
        title={route === "root" ? "Interactive pop" : route === "chart" ? "Chart scrub" : "Cover page"}
        headerVisible={!hidden()}
        ios={{ interactivePopEnabled: route !== "chart" || (enabled() && !scrubbing()) }}
      >
        <View style={{ flex: 1, padding: 16, gap: 12, backgroundColor: "#ffffff" }}>
          <If when={route === "root"}>
            <Text style={{ color: "#0f172a", fontSize: 18 }}>iOS interactive pop</Text>
            <Action label="Open chart" onPress={() => { enabled.set(true); hidden.set(false); nav.push("chart"); }} />
            <Text style={{ color: "#475569" }}>At the root, scroll this row left and right.</Text>
            <Scroll testID="root-list" horizontal style={{ height: 100, flexGrow: 0 }} contentStyle={{ flexDirection: "row", gap: 12 }}>
              <Each of={bars}>{(_, index) => <View style={{ width: 160, height: 80, backgroundColor: "#dbeafe", padding: 16 }}>
                <Text>{`Root card ${index() + 1}`}</Text>
              </View>}</Each>
            </Scroll>
          </If>
          <If when={route === "chart"}>
            <Text style={{ color: "#0f172a" }}>{`Swipe back: ${enabled() && !scrubbing() ? "enabled" : "disabled"}`}</Text>
            <Action label="Toggle swipe back" onPress={() => enabled.set(!enabled())} />
            <Action label="Toggle header" onPress={() => hidden.set(!hidden())} />
            <Text style={{ color: "#475569" }}>Hold the chart until it says Scrubbing, then drag sideways. Lift to restore the toggle's setting.</Text>
            <View testID="chart" delayLongPress={400}
              onTouchStart={e => cursor.set(e.locationX)}
              onLongPress={() => scrubbing.set(true)}
              onTouchMove={e => { if (scrubbing()) cursor.set(e.locationX); }}
              onTouchEnd={finish} onTouchCancel={finish}
              style={{ height: 180, backgroundColor: "#eff6ff", overflow: "hidden", borderRadius: 8 }}>
              <View style={{ pointerEvents: "none", flex: 1, flexDirection: "row", alignItems: "flex-end", gap: 8, padding: 12 }}>
                <Each of={bars}>{height => <View style={{ flex: 1, height: height(), backgroundColor: "#2563eb" }} />}</Each>
              </View>
              <View style={{ pointerEvents: "none", position: "absolute", left: 0, top: 0, bottom: 0, width: 2, backgroundColor: "#0f172a", translate: [cursor, 0], opacity: scrubbing() ? 1 : 0 }} />
            </View>
            <Text style={{ color: "#0f172a" }}>{scrubbing() ? "Scrubbing" : "Hold to scrub"}</Text>
            <Action label="Push cover page" onPress={() => nav.push("cover")} />
          </If>
          <If when={route === "cover"}>
            <Text style={{ color: "#0f172a" }}>This page allows swipe back, independently of the chart below it.</Text>
            <Action label="Toggle covered chart" onPress={() => enabled.set(!enabled())} />
          </If>
          <If when={route !== "root"}>
            <Action label="Programmatic back" onPress={() => nav.pop()} />
          </If>
          <View testID="edge" style={{ position: "absolute", left: 0, bottom: 40, width: 4, height: 40 }} />
          <View testID="content" style={{ position: "absolute", left: 100, bottom: 40, width: 4, height: 40 }} />
        </View>
      </NavPage>}
    </NavStack>
  </View>;
}

mount(App);
