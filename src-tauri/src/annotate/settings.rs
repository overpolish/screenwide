// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether live annotation is offered at all, what becomes of the annotations
//! when the overlay closes, and the dress a fresh stroke is drawn in.

use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::editor::annotations::{AnnotationHead, AnnotationKind};

/// The stroke presets the editor's arrow panel offers, in points, which is
/// what keeps a live annotation and an editor annotation the same weight. The
/// twin of `ANNOTATION_WIDTHS` in
/// `src/components/shared/annotation-style/widths.ts`.
const MIN_WIDTH: f64 = 8.0;
const MAX_WIDTH: f64 = 24.0;
/// The smallest and largest disc diameters the counter control offers, in
/// points. The twins of the ends of `ANNOTATION_COUNTER_SIZES` in the same
/// module, and of `COUNTER_SIZES` in
/// `src-tauri/src/editor/annotations/counter/model.rs`.
const MIN_COUNTER_SIZE: f64 = 28.0;
const MAX_COUNTER_SIZE: f64 = 80.0;
/// The palette's yellow, as in
/// `src/components/shared/annotation-style/palette.ts`.
const DEFAULT_COLOR: &str = "#ffcc00";

/// Where the user dragged the toolbar, in logical points from the top-left of
/// the display it was dropped on. Held per display so the toolbar comes back
/// where it was left on that screen, and falls back to top-centre on a
/// display it has never been dragged on.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolbarPosition {
  pub display_id: u32,
  pub x: f64,
  pub y: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AnnotateSettings {
  pub enabled: bool,
  /// Whether annotations survive the overlay closing. Off by default: the
  /// overlay is normally toggled off to be rid of what is on screen.
  pub keep_annotations_between_sessions: bool,
  /// What a fresh stroke is: the editor's own kind, so a tool added there
  /// is the same word here and in the stored settings.
  pub default_shape: AnnotationKind,
  /// `#rrggbb` or `#rrggbbaa`, as a document stores it.
  pub default_color: String,
  pub default_width: f64,
  /// The disc a fresh counter is drawn at, in points. Kept apart from the
  /// arrow's stroke because they are different measurements of different
  /// things: eight points of stroke would be a disc too small to hold a
  /// number.
  pub default_counter_size: f64,
  /// Where a fresh counter's tail points, in radians clockwise from east. Live
  /// annotations cannot be picked up again, so the aim is chosen before the
  /// counter is dropped rather than turned afterwards.
  pub default_counter_angle: f64,
  /// Which ends of a fresh arrow carry a head. The editor's own type, so the
  /// live overlay and the editor dress an arrow from the same value.
  pub default_head: AnnotationHead,
  /// Whether a fresh highlight is drawn as a marker stroke by hand rather
  /// than as a clean band.
  pub highlight_hand_drawn: bool,
  /// Whether a fresh highlight is laid by hand over the box its drag spans
  /// rather than fitted to the text under it.
  pub highlight_manual: bool,
  /// Whether a fresh highlight tints what it covers rather than recolouring
  /// the page under it.
  pub highlight_tint: bool,
  /// Whether a fresh shape is drawn as a pen stroke by hand rather than as a
  /// clean outline. Its own choice rather than the highlight's: a marker and
  /// an outline are drawn by hand for different reasons.
  pub shape_hand_drawn: bool,
  /// How round a fresh shape's corners are, as a percentage of its shorter
  /// side from 0 to 50: a square rounded all the way is a circle.
  pub shape_radius: f64,
  /// Whether a fresh spotlight also blurs what lies outside it.
  pub spotlight_blur: bool,
  /// How round a fresh spotlight's corners are, as a percentage of its
  /// shorter side from 0 to 50.
  pub spotlight_radius: f64,
  /// How far a fresh spotlight's edge fades, as a percentage of its shorter
  /// side from 0 to 50.
  pub spotlight_softness: f64,
  pub toolbar_position: Option<ToolbarPosition>,
}

impl Default for AnnotateSettings {
  fn default() -> Self {
    Self {
      enabled: true,
      keep_annotations_between_sessions: false,
      default_shape: AnnotationKind::Arrow,
      default_color: DEFAULT_COLOR.to_owned(),
      default_width: MIN_WIDTH,
      default_counter_size: MIN_COUNTER_SIZE,
      default_counter_angle: 0.0,
      default_head: AnnotationHead::default(),
      highlight_hand_drawn: false,
      highlight_manual: false,
      highlight_tint: false,
      shape_hand_drawn: false,
      shape_radius: 0.0,
      spotlight_blur: false,
      spotlight_radius: crate::editor::annotations::spotlight::model::NEW_SPOTLIGHT_RADIUS,
      spotlight_softness: crate::editor::annotations::spotlight::model::NEW_SPOTLIGHT_SOFTNESS,
      toolbar_position: None,
    }
  }
}

