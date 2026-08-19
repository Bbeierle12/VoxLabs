//! Binary entry points. The real work lives in the `voice_harmonic_engine`
//! library crate; this file is only the thin per-platform `main`.

// Desktop: hand off to the shared native runner in the library.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn main() -> anyhow::Result<()> {
    voice_harmonic_engine::run()
}

// Android: the real entry point is `android_main` inside the library's `android`
// module, loaded from the cdylib by NativeActivity — the binary target is unused
// on Android. A `[[bin]]` still needs a `main`, so provide an empty stub.
#[cfg(target_os = "android")]
fn main() {}

/// Id of the `<canvas>` in `index.html` that eframe renders into.
#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "the_canvas_id";

// Web entry point. Renders the existing dashboard in the browser with the DSP
// stubbed: the concurrency bridges are built so the UI has its event/telemetry/
// profile handles, but nothing writes the profile yet, so the dashboard sits in
// "SEARCHING" until the Web Audio capture/synthesis layer is wired.
//
// Every startup step returns an error rather than panicking: a panic here left
// the user staring at the blank black canvas with the reason visible only in
// the devtools console. `report_startup_failure` puts it on the page.
#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async {
        if let Err(msg) = start_web().await {
            report_startup_failure(&msg);
        }
    });
}

#[cfg(target_arch = "wasm32")]
async fn start_web() -> Result<(), String> {
    use eframe::wasm_bindgen::JsCast as _;
    use voice_harmonic_engine::{ConcurrencyBridges, DashboardApp};

    let document = web_sys::window()
        .ok_or("no global `window` — is this running in a browser?")?
        .document()
        .ok_or("no `document` on `window`")?;
    let canvas = document
        .get_element_by_id(CANVAS_ID)
        .ok_or_else(|| format!("missing element #{CANVAS_ID} in the page"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| format!("#{CANVAS_ID} is not a <canvas> element"))?;

    let bridges = ConcurrencyBridges::new();
    let event_tx = bridges.event_tx;
    let telemetry = bridges.telemetry.clone();
    let ui_profile_rx = bridges.ui_profile_rx;
    let spectrum_rx = bridges.spectrum_rx;
    let scope_rx = bridges.scope_rx;

    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(move |cc| {
                Ok(Box::new(DashboardApp::new(
                    cc,
                    event_tx,
                    telemetry,
                    ui_profile_rx,
                    spectrum_rx,
                    scope_rx,
                    // Web has no analysis thread; a nominal rate keeps the
                    // spectrogram bin→Hz map well-formed (it stays idle).
                    48_000.0,
                    // No filesystem in the browser: the archive lives only
                    // for this session (disk persistence is native-only).
                    None,
                )))
            }),
        )
        .await
        .map_err(|e| format!("eframe web runner failed to start: {e:?}"))
}

/// Renders a startup failure onto the page instead of leaving a blank canvas.
///
/// Uses `set_text_content`, not `set_inner_html`, so the error string — which
/// can carry arbitrary text out of eframe or the browser — is displayed, never
/// parsed as markup. Every DOM step is best-effort: if even the body is
/// missing, the console line is all we can manage.
#[cfg(target_arch = "wasm32")]
fn report_startup_failure(msg: &str) {
    web_sys::console::error_1(&format!("Voice Harmonic Engine failed to start: {msg}").into());

    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    let Ok(banner) = document.create_element("div") else {
        return;
    };
    let _ = banner.set_attribute(
        "style",
        "position:absolute; inset:0; z-index:1; display:flex; align-items:center; \
         justify-content:center; padding:24px; box-sizing:border-box; text-align:center; \
         white-space:pre-line; font:14px/1.5 system-ui, sans-serif; color:#e6f6f8; \
         background:#0a0a0a;",
    );
    banner.set_text_content(Some(&format!(
        "Voice Harmonic Engine failed to start.\n{msg}"
    )));
    let _ = body.append_child(&banner);
}
