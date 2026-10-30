// Controls for `popover-touch.test.tsx`'s scroll: the same list scrolled
// on the page, in a bare `Overlay` and in a `Popover`, each by the same
// 400 pt (from the list's middle the finger leaves the 300 pt list and
// the window's top) and by 100 pt (the finger stays in the list). With
// `--target ios` / `android` the steps are real touches; headless runs
// scroll the named view directly and pass either way.
import { open, test } from "nsh/test";

/** The fixture with the list `where` names on screen. */
async function opened(where: "page" | "overlay" | "popover") {
  const app = await open("./fixtures/popover-control.tsx");
  if (where === "overlay") {
    await app.press("Show overlay");
  } else if (where === "popover") {
    await app.press("Open popover");
  }
  await app.waitFor(`id:${where}-list`);
  await app.waitFor(`text:${where} scrolled: no`);
  return app;
}

for (const where of ["page", "overlay", "popover"] as const) {
  for (const distance of [400, 100]) {
    test(`a ${distance} pt drag scrolls a list in the ${where === "page" ? "page" : `${where}`}`, async () => {
      const app = await opened(where);
      await app.scroll(`id:${where}-list`, distance);
      await app.waitFor(`text:${where} scrolled: yes`);
    });
  }
}
