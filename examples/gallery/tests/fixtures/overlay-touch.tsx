import { mount, Overlay, signal, Text, View } from "nsh";

function App() {
  const log = signal("");
  const record = (event: string) => log.set(log() ? `${log()} > ${event}` : event);
  return (
    <View style={{ flex: 1, paddingTop: 80 }}
      onTouchStartCapture={() => record("root capture")}
      onTouchStart={() => record("root bubble")}>
      <View testID="outside" style={{ height: 60 }}
        onPress={() => record("outside press")}>
        <Text>Outside</Text>
      </View>
      <Overlay style={{ top: 200, left: 20, width: 260, height: 60 }}>
        <View testID="strip" style={{ flex: 1, backgroundColor: "red" }}
          onTouchStartCapture={() => record("strip capture")}
          onTouchStart={() => record("strip bubble")}
          onPress={() => record("strip press")}>
          <Text>Overlay strip</Text>
          <Overlay style={{ top: 300, left: 20, width: 260, height: 60 }}>
            <View testID="inner" style={{ flex: 1 }}
              onTouchStartCapture={() => record("inner capture")}
              onTouchStart={() => record("inner bubble")}
              onPress={() => record("inner press")}>
              <Text>Nested overlay</Text>
            </View>
          </Overlay>
        </View>
      </Overlay>
      <Text>{log()}</Text>
    </View>
  );
}

mount(App);
