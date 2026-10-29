// A row with a press and a `panActivation` pan: a tap presses it, a swipe
// that activates the pan is the pan's alone and presses nothing on lift,
// on every platform.
import { expect, open, test } from "nsh/test";

test("a tap presses the row and its label", async () => {
  const app = await open("./fixtures/pan-gate.tsx");
  await app.press("text:row");
  await app.waitFor("rows: 1");
  await app.press("text:label");
  await app.waitFor("labels: 1");
  expect(app.shows("rows: 1")).toBe(true);
});

test("a sideways swipe across the row pans it and presses nothing", async () => {
  const app = await open("./fixtures/pan-gate.tsx");
  await app.drag("text:row", { x: 80, y: 0 });
  await app.waitFor("pan: released");
  await app.wait(300);
  expect(app.shows("rows: 0")).toBe(true);
});

test("a sideways swipe from the label pans the row and presses neither", async () => {
  const app = await open("./fixtures/pan-gate.tsx");
  await app.drag("text:label", { x: 80, y: 0 });
  await app.waitFor("pan: released");
  await app.wait(300);
  expect(app.shows("labels: 0")).toBe(true);
  expect(app.shows("rows: 0")).toBe(true);
});
