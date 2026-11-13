// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
mod annotate;
mod app_accent;
mod app_locale;
mod app_windows;
#[cfg(target_os = "macos")]
mod async_runtime;
mod audio_preview;
mod camera_preview;
#[cfg(target_os = "windows")]
mod capture_border_windows;
mod capture_overlays;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cursor_scrub;
#[cfg(target_os = "windows")]
mod desktop_icons;
#[cfg(debug_assertions)]
mod dev_identity;
mod editor;
mod fault;
mod file_browser;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod glide;
#[cfg(target_os = "windows")]
mod hidden_apps_windows;
mod image_analysis;
mod invoke_handler;
mod keep_awake;
mod launch_at_login;
mod licensing;
mod microsoft_store;
mod moments;
mod monitor_topology;
mod osc;
mod permissions;
mod plugins;
mod project_browser;
mod projects_folder;
mod precompiled_shaders {
  include!(concat!(env!("OUT_DIR"), "/precompiled_shaders.rs"));
}
mod recording;
mod recording_inputs;
mod recording_sources;
mod references;
mod ruler;
mod screenshots;
mod settings;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod settings_preview;
mod shortcuts;
mod silence;
mod startup;
#[cfg(debug_assertions)]
mod storybook_native;
mod text_recognition;
mod tooltip_window;
mod transcription;
#[cfg(desktop)]
mod tray;
mod updates;
mod user_work;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod window_front;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  #[cfg(target_os = "macos")]
  async_runtime::install();
  #[cfg(target_os = "macos")]
  screenwide_i18n::apply_layout_direction();
  let context = tauri::generate_context!();
  #[cfg(debug_assertions)]
  let context = dev_identity::apply(context);
  let builder = plugins::with_plugins(tauri::Builder::default());
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let builder = builder.manage(glide::settings::GlideSettingsState::default());
  let app = builder
    .manage(annotate::AnnotateState::default())
    .manage(audio_preview::AudioPreviewState::default())
    .manage(camera_preview::CameraPreviewState::default())
    .manage(project_browser::open_failures::OpenFailures::default())
    .manage(recording::failures::RecordingFailures::default())
    .manage(editor::EditorState::default())
    .manage(editor::recording_preview_player::RecordingPreviewPlayerState::default())
    .manage(editor::screenshot_preview::ScreenshotPreviewState::default())
    .manage(licensing::LicensingState::default())
    .manage(permissions::PermissionState::default())
    .manage(recording::RecordingState::default())
    .manage(recording::ReplayState::default())
    .manage(ruler::RulerState::default())
    .manage(settings::GeneralSettingsState::default())
    .manage(shortcuts::ShortcutSettingsState::default())
    .manage(text_recognition::TextRecognitionState::default())
    .manage(tooltip_window::TooltipState::default())
    .invoke_handler(editor::sound_library::with_commands(
      app_windows::commands::with_commands(invoke_handler::invoke_handler()),
    ))
    .setup(startup::setup)
    .build(context)
    .expect("error while running tauri application");

  #[cfg(target_os = "macos")]
  let mut app = app;
  #[cfg(target_os = "macos")]
  app.set_dock_visibility(false);
  app.run(|app, event| {
    #[cfg(target_os = "macos")]
    if let tauri::RunEvent::Opened { urls } = event {
      startup::open_files(app, urls);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, event);
  });
}
