// Compatibility of the iOS-only option with normal back navigation.
// UIKit's recognizers are covered by the platform's InteractivePop tests;
// the headless compat recorder does not expose their enabled state.
import { signal, Text, View } from "nsh";
import { Button, createNavStack, NavPage, NavStack } from "nsh/base";

export default function App() {
  const nav = createNavStack(["root", "detail"]);
  const enabled = signal(false);
  return <View style={{ flex: 1 }}>
    <Text>{`Pop enabled: ${enabled()}`}</Text>
    <Text>{`Pop depth: ${nav.routes().length}`}</Text>
    <Button onPress={() => enabled.set(!enabled())}><Text>Toggle iOS pop</Text></Button>
    <Button onPress={() => nav.pop()}><Text>Pop programmatically</Text></Button>
    <NavStack state={nav} style={{ flex: 1 }}>
      {route => <NavPage testID={`interactive-${route}`} ios={{ interactivePopEnabled: enabled() }}>
        <Text>{`Pop page: ${route}`}</Text>
      </NavPage>}
    </NavStack>
  </View>;
}