/// Settings the overlay can actually draw with. A colour the compositor cannot
/// read would cost the annotation rather than the setting, so it is refused
/// here.
fn validated(mut settings: AnnotateSettings) -> Result<AnnotateSettings, String> {
  let color = settings.default_color.to_ascii_lowercase();
  let digits = color
    .strip_prefix('#')
    .filter(|digits| matches!(digits.len(), 6 | 8))
    .filter(|digits| digits.chars().all(|digit| digit.is_ascii_hexdigit()))
    .ok_or_else(|| "That is not an annotation colour".to_owned())?;
  // The overlay's annotations cannot be picked up again, so a text box could
  // never be typed into there, nor a magnifier's zoom area moved onto what
  // it is meant to enlarge. A redaction drawn on the live desktop hides
  // nothing from the screen it covers, so the overlay offers none.
  if matches!(
    settings.default_shape,
    AnnotationKind::Text | AnnotationKind::Redact | AnnotationKind::Magnify
  ) {
    return Err(
      "The live overlay draws arrows, counters, highlights, shapes, spotlights and strokes"
        .to_owned(),
    );
  }
  if !settings.default_width.is_finite()
    || !(MIN_WIDTH..=MAX_WIDTH).contains(&settings.default_width)
  {
    return Err("That is not an annotation stroke width".to_owned());
  }
  if !settings.default_counter_size.is_finite()
    || !(MIN_COUNTER_SIZE..=MAX_COUNTER_SIZE).contains(&settings.default_counter_size)
  {
    return Err("That is not a counter size".to_owned());
  }
  if !settings.default_counter_angle.is_finite() {
    return Err("That is not a counter angle".to_owned());
  }
  let share = |value: f64| value.is_finite() && (0.0..=50.0).contains(&value);
  if !share(settings.shape_radius) {
    return Err("That is not a shape radius".to_owned());
  }
  if !share(settings.spotlight_radius) || !share(settings.spotlight_softness) {
    return Err("That is not a spotlight radius or softness".to_owned());
  }
  if settings
    .toolbar_position
    .is_some_and(|position| !position.x.is_finite() || !position.y.is_finite())
  {
    return Err("That is not a toolbar position".to_owned());
  }
  settings.default_color = format!("#{digits}");
  Ok(settings)
}

static SETTINGS: LazyLock<RwLock<AnnotateSettings>> =
  LazyLock::new(|| RwLock::new(AnnotateSettings::default()));

pub fn current() -> AnnotateSettings {
  SETTINGS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .clone()
}

pub fn enabled() -> bool {
  SETTINGS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .enabled
}

pub(super) fn keep_annotations_between_sessions() -> bool {
  SETTINGS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .keep_annotations_between_sessions
}

fn path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|path| path.join("annotate-settings.json"))
    .map_err(|error| error.to_string())
}

pub fn initialize(app: &AppHandle) {
  let settings = path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    .and_then(|settings| validated(settings).ok())
    .unwrap_or_default();
  *SETTINGS.write().unwrap_or_else(|error| error.into_inner()) = settings;
}

#[tauri::command]
pub fn get_annotate_settings() -> AnnotateSettings {
  current()
}

#[tauri::command]
pub fn set_annotate_settings(
  app: AppHandle,
  settings: AnnotateSettings,
) -> Result<AnnotateSettings, String> {
  let settings = validated(settings)?;
  let path = path(&app)?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let bytes = serde_json::to_vec_pretty(&settings).map_err(|error| error.to_string())?;
  let previous = std::mem::replace(
    &mut *SETTINGS.write().unwrap_or_else(|error| error.into_inner()),
    settings.clone(),
  );
  // The shortcut is registered against the feature being on, so it has to
  // follow the same swap the overlay does - and be put back with it.
  let result = crate::shortcuts::sync_annotate_enabled(&app)
    .and_then(|()| std::fs::write(path, bytes).map_err(|error| error.to_string()));
  if let Err(error) = result {
    *SETTINGS.write().unwrap_or_else(|error| error.into_inner()) = previous;
    let _ = crate::shortcuts::sync_annotate_enabled(&app);
    return Err(error);
  }
  if !settings.enabled {
    super::disable(&app);
  }
  if settings.default_shape != previous.default_shape {
    super::follow_tool(&app);
  }
  crate::tray::refresh(&app);
  let _ = app.emit("annotate-settings://changed", &settings);
  Ok(settings)
}

/// Remembers where the toolbar was dropped. It goes through the ordinary
/// write so the change event fires: the toolbar and the Settings page are one
/// state, and a drag is an edit like any other.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) fn store_toolbar_position(
  app: &AppHandle,
  position: ToolbarPosition,
) -> Result<(), String> {
  let settings = AnnotateSettings {
    toolbar_position: Some(position),
    ..current()
  };
  set_annotate_settings(app.clone(), settings).map(|_| ())
}

/// The keyboard picking a tool up. Goes through the ordinary write for the
/// same reason a drag does.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(super) fn store_default_shape(app: &AppHandle, shape: AnnotationKind) -> Result<(), String> {
  let settings = AnnotateSettings {
    default_shape: shape,
    ..current()
  };
  set_annotate_settings(app.clone(), settings).map(|_| ())
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
