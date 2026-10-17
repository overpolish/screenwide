// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{path::PathBuf, sync::RwLock};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::settings::background_preset::BackgroundPreset;
use screenwide_accent::AccentPreference;
use screenwide_i18n::t;
use screenwide_scenes::SceneTemplate;
use screenwide_text::captions::CaptionSettings;
use screenwide_text::{TextLayout, TextStyle, TextTemplateId};

const SETTINGS_FILE: &str = "settings.json";
const SETTINGS_CHANGED_EVENT: &str = "settings://changed";

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct GeneralSettings {
  pub accent: AccentPreference,
  /// Colours the annotation tools were given that none of their presets
  /// offers, newest last. Kept beside the saved backgrounds because they are
  /// the same kind of thing: a choice made once that should be findable the
  /// next time, rather than anything about the capture being edited.
  pub annotation_colors: Vec<String>,
  /// Backgrounds saved from the editor's background picker, in the order
  /// they were saved. The built-in ones are not kept here: they ship with
  /// the app and would only go stale on disk.
  pub background_presets: Vec<BackgroundPreset>,
  /// Where new projects are made. None is the default, a Screenwide folder in
  /// the platform's Movies or Videos folder.
  pub project_directory: Option<PathBuf>,
  pub recording_directory: Option<PathBuf>,
  pub screenshot_directory: Option<PathBuf>,
  pub open_location_after_export: bool,
  pub record_screenwide_windows: bool,
  pub show_recording_confidence_checks: bool,
  /// Whether the recording pill shows how much of the recording is on disk.
  pub show_recording_size: bool,
  /// Whether a new recording opens with zoom scenes made from its clicks and
  /// typing.
  pub auto_zoom: bool,
  /// Custom scene layouts saved from the Scene panel, oldest first, offered
  /// in every recording.
  pub scene_templates: Vec<SceneTemplate>,
  /// Caption templates as they were last saved as defaults, in place of the
  /// style each ships with; one never saved is not kept.
  pub caption_styles: Vec<TextStyle>,
  /// The template a recording's captions start from.
  pub caption_template: TextTemplateId,
  /// What a recording's captions start out saying, as last set as default.
  pub caption_settings: CaptionSettings,
  /// The layout options set by hand and kept for new captions, over any
  /// template.
  pub caption_layout: TextLayout,
  pub launch_at_login: bool,
  pub show_recording_bar_on_launch: bool,
  pub recording_countdown_seconds: u8,
  /// How long Delayed Screenshot waits before the shutter fires.
  pub screenshot_delay_seconds: u8,
}

impl Default for GeneralSettings {
  fn default() -> Self {
    Self {
      accent: AccentPreference::System,
      annotation_colors: Vec::new(),
      background_presets: Vec::new(),
      project_directory: None,
      recording_directory: None,
      screenshot_directory: None,
      open_location_after_export: true,
      record_screenwide_windows: false,
      show_recording_confidence_checks: true,
      show_recording_size: false,
      auto_zoom: true,
      scene_templates: Vec::new(),
      caption_styles: Vec::new(),
      caption_template: TextTemplateId::default(),
      caption_settings: CaptionSettings::default(),
      caption_layout: TextLayout::default(),
      launch_at_login: false,
      show_recording_bar_on_launch: true,
      recording_countdown_seconds: 0,
      screenshot_delay_seconds: 3,
    }
  }
}

#[derive(Default)]
pub struct GeneralSettingsState(RwLock<GeneralSettings>);

fn path(app: &AppHandle) -> Result<PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|directory| directory.join(SETTINGS_FILE))
    .map_err(|error| error.to_string())
}

fn read(app: &AppHandle) -> GeneralSettings {
  let mut settings: GeneralSettings = path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|contents| serde_json::from_slice(&contents).ok())
    .unwrap_or_default();
  // A template edited by hand into something no scene can take is dropped
  // alone, rather than refusing every setting beside it.
  settings.scene_templates.retain(SceneTemplate::is_valid);
  settings.caption_styles.retain(TextStyle::is_valid);
  settings
}

fn write(app: &AppHandle, settings: &GeneralSettings) -> Result<(), String> {
  let path = path(app)?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let contents = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
  std::fs::write(path, contents).map_err(|error| error.to_string())
}

