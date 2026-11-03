// Exercises NavPage ios={{ interactivePopEnabled }} through the example.
// Real UIKit touches required: run with --target ios on iOS 26 or later.
import { expect, open, test } from "nsh/test";

const entry = "../../../../examples/interactive-pop/App.tsx";

for (const header of ["visible", "hidden"]) {
  for (const start of ["edge", "content"]) {
    test(`interactive pop toggles reactively from ${start}, header ${header}`, async () => {
      const app = await open(entry);
      await app.press("Open chart");
      await app.wait(800);
      if (header === "hidden") await app.press("Toggle header");
      await app.press("Toggle swipe back");
      await app.waitFor("Swipe back: disabled");
      await app.drag(`id:${start}`, { x: 280, y: 0 });
      await app.wait(800);
      expect(app.shows("Depth: 2")).toBe(true);
      await app.press("Toggle swipe back");
      await app.waitFor("Swipe back: enabled");
      await app.drag(`id:${start}`, { x: 280, y: 0 });
      await app.waitFor("Depth: 1");
    });
  }
}

test("a covered page cannot disable the top page, and regains its own setting", async () => {
  const app = await open(entry);
  await app.press("Open chart");
  await app.wait(800);
  await app.press("Push cover page");
  await app.wait(800);
  await app.press("Toggle covered chart");
  await app.drag("id:content", { x: 280, y: 0 });
  await app.waitFor("Depth: 2");
  await app.wait(800);
  expect(app.shows("Swipe back: disabled")).toBe(true);
  await app.drag("id:edge", { x: 280, y: 0 });
  await app.wait(800);
  expect(app.shows("Depth: 2")).toBe(true);
  // The native back button is independent of the recognizers.
  await app.back();
  await app.waitFor("Depth: 1");
});

test("cancelling a pop keeps the cover enabled above a disabled chart", async () => {
  const app = await open(entry);
  await app.press("Open chart");
  await app.wait(800);
  await app.press("Toggle swipe back");
  await app.press("Push cover page");
  await app.wait(800);
  await app.drag("id:edge", { x: 20, y: 0 });
  await app.wait(800);
  expect(app.shows("Depth: 3")).toBe(true);
  await app.drag("id:edge", { x: 280, y: 0 });
  await app.waitFor("Depth: 2");
  await app.wait(800);
  expect(app.shows("Swipe back: disabled")).toBe(true);
  await app.drag("id:content", { x: 280, y: 0 });
  await app.wait(800);
  expect(app.shows("Depth: 2")).toBe(true);
});
