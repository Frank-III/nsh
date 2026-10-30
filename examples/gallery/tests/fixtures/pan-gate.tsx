import { mount, PanResponder, Scroll, signal, Text, View } from "nsh";

// A row of a vertical list that is both pressed and swiped sideways (an
// inbox row): its pan only takes the finger past 10 points sideways, and a
// pressable label inside it.
function App() {
  const rows = signal(0);
  const labels = signal(0);
  const pan = signal("idle");
  const handlers = PanResponder.create({
    onPanResponderGrant: () => pan.set("granted"),
    onPanResponderRelease: () => pan.set("released"),
    onPanResponderTerminate: () => pan.set("terminated"),
    onPanResponderTerminationRequest: () => false,
  }).panHandlers;
  return (
    <Scroll style={{ flex: 1 }} contentStyle={{ paddingTop: 100 }}>
      <View
        {...handlers}
        panActivation={{ activeOffsetX: 10, failOffsetY: 10 }}
        onPress={() => rows.set(rows() + 1)}
        style={{ height: 120, paddingHorizontal: 20, gap: 20 }}
      >
        <Text>row</Text>
        <Text onPress={() => labels.set(labels() + 1)}>label</Text>
      </View>
      <Text>{`rows: ${rows()}`}</Text>
      <Text>{`labels: ${labels()}`}</Text>
      <Text>{`pan: ${pan()}`}</Text>
      <View style={{ height: 1200 }} />
    </Scroll>
  );
}

mount(App);
