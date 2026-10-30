import { mount, signal, Text, View } from "nsh";
import { Input } from "nsh/base";

// Accessibility labels of inputs, checked with Accessibility Inspector on
// iOS: the editor (UITextField / UITextView) carries the label, after a
// label update, a clear, and a switch between one line and several.
function App() {
  const label = signal<string | undefined>("Title");
  const multiline = signal(false);
  const toggle = (text: string, press: () => void) => (
    <Text role="button" onPress={press} style={{ padding: 8 }}>{text}</Text>
  );
  return <View style={{ flex: 1, paddingTop: 80, gap: 12, paddingHorizontal: 16 }}>
    <Input testID="title" accessibilityLabel={label()} multiline={multiline()} style={{ height: multiline() ? 96 : 40 }} />
    <Input testID="notes" accessibilityLabel="Notes" multiline style={{ height: 96 }} />
    <Text>{`label: ${label() ?? "none"}, multiline: ${multiline()}`}</Text>
    {toggle("Rename", () => label.set("Meeting title"))}
    {toggle("Clear label", () => label.set(undefined))}
    {toggle("Toggle multiline", () => multiline.set(!multiline()))}
  </View>;
}

mount(App);
