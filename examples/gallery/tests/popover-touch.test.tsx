// A popover's content takes touches on phones and tablets: there its
// backdrop, which takes the presses outside it, lies below the popup. The
// steps are real touches with `--target ios` / `android`, which land on
// what is on top; headless runs press the view a selector names.
import { expect, open, test } from "nsh/test";

/** The fixture with its popover open. */
async function opened() {
  const app = await open("./fixtures/popover-rows.tsx");
  await app.press("Open menu");
  await app.waitFor("text:changes: opened by triggerPress");
  await app.waitFor("Row 1");
  return app;
}

test("a row of an open popover takes the press", async () => {
  const app = await opened();
  await app.press("Row 2");
  await app.waitFor("text:pressed: Row 2");
  // The press was the row's only: the popover stays open.
  await app.wait(300);
  expect(app.texts()).toContain("changes: opened by triggerPress");
  expect(app.texts()).toContain("Row 2");
});

test("a drag in an open popover scrolls its content", async () => {
  const app = await open("./fixtures/popover-control.tsx");
  await app.press("Open popover");
  await app.waitFor("id:popover-list");
  await app.waitFor("text:popover scrolled: no");
  await app.scroll("id:popover-list", 100);
  await app.waitFor("text:popover scrolled: yes");
});

test("a maxHeight popover list measures 300 pt and scrolls", async () => {
  const app = await opened();
  await app.waitFor("text:list height: 300");
  await app.scroll("id:menu-list", 100);
  await app.waitFor("text:scrolled: yes");
  expect(app.texts()).toContain("changes: opened by triggerPress");
});

test("a press outside the popup closes it and presses nothing in it", async () => {
  const app = await opened();
  // The trigger is outside the popup: on a touch screen the backdrop over
  // it takes the press (`pressOutside`); headless runs press the trigger
  // itself, which closes it too (`triggerPress`).
  await app.press("Open menu");
  await app.waitFor("Row 1", { state: "gone" });
  const changes = app.texts().find((t) => t.startsWith("changes: "));
  expect(["changes: opened by triggerPress, closed by pressOutside", "changes: opened by triggerPress, closed by triggerPress"]).toContain(changes);
  expect(app.texts()).toContain("pressed: none");
});
