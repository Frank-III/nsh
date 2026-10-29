import { open, test } from "nsh/test";

// Hardware key commands reach the app's keymap on iOS (an iPad's keyboard):
// with nothing focused, in a focused input, and in a presented sheet.
// `nsh test --target ios` sends the keys as IOHID keyboard events, as a
// real keyboard's, without Simulator.app's keyboard capture.

test("keys with nothing focused", async () => {
  const app = await open("./fixtures/ios-keys.tsx");
  await app.keyboard.press("mod+k");
  await app.keyboard.press("down");
  await app.keyboard.press("up");
  await app.waitFor("text:keys: mod+k down up");
});

test("keys in a focused input", async () => {
  const app = await open("./fixtures/ios-keys.tsx");
  await app.type("id:field", "x");
  await app.keyboard.press("down");
  await app.keyboard.press("mod+k");
  await app.waitFor("text:keys: mod+k");
});

test("keys in a presented sheet", async () => {
  const app = await open("./fixtures/ios-keys.tsx");
  await app.press("id:open");
  await app.waitFor("id:sheet-field");
  await app.keyboard.press("mod+k");
  await app.keyboard.press("down");
  await app.waitFor("text:sheet keys: mod+k down");
  await app.type("id:sheet-field", "y");
  await app.keyboard.press("mod+k");
  await app.waitFor("text:sheet keys: mod+k down mod+k");
});

test("keys after a field resigns", async () => {
  const app = await open("./fixtures/ios-keys.tsx");
  await app.type("id:field", "x");
  await app.waitFor("text:field focused");
  await app.press("id:blur");
  await app.waitFor("text:field blurred");
  await app.keyboard.press("down");
  await app.waitFor("text:keys: down");
});

test("keys after Escape dismisses a sheet", async () => {
  const app = await open("./fixtures/ios-keys.tsx");
  await app.press("id:open");
  await app.waitFor("id:sheet-field");
  await app.keyboard.press("escape");
  await app.waitFor("text:sheet closed");
  await app.keyboard.press("down");
  await app.waitFor("text:keys: down");
});
