//! `ios={{ panScrollHandoff }}` reaches `ViewProps.pan_scroll_handoff`
//! when the environment is iOS. The renderer sends only the current
//! platform's object, so the same JSX leaves the field false on Android.
//! Android's pans already yield inside `Box.scrollTakes` and that
//! backend does not read the field.

use std::time::Duration;

use nsh_core::{ApproxTextEngine, MountItem, Size, geometry::px};
use nsh_engine::{Bundle, Engine, EngineConfig, Environment, Idiom, Os};

use crate::ChannelSink;

const APP: &str = r#"import { mount, View } from "nsh";

function App() {
  return (
    <View>
      <View testID="sheet" ios={{ panScrollHandoff: true }} style={{ height: 40 }} />
      <View testID="still" ios={{ panScrollHandoff: false }} style={{ height: 40 }} />
      <View testID="plain" style={{ height: 40 }} />
    </View>
  );
}

mount(App);
"#;

/// `testID` to `pan_scroll_handoff` on the first frame, for `os`.
fn handoff_on(os: Os) -> Vec<(String, bool)> {
    let dir = std::env::temp_dir().join(format!(
        "nsh-pan-handoff-{}-{}",
        std::process::id(),
        os.name()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let entry = dir.join("App.tsx");
    std::fs::write(&entry, APP).unwrap();
    let bundle = nsh_bundler::bundle(&nsh_bundler::BundleOptions {
        entry,
        minify: false,
        split: false,
        ui_sets: vec![nsh_base::registry().into()],
    })
    .unwrap();
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: nsh_engine::App::Js(Bundle::source(bundle.code)),
        viewport: Size::new(px(390.0), px(844.0)),
        trace: false,
        network: None,
        websocket: None,
        refuse_connections: None,
        filesystem: None,
        assets: None,
        environment: Environment {
            os,
            idiom: Idiom::Phone,
            ..Environment::default()
        },
        header_heights: (nsh_core::Points(44.0), nsh_core::Points(96.0)),
        limits: nsh_engine::Limits::default(),
        ui_sets: Vec::new(),
        runtime: None,
        data_dir: None,
        cache: nsh_engine::CacheOptions::off(),
        storage: None,
        clock: Default::default(),
        context: Default::default(),
        pinned: Default::default(),
        clipboard: None,
        host: Default::default(),
        picker: None,
        speech: None,
    };
    let _engine = Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap();
    let batch = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("first frame");
    let mut rows = Vec::new();
    for item in &batch.items {
        if let MountItem::ViewProps { props, .. } = item
            && let Some(id) = props.test_id.as_deref()
        {
            rows.push((id.to_owned(), props.pan_scroll_handoff));
        }
    }
    rows.sort();
    rows
}

#[test]
fn ios_pan_scroll_handoff_reaches_the_view_props() {
    assert_eq!(
        handoff_on(Os::Ios),
        vec![
            ("plain".to_owned(), false),
            ("sheet".to_owned(), true),
            ("still".to_owned(), false),
        ]
    );
}

#[test]
fn android_leaves_pan_scroll_handoff_unset() {
    assert_eq!(
        handoff_on(Os::Android),
        vec![
            ("plain".to_owned(), false),
            ("sheet".to_owned(), false),
            ("still".to_owned(), false),
        ]
    );
}
