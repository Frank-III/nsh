import { mount, signal, Text, View } from "nsh";
import { Input } from "nsh/base";

// The field's box raises itself while the field has focus and moves below
// and above the cover with each key; the cover lies over the box's spot, so
// a press there lands on whichever of the two is on top.
function App() {
  const focused = signal(false);
  const blurs = signal(0);
  const z = signal(0);
  const pressed = signal("none");
  return (
    <View style={{ flex: 1, paddingTop: 80 }}>
      <View style={{ zIndex: z() }}>
        <Input
          testID="field"
          placeholder="Focus and type"
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
        <View testID="field-spot" role="button" onPress={() => pressed.set("field")} style={{ height: 20 }} />
      </View>
      <View
        testID="cover"
        role="button"
        onPress={() => pressed.set("cover")}
        style={{ zIndex: 1, height: 20, marginTop: -20, backgroundColor: "red" }}
      />
      <Text>{`focused: ${focused()}, blurs: ${blurs()}`}</Text>
      <Text>{`pressed: ${pressed()}`}</Text>
      <Input testID="other" placeholder="Move focus here" style={{ height: 44 }} />
    </View>
  );
}

mount(App);