fn validate(settings: &GeneralSettings) -> Result<(), String> {
  if !matches!(settings.recording_countdown_seconds, 0 | 3 | 5) {
    return Err("The countdown must be off, 3 seconds or 5 seconds".to_owned());
  }
  if !matches!(settings.screenshot_delay_seconds, 3 | 5 | 10) {
    return Err("The screenshot delay must be 3, 5 or 10 seconds".to_owned());
  }
  for directory in [
    settings.project_directory.as_ref(),
    settings.recording_directory.as_ref(),
    settings.screenshot_directory.as_ref(),
  ]
  .into_iter()
  .flatten()
  {
    if !directory.is_dir() {
      return Err(format!("{} is no longer available", directory.display()));
    }
  }
  if !settings.scene_templates.iter().all(SceneTemplate::is_valid) {
    return Err("A scene template is not valid".to_owned());
  }
  if !settings.caption_styles.iter().all(TextStyle::is_valid) {
    return Err("A caption template is not valid".to_owned());
  }
  Ok(())
}

pub fn initialize(app: &AppHandle) {
  let mut settings = read(app);
  settings.launch_at_login = app.autolaunch().is_enabled().unwrap_or(false);
  screenwide_accent::set_preference(settings.accent);
  *app
    .state::<GeneralSettingsState>()
    .0
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings;
}

pub fn current(app: &AppHandle) -> GeneralSettings {
  app
    .state::<GeneralSettingsState>()
    .0
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .clone()
}

#[tauri::command]
pub fn get_general_settings(app: AppHandle) -> GeneralSettings {
  current(&app)
}

#[tauri::command]
pub fn set_general_settings(
  app: AppHandle,
  mut settings: GeneralSettings,
) -> Result<GeneralSettings, String> {
  validate(&settings)?;
  let current_settings = current(&app);
  if settings.launch_at_login != current_settings.launch_at_login {
    let autolaunch = app.autolaunch();
    if settings.launch_at_login {
      autolaunch.enable().map_err(|error| error.to_string())?;
    } else {
      autolaunch.disable().map_err(|error| error.to_string())?;
    }
    settings.launch_at_login = autolaunch.is_enabled().unwrap_or(settings.launch_at_login);
  }
  #[cfg(target_os = "windows")]
  let capture_affinity_changed =
    settings.record_screenwide_windows != current_settings.record_screenwide_windows;
  #[cfg(target_os = "windows")]
  if capture_affinity_changed {
    crate::app_windows::sync_capture_affinity(&app, settings.record_screenwide_windows)
      .map_err(|error| error.to_string())?;
  }
  #[cfg(target_os = "windows")]
  if let Err(error) = write(&app, &settings) {
    if capture_affinity_changed {
      let _ =
        crate::app_windows::sync_capture_affinity(&app, current_settings.record_screenwide_windows);
    }
    return Err(error);
  }
  #[cfg(not(target_os = "windows"))]
  write(&app, &settings)?;
  *app
    .state::<GeneralSettingsState>()
    .0
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings.clone();
  if settings.accent != current_settings.accent {
    screenwide_accent::preference_changed(&app, settings.accent);
  }
  let _ = app.emit(SETTINGS_CHANGED_EVENT, &settings);
  Ok(settings)
}

#[tauri::command]
pub async fn browse_default_location(
  app: AppHandle,
  kind: String,
) -> Result<Option<PathBuf>, String> {
  let settings = current(&app);
  let start = match kind.as_str() {
    "project" => settings
      .project_directory
      .or_else(|| crate::projects_folder::projects_directory(&app).ok()),
    "recording" => settings.recording_directory,
    "screenshot" => settings.screenshot_directory,
    _ => return Err("Unknown default location".to_owned()),
  };
  let parent = app.get_webview_window(crate::app_windows::WindowLabel::Settings.as_str());
  tauri::async_runtime::spawn_blocking(move || {
    use tauri_plugin_dialog::DialogExt;
    let mut dialog = app.dialog().file().set_title(t!("dialog-choose-folder"));
    if let Some(start) = start {
      dialog = dialog.set_directory(start);
    }
    if let Some(parent) = parent {
      dialog = dialog.set_parent(&parent);
    }
    dialog
      .blocking_pick_folder()
      .and_then(|path| path.into_path().ok())
  })
  .await
  .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
