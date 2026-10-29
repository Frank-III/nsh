import { keymap, mount, signal, Text, View } from "nsh";
import { Input, Sheet } from "nsh/base";

// Shortcuts heard with nothing focused, in an input, and in a presented sheet.
function App() {
  const log = signal("");
  const sheet = signal(false);
  const hit = (name: string) => () => log.set(`${log()} ${name}`.trim());
  keymap.bind("mod+k", hit("mod+k"), { when: "always" });
  keymap.bind("up", hit("up"));
  keymap.bind("down", hit("down"));
  keymap.bind("escape", hit("escape"));
  return <View style={{ flex: 1, paddingTop: 80 }}>
    <Input testID="field" style={{ height: 40 }} />
    <Text testID="open" onPress={() => sheet.set(true)}>open sheet</Text>
    <Text>{`keys: ${log()}`}</Text>
    <Sheet open={sheet()} onOpenChange={(open) => sheet.set(open)} popupStyle={{ padding: 24, minHeight: 300 }}>
      <Input testID="sheet-field" style={{ height: 40 }} />
      <Text>{`sheet keys: ${log()}`}</Text>
    </Sheet>
  </View>;
}

mount(App);
