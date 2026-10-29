import { mount, signal, Text, View } from "nsh";
import { Input } from "nsh/base";

// The field's box raises itself while the field has focus, and each key
// moves it below or above its two siblings; the blur lowers it again.
function App() {
  const focused = signal(false);
  const blurs = signal(0);
  const z = signal(0);
  return (
    <View style={{ flex: 1, paddingTop: 80, padding: 20 }}>
      <View style={{ zIndex: z() }}>
        <Input
          testID="field"
          placeholder="Focus and type abc"
          autoCapitalize="none"
          autoCorrect={false}
          style={{ height: 44 }}
          onFocus={() => {
            focused.set(true);
            z.set(3);
          }}
          onBlur={() => {
            focused.set(false);
            blurs.set(blurs() + 1);
            z.set(0);
          }}
          onChange={({ value }) => z.set(value.length % 2 === 0 ? 3 : -1)}
        />
      </View>
      <View style={{ zIndex: 1, height: 20, backgroundColor: "red" }} />
      <View style={{ zIndex: 2, height: 20, backgroundColor: "blue" }} />
      <Text>{`focused: ${focused()}, blurs: ${blurs()}`}</Text>
      <Input testID="other" placeholder="Move focus here" style={{ height: 44 }} />
    </View>
  );
}

mount(App);
