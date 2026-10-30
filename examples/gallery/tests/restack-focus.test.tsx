import type { App } from "nsh/test";
import { expect, open, test } from "nsh/test";

// A tap focuses an input on a native host, the path the fix is about;
// headless, a press takes no input, so the field takes the focus as a
// typing step gives it.
async function tap(app: App, selector: string) {
  try {
    await app.press(selector);
  } catch {
    await app.type(selector, "");
  }
}

test("restacking a focused field's ancestor preserves keyboard focus", async () => {
  const app = await open("./fixtures/restack-focus.tsx");
  await tap(app, "id:field");
  await app.waitFor("focused: true, blurs: 0");
  // Keys go to the current responder; app.type would refocus the field.
  for (const key of ["a", "b", "c"]) {
    await app.keyboard.press(key);
    await app.wait(100);
    await app.waitFor("focused: true, blurs: 0");
  }
  expect(app.value("id:field")).toBe("abc");
  await tap(app, "id:other");
  await app.waitFor("focused: false, blurs: 1");
});

test("restacking a focused field's ancestor keeps the stacking order", async () => {
  const app = await open("./fixtures/restack-stacking.tsx");
  // Each key moves the field's box below or above the cover.
  await app.type("id:field", "a");
  await app.waitFor("focused: true, blurs: 0");
  // The next keys go to the current responder: app.type would press the
  // field again.
  for (const key of ["b", "c", "d"]) {
    await app.keyboard.press(key);
    await app.waitFor("focused: true, blurs: 0");
  }
  expect(app.value("id:field")).toBe("abcd");
  // The box is on top: a finger on the spot the cover lies over presses the
  // box (on a native host the platform hit-tests its own subview order).
  await app.press("id:field-spot");
  await app.waitFor("pressed: field");
  await app.type("id:other", "x");
  await app.waitFor("focused: false, blurs: 1");
  // Lowered on blur, the box is under the cover again.
  await app.press("id:cover");
  await app.waitFor("pressed: cover");
});
