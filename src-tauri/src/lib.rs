// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
mod alert;
mod annotate;
mod app_accent;
mod app_locale;
mod app_windows;
mod audio_preview;
mod camera_frames;
mod camera_preview;
mod capture_overlays;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cursor_scrub;
mod editor;
mod fault;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod glide;
mod image_analysis;
mod invoke_handler;
mod moments;
mod monitor_topology;
mod osc;
mod permissions;
mod plugins;
mod project_browser;
mod projects_folder;
mod recording;
mod recording_inputs;
mod recording_sources;
mod ruler;
mod screenshots;
mod settings;
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
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  #[cfg(target_os = "macos")]
  screenwide_i18n::apply_layout_direction();
  let builder = plugins::with_plugins(tauri::Builder::default());
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let builder = builder.manage(glide::settings::GlideSettingsState::default());
  let app = builder
    .manage(annotate::AnnotateState::default())
    .manage(audio_preview::AudioPreviewState::default())
    .manage(camera_preview::CameraPreviewState::default())
    .manage(alert::AlertState::default())
    .manage(editor::EditorState::default())
    .manage(editor::recording_preview_player::RecordingPreviewPlayerState::default())
    .manage(editor::screenshot_preview::ScreenshotPreviewState::default())
    .manage(permissions::PermissionState::default())
    .manage(recording::RecordingState::default())
    .manage(recording::ReplayState::default())
    .manage(ruler::RulerState::default())
    .manage(settings::GeneralSettingsState::default())
    .manage(shortcuts::ShortcutSettingsState::default())
    .manage(text_recognition::TextRecognitionState::default())
    .manage(text_recognition::qr_details::QrDetailsState::default())
    .manage(tooltip_window::TooltipState::default())
    .invoke_handler(invoke_handler::invoke_handler())
    .setup(startup::setup)
    .build(tauri::generate_context!())
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
