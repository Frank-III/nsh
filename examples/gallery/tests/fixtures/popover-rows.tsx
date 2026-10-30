import { mount, Scroll, signal, Text, View } from "nsh";
import { Popover } from "nsh/base";

const rows = Array.from({ length: 30 }, (_, i) => `Row ${i + 1}`);

function App() {
  const open = signal(false);
  const changes = signal<string[]>([]);
  const pressed = signal<string[]>([]);
  const scrolled = signal(false);
  const height = signal(0);
  const offset = signal(0);
  const events = signal<string[]>([]);
  const note = (event: string) => () => events.set([...events(), event]);
  return (
    <View style={{ flex: 1, paddingTop: 100, paddingHorizontal: 20, gap: 8 }}>
      <Text>{`changes: ${changes().join(", ") || "none"}`}</Text>
      <Text>{`pressed: ${pressed().join(", ") || "none"}`}</Text>
      <Text>{`scrolled: ${scrolled() ? "yes" : "no"}`}</Text>
      <Popover
        open={open()}
        onOpenChange={(e) => {
          open.set(e.open);
          changes.set([...changes(), `${e.open ? "opened" : "closed"} by ${e.reason}`]);
        }}
        placement="bottom"
        align="start"
        trigger={
          <View role="button" style={{ height: 44, width: 160, justifyContent: "center" }}>
            <Text>Open menu</Text>
          </View>
        }
        popupStyle={{ width: 240, backgroundColor: "#ffffff", borderRadius: 8 }}
      >
        <Scroll
          testID="menu-list"
          style={{ maxHeight: 300 }}
          onLayout={({ bounds }) => height.set(Math.round(bounds.size.height))}
          onScroll={(e) => {
            scrolled.set(e.offset.y > 0);
            offset.set(Math.round(e.offset.y));
          }}
          onScrollDragStart={note("drag start")}
          onScrollDragEnd={note("drag end")}
          onScrollEnd={note("scroll end")}
        >
          <Each of={rows}>
            {(row) => (
              <View
                role="button"
                onPress={() => pressed.set([...pressed(), row()])}
                style={{ height: 44, justifyContent: "center", paddingHorizontal: 12 }}
              >
                <Text>{row()}</Text>
              </View>
            )}
          </Each>
        </Scroll>
      </Popover>
      <View style={{ marginTop: 308, gap: 8 }}>
        <Text>{`list height: ${height()}`}</Text>
        <Text>{`offset: ${offset()}`}</Text>
        <Text>{`events: ${events().join(", ") || "none"}`}</Text>
      </View>
    </View>
  );
}

mount(App);
