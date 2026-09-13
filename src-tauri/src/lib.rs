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
mod confirm_sheet;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod cursor_scrub;
mod desktop_capture;
mod editor;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod glide;
mod image_analysis;
mod monitor_topology;
mod osc;
mod permissions;
mod plugins;
mod recording;
mod recording_inputs;
mod recording_sources;
mod ruler;
mod screenshots;
mod settings;
mod shortcuts;
#[cfg(target_os = "macos")]
mod startup;
#[cfg(debug_assertions)]
mod storybook_native;
mod system_accent;
mod text_recognition;
mod tooltip_window;
#[cfg(desktop)]
mod tray;
mod updates;
mod windows;
#[cfg(target_os = "macos")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let builder = plugins::with_plugins(tauri::Builder::default());
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let builder = builder.manage(glide::settings::GlideSettingsState::default());
  let app = builder
    .manage(audio_preview::AudioPreviewState::default())
    .manage(camera_preview::CameraPreviewState::default())
    .manage(confirm_sheet::ConfirmSheetState::default())
    .manage(editor::EditorState::default())
    .manage(editor::recording_preview_player::RecordingPreviewPlayerState::default())
    .manage(editor::screenshot_preview::ScreenshotPreviewState::default())
    .manage(permissions::PermissionState::default())
    .manage(recording::RecordingState::default())
    .manage(ruler::RulerState::default())
    .manage(settings::GeneralSettingsState::default())
    .manage(shortcuts::ShortcutSettingsState::default())
    .manage(text_recognition::TextRecognitionState::default())
    .manage(text_recognition::qr_details::QrDetailsState::default())
    .manage(tooltip_window::TooltipState::default())
    .invoke_handler(tauri::generate_handler![
      audio_preview::start_audio_preview,
      audio_preview::stop_audio_preview,
      camera_preview::start_camera_preview,
      camera_preview::stop_camera_preview,
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      cursor_scrub::begin_cursor_scrub,
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      cursor_scrub::end_cursor_scrub,
      confirm_sheet::fit_confirm_sheet,
      confirm_sheet::get_confirm_sheet,
      confirm_sheet::resolve_confirm_sheet,
      editor::commands::browse_background_image,
      editor::commands::browse_export_directory,
      editor::commands::cancel_export_job,
      editor::commands::copy_editor_to_clipboard,
      editor::commands::focus_editor_window,
      editor::export_window::hide_export_options,
      editor::export_window::resize_export_options,
      editor::export_window::show_export_options,
      editor::commands::get_screenshot_content_bounds,
      editor::recording_preview_player::recenter::get_recording_content_bounds,
      editor::preview::estimate_recording_export,
      editor::preview::get_editor_snapshot,
      editor::recording_preview::get_recording_preview,
      editor::recording_preview::get_recording_keyboard_timeline,
      editor::recording_preview_player::commands::pause_recording_preview,
      editor::recording_preview_player::audio_visualizer::set_recording_audio_visualizer,
      editor::recording_preview_player::surface_commands::layout_recording_preview_surface,
      editor::recording_preview_player::surface_commands::set_recording_preview_zoom,
      editor::recording_preview_player::surface_commands::reset_recording_preview_view,
      editor::recording_preview_player::surface_commands::set_recording_preview_fit_basis,
      editor::recording_preview_player::editor_suspend::set_recording_preview_editor_suspended,
      editor::recording_preview_player::commands::playback::play_recording_preview,
      editor::recording_preview_player::commands::seek_recording_preview,
      editor::recording_preview_player::commands::select_recording_preview_audio,
      editor::recording_preview_player::commands::set_recording_preview_audio_volumes,
      editor::recording_preview_player::commands::set_recording_preview_cursor_effects,
      editor::recording_preview_player::keyboard_command::set_recording_preview_keyboard_effects,
      editor::recording_preview_player::keyboard_command::set_recording_preview_deleted_keyboard_shortcuts,
      editor::recording_preview_player::commands::set_recording_preview_composition,
      editor::recording_preview_player::commands::start_recording_preview_player,
      editor::recording_preview_player::commands::stop_recording_preview_player,
        editor::recording_preview_player::timeline_thumbnails::copy_recording_preview_frame_to_clipboard,
      editor::recording_preview_player::timeline_thumbnails::stream_recording_timeline_thumbnails,
      editor::save::save_export,
      editor::screenshot_preview::layout_screenshot_preview_surface,
      editor::screenshot_preview::refresh_screenshot_preview_sources,
      editor::screenshot_preview::set_screenshot_preview_editor_suspended,
      editor::screenshot_preview::set_screenshot_preview_zoom,
      editor::screenshot_preview::reset_screenshot_preview_view,
      editor::screenshot_preview::set_screenshot_preview_fit_basis,
      editor::screenshot_preview::start_screenshot_preview,
      editor::screenshot_preview::stop_screenshot_preview,
      editor::commands::set_export_directory,
      editor::commands::set_recording_timeline_edit,
      editor::commands::set_screenshot_background_radius,
      editor::commands::set_screenshot_radius,
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      glide::settings::get_glide_settings,
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      glide::settings::set_glide_settings,
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
      recording_sources::list_monitor_thumbnails,
      recording_sources::list_monitors,
      recording_sources::list_windows,
      recording_sources::selected_window_available,
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      settings::shortcut_defaults::get_shortcut_defaults,
      text_recognition::settings::get_ocr_settings,
      text_recognition::settings::set_ocr_settings,
      ruler::settings::get_ruler_settings,
      ruler::settings::set_ruler_settings,
      ruler::cancel_ruler,
      ruler::set_ruler_screenshot_mode,
      screenshots::scrolling::command::capture_scrolling_still,
      screenshots::capture_still,
      screenshots::thumbnail::render_background_thumbnail,
      text_recognition::cancel_text_recognition,
      text_recognition::close_qr_details,
      text_recognition::copy_recognition_content,
      text_recognition::get_qr_details,
      text_recognition::start_text_recognition,
      tooltip_window::fit_tooltip,
      tooltip_window::get_tooltip,
      tooltip_window::hide_tooltip,
      tooltip_window::show_tooltip,
      updates::update_checks_enabled,
      updates::hide_update_prompt,
      updates::show_update_prompt,
      settings::hide_settings,
      settings::preferences::browse_default_location,
      settings::preferences::get_general_settings,
      settings::preferences::set_general_settings,
      settings::wallpapers::list_system_wallpapers,
      settings::show_settings,
      shortcuts::get_shortcut_settings,
      shortcuts::resume_shortcut_action,
      shortcuts::begin_shortcut_capture,
      shortcuts::end_shortcut_capture,
      shortcuts::set_shortcut_binding,
      system_accent::get_system_accent,
      windows::color_panel::close_color_panel,
      windows::color_panel::show_color_panel,
      windows::source_selector::collapse_recording_source_selector,
      windows::source_selector::get_recording_source_selector_state,
      windows::region_gesture::begin_region_selector_gesture,
      windows::finish_recording_bar_drag,
      windows::region_gesture::finish_region_selector_gesture,
      windows::dock::finish_recording_dock_drag,
      windows::dock::resize_recording_dock,
      windows::hide_recording_ui,
      windows::recording_ui_visible,
      windows::toggle_recording_ui,
      windows::region::hide_region_selector,
      windows::options::hide_standalone_listbox,
      windows::options::fit_standalone_listbox,
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
      windows::options::placement::move_standalone_listbox,
      windows::panel_space::grow_editor_for_panel,
      windows::source_selector::expand_recording_source_selector,
    ])
    .setup(|app| {
      #[cfg(debug_assertions)]
      if let Some(preview_url) = std::env::var_os("SCREENWIDE_STORYBOOK_NATIVE_URL") {
        storybook_native::show(app.handle(), &preview_url.to_string_lossy())?;
        return Ok(());
      }

      #[cfg(target_os = "macos")]
      {
        editor::initialize_cursor_artwork();
      }
      #[cfg(desktop)]
      tray::initialize(app)?;
      settings::initialize(app.handle());
      // Load controls before native input monitoring starts.
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      glide::settings::initialize(app.handle());
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      glide::initialize(app.handle()).map_err(std::io::Error::other)?;
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
        windows::initialize_standalone_listbox(app.handle())?;
        windows::initialize_recording_dock(app.handle())?;
      }
      windows::initialize_predefined_windows(app.handle())?;
      windows::initialize_recording_bar_position(app.handle())?;
      windows::initialize_topology_management(app.handle());
      windows::manage_recording_bar_movement(app.handle());
      windows::manage_recording_dock_movement(app.handle());
      editor::initialize(app.handle());
      let has_pending_export = editor::has_pending_workspace(app.handle());
      shortcuts::initialize(app.handle());
      system_accent::initialize(app.handle());
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
      startup::hide_unrequested_windows(app.handle())?;

      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while running tauri application");

  #[cfg(target_os = "macos")]
  let mut app = app;
  #[cfg(target_os = "macos")]
  if !editor::has_pending_workspace(app.handle()) {
    app.set_dock_visibility(false);
  }
  app.run(|_, _| {});
}
