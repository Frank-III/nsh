# Interactive pop

A manual iOS navigation check with a toggle and a long-press chart scrub.
`ios={{ interactivePopEnabled }}` defaults to true and affects iOS and Mac
Catalyst. False pauses both pop recognizers. True restores each recognizer's
saved UIKit state; it leaves untouched recognizers alone. Content swipes keep
the system's behavior, which can differ between iPhone and iPad.

On a Mac with Xcode, from this checkout:

```sh
CARGO_BUILD_JOBS=4 cargo build -p nsh-cli
CARGO_BUILD_JOBS=4 cargo build -p nsh-ios-host --target aarch64-apple-ios-sim
CARGO_BUILD_JOBS=4 cargo check -p nsh-platform-ios --target aarch64-apple-ios-sim
CARGO_BUILD_JOBS=4 cargo clippy -p nsh-platform-ios --target aarch64-apple-ios-macabi --all-targets
./target/debug/nsh test crates/nsh-platform-ios/tests/navigation/InteractivePop.test.tsx --target ios
./target/debug/nsh run examples/interactive-pop --target ios
```

Use an iOS 26+ iPhone simulator with content pop enabled by UIKit for the six automated cases. Older iOS versions have
only the edge recognizer. The automated tests need real UIKit input; headless
runs do not test gesture recognition.

1. Open chart. Swipe right from the left edge below the controls. It returns
   to the root. Open chart again and repeat from the middle of the content.
   Compare that swipe with base on the same device: if UIKit leaves it disabled
   there, it must stay disabled here too.
2. Open chart, tap Toggle swipe back, and wait for Swipe back: disabled.
   Both swipes leave the page in place. The native back button still returns.
3. Open chart. Toggle swipe back off, then on. Both swipes regain their prior behavior.
   Repeat with Toggle header hiding the header.
4. With swipe back enabled, hold the chart for at least 400 ms until Scrubbing
   appears. Drag horizontally across the bars without lifting. The cursor
   follows and the page stays. Lift: Hold to scrub and Swipe back: enabled
   return. A new edge swipe below the chart pops. Repeat with the toggle disabled;
   release must leave swipe back disabled.
5. Disable swipe back and Push cover page. Swipe the cover away: the chart
   still says disabled and rejects both swipes. Repeat starting with the
   chart enabled; use Toggle covered chart on the cover. The cover must still
   pop, revealing a disabled chart.
6. With swipe enabled, start a short edge swipe and return to its starting
   position before lifting to cancel. The page stays and the next complete
   swipe works. Repeat on the cover above a disabled chart: cancel leaves the
   cover swipeable; completing reveals the disabled chart.
7. While disabled, Programmatic back still works. At the root, both swipe
   positions keep the root in place. Scroll the root's horizontal card row left
   and right, including a drag starting near the left edge once the row has
   scrolled. The cards must move while Depth stays 1, as on base.

For native red/green, reuse the existing complete base and fix checkouts.
Copy this example directory and `InteractivePop.test.tsx` to the base checkout,
rebuild its CLI and iOS host, and run the same test command. The disabled-swipe
assertions must fail on the base and pass on the fix. Reusing a host binary
from the other checkout invalidates that comparison.
