#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    // Save PNG. A WebView has no download handler for `<a download>` (Android
    // in particular drops the click silently), so the frontend asks the OS for
    // a destination through the dialog plugin and writes the bytes through the
    // fs plugin — see src/utils/save-png.ts. The dialog adds the chosen path to
    // the fs scope at runtime, so no static scope is configured.
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Coral is a live-microphone app. On Linux the WebKitGTK webview refuses
      // getUserMedia by default: it emits a `permission-request` signal that
      // nothing answers, so the request is denied (unlike a browser, which
      // prompts). Turn on the media-stream setting and auto-grant the request
      // so the mic works in the packaged app exactly as it does on the web.
      #[cfg(target_os = "linux")]
      {
        use tauri::Manager;
        use webkit2gtk::{PermissionRequestExt, SettingsExt, WebViewExt};
        if let Some(window) = app.get_webview_window("main") {
          window.with_webview(|webview| {
            let wv = webview.inner();
            if let Some(settings) = WebViewExt::settings(&wv) {
              settings.set_enable_media_stream(true);
            }
            wv.connect_permission_request(|_, request| {
              request.allow();
              true
            });
          })?;
        }
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
