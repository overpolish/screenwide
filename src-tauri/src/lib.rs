// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
mod audio_preview;
mod camera_format;
#[cfg(target_os = "macos")]
mod camera_frame_rate;
mod camera_frames;
#[cfg(target_os = "windows")]
mod camera_power_line;
mod camera_preview;
mod capture_geometry;
#[cfg(target_os = "macos")]
mod capture_kit;
mod capture_overlays;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cursor_scrub;
mod desktop_capture;
mod exports;
mod image_analysis;
mod osc;
mod permissions;
mod recording;
mod recording_inputs;
mod recording_sources;
mod ruler;
mod screenshots;
mod settings;
mod shortcuts;
#[cfg(debug_assertions)]
mod storybook_native;
mod text_recognition;
#[cfg(desktop)]
mod tray;
mod updates;
mod windows;
use tauri::Manager;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let builder = tauri::Builder::default()
    .plugin(tauri_plugin_autostart::init(
      tauri_plugin_autostart::MacosLauncher::LaunchAgent,
      None,
    ))
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(
      tauri_plugin_window_state::Builder::default()
        .with_state_flags(tauri_plugin_window_state::StateFlags::POSITION)
        .with_filter(|label| label == windows::WindowLabel::RecordingBar.as_str())
        .skip_initial_state(windows::WindowLabel::RecordingBar.as_str())
        .build(),
    );
  #[cfg(target_os = "macos")]
  let builder = builder
    .plugin(tauri_plugin_macos_permissions::init())
    .plugin(tauri_nspanel::init());
  let app = builder
    .manage(audio_preview::AudioPreviewState::default())
    .manage(camera_preview::CameraPreviewState::default())
    .manage(exports::ExportState::default())
    .manage(exports::recording_preview_player::RecordingPreviewPlayerState::default())
    .manage(exports::screenshot_preview::ScreenshotPreviewState::default())
    .manage(permissions::PermissionState::default())
    .manage(recording::RecordingState::default())
    .manage(ruler::RulerState::default())
    .manage(settings::GeneralSettingsState::default())
    .manage(shortcuts::ShortcutSettingsState::default())
    .manage(text_recognition::TextRecognitionState::default())
    .manage(text_recognition::qr_details::QrDetailsState::default())
    .invoke_handler(tauri::generate_handler![
      audio_preview::start_audio_preview,
      audio_preview::stop_audio_preview,
      camera_preview::start_camera_preview,
      camera_preview::stop_camera_preview,
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      cursor_scrub::begin_cursor_scrub,
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      cursor_scrub::end_cursor_scrub,
      exports::commands::browse_export_directory,
      exports::commands::cancel_export,
      exports::commands::cancel_export_job,
      exports::commands::copy_export_to_clipboard,
      exports::commands::focus_export_window,
      exports::commands::get_screenshot_content_bounds,
      exports::recording_preview_player::recenter::get_recording_content_bounds,
      exports::preview::estimate_recording_export,
      exports::preview::get_export_snapshot,
      exports::recording_preview::get_recording_preview,
      exports::recording_preview::get_recording_keyboard_timeline,
      exports::recording_preview_player::commands::pause_recording_preview,
      exports::recording_preview_player::surface_commands::layout_recording_preview_surface,
      exports::recording_preview_player::surface_commands::set_recording_preview_zoom,
      exports::recording_preview_player::commands::playback::play_recording_preview,
      exports::recording_preview_player::commands::seek_recording_preview,
      exports::recording_preview_player::commands::select_recording_preview_audio,
      exports::recording_preview_player::commands::set_recording_preview_audio_volumes,
      exports::recording_preview_player::commands::set_recording_preview_cursor_effects,
      exports::recording_preview_player::keyboard_command::set_recording_preview_keyboard_effects,
      exports::recording_preview_player::keyboard_command::set_recording_preview_deleted_keyboard_shortcuts,
      exports::recording_preview_player::commands::set_recording_preview_composition,
      exports::recording_preview_player::commands::start_recording_preview_player,
      exports::recording_preview_player::commands::stop_recording_preview_player,
        exports::recording_preview_player::timeline_thumbnails::copy_recording_preview_frame_to_clipboard,
      exports::recording_preview_player::timeline_thumbnails::stream_recording_timeline_thumbnails,
      exports::save::save_export,
      exports::screenshot_preview::layout_screenshot_preview_surface,
      exports::screenshot_preview::refresh_screenshot_preview_sources,
      exports::screenshot_preview::set_screenshot_preview_zoom,
      exports::screenshot_preview::start_screenshot_preview,
      exports::screenshot_preview::stop_screenshot_preview,
      exports::commands::set_export_directory,
      exports::commands::set_recording_timeline_edit,
      exports::commands::set_screenshot_background_radius,
      exports::commands::set_screenshot_radius,
      permissions::open_permission_settings,
      permissions::open_permissions_window,
      permissions::permission_snapshot,
      permissions::request_permission,
      permissions::require_permissions,
      permissions::dismiss_permissions_window,
      permissions::restart_app,
      recording::commands::cancel_recording,
      recording::commands::get_recording_snapshot,
      recording::commands::pause_recording,
      recording::commands::resume_recording,
      recording::commands::start_recording_monitor,
      recording::commands::start_recording,
      recording::commands::stop_recording_monitor,
      recording::commands::stop_recording,
      recording_inputs::list_cameras,
      recording_inputs::list_microphones,
      recording_sources::list_applications,
      recording_sources::list_monitors,
      recording_sources::list_windows,
      recording_sources::resize_window,
      recording_sources::selected_window_available,
      ruler::cancel_ruler,
      ruler::set_ruler_screenshot_mode,
      screenshots::scrolling::command::capture_scrolling_still,
      screenshots::capture_still,
      text_recognition::cancel_text_recognition,
      text_recognition::close_qr_details,
      text_recognition::copy_recognition_content,
      text_recognition::get_qr_details,
      text_recognition::start_text_recognition,
      updates::update_checks_enabled,
      updates::hide_update_prompt,
      updates::show_update_prompt,
      settings::hide_settings,
      settings::preferences::browse_default_location,
      settings::preferences::get_general_settings,
      settings::preferences::set_general_settings,
      settings::show_settings,
      shortcuts::get_shortcut_settings,
      shortcuts::resume_shortcut_action,
      shortcuts::begin_shortcut_capture,
      shortcuts::end_shortcut_capture,
      shortcuts::set_shortcut_binding,
      windows::source_selector::collapse_recording_source_selector,
      windows::source_selector::get_recording_source_selector_state,
      windows::region_gesture::begin_region_selector_gesture,
      windows::finish_recording_bar_drag,
      windows::region_gesture::finish_region_selector_gesture,
      windows::dock::finish_recording_dock_drag,
      windows::dock::resize_recording_dock,
      windows::options::hide_recording_options,
      windows::options::get_recording_options_state,
      windows::hide_recording_ui,
      windows::recording_ui_visible,
      windows::toggle_recording_ui,
      windows::region::hide_region_selector,
      windows::options::hide_standalone_listbox,
      windows::region::set_recording_controls_borrowed,
      windows::source_selector::set_recording_source_selector_visible,
      windows::region::set_region_selector_opacity,
      windows::region::set_region_selector_passthrough,
      windows::region::set_screenshot_region_session,
      windows::region::show_region_selector,
      windows::screenshot_region::magnifier::prepare_screenshot_region_magnifier,
      windows::screenshot_region::osc_command::set_screenshot_region_osc,
      windows::screenshot_region::presentation::set_region_selector_osc_frame_visible,
      windows::options::show_standalone_listbox,
      windows::options::set_recording_options_content_height,
      windows::source_selector::expand_recording_source_selector,
      windows::options::toggle_recording_options,
    ])
    .setup(|app| {
      #[cfg(debug_assertions)]
      if let Some(preview_url) = std::env::var_os("SCREENWIDE_STORYBOOK_NATIVE_URL") {
        storybook_native::show(app.handle(), &preview_url.to_string_lossy())?;
        return Ok(());
      }

      #[cfg(target_os = "macos")]
      {
        exports::initialize_cursor_artwork();
      }
      #[cfg(desktop)]
      tray::initialize(app)?;
      settings::initialize(app.handle());
      let recording_bar_preview_enabled =
        cfg!(debug_assertions) && std::env::var_os("SCREENWIDE_SHOW_RECORDING_BAR").is_some();
      let show_recording_bar_on_launch = recording_bar_preview_enabled
        || settings::current(app.handle()).show_recording_bar_on_launch;

      // Create panels lazily so conversion cannot order a stale frame onscreen.
      #[cfg(not(target_os = "macos"))]
      {
        windows::initialize_recording_bar(app.handle())?;
        windows::initialize_recording_source_selector(app.handle())?;
        windows::initialize_region_selector(app.handle())?;
        windows::initialize_recording_options(app.handle())?;
        windows::initialize_standalone_listbox(app.handle())?;
        windows::initialize_recording_dock(app.handle())?;
      }
      if let Some(window) = app.get_webview_window(windows::WindowLabel::Settings.as_str()) {
        windows::initialize_normal_window(&window)?;
      }
      if let Some(window) = app.get_webview_window(windows::WindowLabel::Update.as_str()) {
        windows::initialize_normal_window(&window)?;
      }
      for label in [
        windows::WindowLabel::ExportRecording,
        windows::WindowLabel::ExportScreenshot,
      ] {
        if let Some(window) = app.get_webview_window(label.as_str()) {
          windows::initialize_export(&window)?;
        }
      }
      windows::hide_instead_of_close(app.handle(), windows::WindowLabel::RecordingBar);
      windows::hide_instead_of_close(app.handle(), windows::WindowLabel::RecordingSourceSelector);
      windows::hide_instead_of_close(app.handle(), windows::WindowLabel::RegionSelector);
      windows::hide_instead_of_close(app.handle(), windows::WindowLabel::RecordingOptions);
      windows::hide_instead_of_close(app.handle(), windows::WindowLabel::StandaloneListbox);
      windows::hide_instead_of_close(app.handle(), windows::WindowLabel::RecordingDock);
      windows::hide_instead_of_close(app.handle(), windows::WindowLabel::ExportRecording);
      windows::hide_instead_of_close(app.handle(), windows::WindowLabel::ExportScreenshot);
      windows::hide_instead_of_close(app.handle(), windows::WindowLabel::Settings);
      windows::initialize_recording_bar_position(app.handle())?;
      windows::initialize_topology_management(app.handle());
      windows::manage_recording_bar_movement(app.handle());
      windows::manage_recording_dock_movement(app.handle());
      exports::initialize(app.handle());
      let has_pending_export = exports::has_pending_workspace(app.handle());
      shortcuts::initialize(app.handle());
      windows::manage_transient_popover_dismissal(app.handle());

      #[cfg(target_os = "macos")]
      permissions::show_on_launch(
        app.handle(),
        show_recording_bar_on_launch,
        has_pending_export,
      )?;

      #[cfg(not(target_os = "macos"))]
      if show_recording_bar_on_launch && !has_pending_export {
        windows::show_recording_ui(app.handle())?;
      }
      permissions::start_watcher(app.handle().clone());

      #[cfg(target_os = "macos")]
      {
        // Native effects can order ordinary app windows during setup. Their
        // first presentation always belongs to an explicit user action.
        let app_handle = app.handle().clone();
        // Recovery may have put an artifact in one workspace; that window is
        // the one presentation the user did ask for, so it alone is spared.
        let mut labels = vec![windows::WindowLabel::Settings];
        labels.extend(
          exports::ExportKind::ALL
            .into_iter()
            .filter(|kind| !exports::has_pending_workspace_kind(app.handle(), *kind))
            .map(exports::ExportKind::window_label),
        );
        app.handle().run_on_main_thread(move || {
          for label in &labels {
            if let Some(window) = app_handle.get_webview_window(label.as_str()) {
              let _ = window.hide();
            }
          }
        })?;
      }

      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while running tauri application");

  #[cfg(target_os = "macos")]
  let mut app = app;
  #[cfg(target_os = "macos")]
  if !exports::has_pending_workspace(app.handle()) {
    app.set_dock_visibility(false);
  }
  app.run(|_, _| {});
}
