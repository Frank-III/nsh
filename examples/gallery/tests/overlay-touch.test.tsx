import { expect, open, test } from "nsh/test";

test("overlay touches capture and bubble through their declaring ancestors", async () => {
  const app = await open("./fixtures/overlay-touch.tsx");
  await app.press("id:strip");
  expect(app.texts()).toContain("root capture > strip capture > strip bubble > root bubble > strip press");
  await app.press("id:outside");
  expect(app.texts()).toContain("root capture > strip capture > strip bubble > root bubble > strip press > root capture > root bubble > outside press");
});

test("nested overlays retain each declaring ancestor", async () => {
  const app = await open("./fixtures/overlay-touch.tsx");
  await app.press("id:inner");
  expect(app.texts()).toContain("root capture > strip capture > inner capture > inner bubble > strip bubble > root bubble > inner press");
});
