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

// Web entry point. Renders the existing dashboard in the browser with the DSP
// stubbed: the concurrency bridges are built so the UI has its event/telemetry/
// profile handles, but nothing writes the profile yet, so the dashboard sits in
// "SEARCHING" until the Web Audio capture/synthesis layer is wired.
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    use voice_harmonic_engine::{ConcurrencyBridges, DashboardApp};

    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no global window")
            .document()
            .expect("no document on window");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("missing element #the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#the_canvas_id is not a <canvas>");

        let bridges = ConcurrencyBridges::new();
        let event_tx = bridges.event_tx;
        let telemetry = bridges.telemetry.clone();
        let ui_profile_rx = bridges.ui_profile_rx;

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(move |cc| {
                    Ok(Box::new(DashboardApp::new(
                        cc,
                        event_tx,
                        telemetry,
                        ui_profile_rx,
                    )))
                }),
            )
            .await
            .expect("failed to start eframe web runner");
    });
}
