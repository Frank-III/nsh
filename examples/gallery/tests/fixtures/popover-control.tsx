// The same list of rows in three places, at about one height on screen: on
// the page, in a bare `Overlay` (no backdrop, no listeners) and in a
// `Popover` (its backdrop below it). `popover-touch-control.test.tsx`
// scrolls each with the same steps, so a run on a device tells a scroll
// step that cannot drive a scroll view in an overlay from a popover that
// keeps its content from scrolling.
import { mount, Overlay, Scroll, signal, Text, View, type ViewRef } from "nsh";
import { Popover } from "nsh/base";

const rows = Array.from({ length: 30 }, (_, i) => `Row ${i + 1}`);

/** What a list reports: whether it moved, how far, and its drag events. */
function tracked() {
  const offset = signal(0);
  const events = signal<string[]>([]);
  const note = (what: string) => () => events.set([...events(), what]);
  return { offset, events, note };
}

function Rows(props: { id: string; list: ReturnType<typeof tracked> }) {
  return (
    <Scroll
      testID={props.id}
      style={{ height: 300, width: 240, backgroundColor: "#ffffff" }}
      onScroll={(e) => props.list.offset.set(Math.round(e.offset.y))}
      onScrollDragStart={props.list.note("drag start")}
      onScrollDragEnd={props.list.note("drag end")}
      onScrollEnd={props.list.note("scroll end")}
    >
      <Each of={rows}>
        {(row) => (
          <View role="button" onPress={() => {}} style={{ height: 44, justifyContent: "center", paddingHorizontal: 12 }}>
            <Text>{row()}</Text>
          </View>
        )}
      </Each>
    </Scroll>
  );
}

function App() {
  const page = tracked();
  const overlay = tracked();
  const popover = tracked();
  const shown = signal(false);
  const open = signal(false);
  let anchor: ViewRef | undefined;
  return (
    <View style={{ flex: 1, paddingTop: 100, paddingHorizontal: 20, gap: 8 }}>
      {/* Three lines above the triggers, as `popover-rows.tsx` has, so the
          lists sit about where its list does. */}
      <Text>{`page scrolled: ${page.offset() > 0 ? "yes" : "no"}`}</Text>
      <Text>{`overlay scrolled: ${overlay.offset() > 0 ? "yes" : "no"}`}</Text>
      <Text>{`popover scrolled: ${popover.offset() > 0 ? "yes" : "no"}`}</Text>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <View ref={anchor} role="button" onPress={() => shown.set(!shown())} style={{ height: 44, width: 120, justifyContent: "center" }}>
          <Text>Show overlay</Text>
        </View>
        <Popover
          open={open()}
          onOpenChange={(e) => open.set(e.open)}
          placement="bottom"
          align="start"
          trigger={
            <View role="button" style={{ height: 44, width: 120, justifyContent: "center" }}>
              <Text>Open popover</Text>
            </View>
          }
          popupStyle={{ width: 240, backgroundColor: "#ffffff", borderRadius: 8 }}
        >
          <Rows id="popover-list" list={popover} />
        </Popover>
      </View>
      {/* Right under the triggers, where the anchored lists open. */}
      <View style={{ marginTop: -8 }}>
        <Rows id="page-list" list={page} />
      </View>
      <If when={shown()}>
        <Overlay anchor={anchor} placement="bottom" align="start">
          <Rows id="overlay-list" list={overlay} />
        </Overlay>
      </If>
      <Text>{`offsets: page ${page.offset()}, overlay ${overlay.offset()}, popover ${popover.offset()}`}</Text>
      <Text>{`page events: ${page.events().join(", ") || "none"}`}</Text>
      <Text>{`overlay events: ${overlay.events().join(", ") || "none"}`}</Text>
      <Text>{`popover events: ${popover.events().join(", ") || "none"}`}</Text>
    </View>
  );
}

mount(App);
