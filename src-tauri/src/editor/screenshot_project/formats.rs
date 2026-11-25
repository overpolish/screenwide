// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A screenshot's formats: the main canvas, then a shape for each platform
//! the screenshot is made for (a YouTube thumbnail, an Instagram square, a
//! TikTok cover), as a recording's are (`project_look::formats`). The editor
//! works them out and changes them; Rust keeps them with the project as the
//! editor sends them (`workspace.rs`), so these types only name them for it.

use std::collections::HashMap;

use screenwide_annotations::Annotation;
use screenwide_platform_overlays::PhoneView;
use serde_json::Value;
use ts_rs::TS;

use crate::editor::project_look::FormatScope;
use crate::editor::ScreenshotWorkspaceOutputSettings;

/// A screenshot's shapes, one of them in hand.
// Only named for the editor's binding: Rust keeps the formats as sent.
#[allow(dead_code)]
#[derive(TS)]
#[ts(export, rename_all = "camelCase")]
pub struct ScreenshotFormats {
  /// The id of the shape in hand, whose workspace is the project's own.
  pub active: String,
  pub list: Vec<ScreenshotShape>,
  /// The phone tall formats are previewed on while editing, whichever is in
  /// hand; none for the app's interface alone.
  #[ts(optional)]
  pub phone: Option<PhoneView>,
  /// Whether the phone, and the interface of the app a format is made for,
  /// are shown over the picture while editing: off until turned on.
  #[ts(optional)]
  pub preview: Option<bool>,
  /// The layers and annotations shown in only some formats; anything not
  /// named here shows in every one.
  #[ts(optional)]
  pub show_in: Option<ScreenshotShowIn>,
  /// Annotations placed their own way on the platforms of one proportion,
  /// as a recording's are: by annotation id, then by the proportion's key
  /// (`ratio-9x16`), the annotation as it is placed there.
  #[ts(optional)]
  pub layouts: Option<HashMap<String, HashMap<String, Annotation>>>,
}

/// The formats each layer and annotation shows in, by id: a format is a
/// platform's template, or the main canvas's shape id.
#[allow(dead_code)]
#[derive(TS)]
#[ts(export, rename_all = "camelCase")]
pub struct ScreenshotShowIn {
  /// By layer id, which lasts one session: the project keeps them by the
  /// layer's place in the stack (`with_layer_keys`), as it does its looks.
  #[ts(optional)]
  pub layers: Option<HashMap<String, FormatScope>>,
  #[ts(optional)]
  pub annotations: Option<HashMap<String, FormatScope>>,
}

/// `formats` with each layer it shows in only some formats renamed by
/// `rename`, and dropped where that names none.
pub(super) fn with_layer_keys(
  mut formats: Value,
  rename: impl Fn(&str) -> Option<String>,
) -> Value {
  let Some(layers) = formats
    .pointer_mut("/showIn/layers")
    .and_then(Value::as_object_mut)
  else {
    return formats;
  };
  *layers = std::mem::take(layers)
    .into_iter()
    .filter_map(|(key, scope)| Some((rename(&key)?, scope)))
    .collect();
  formats
}

/// `formats` with each look its shapes keep converted by `convert`, and
/// dropped where it cannot be.
pub(super) fn with_looks(mut formats: Value, convert: impl Fn(Value) -> Option<Value>) -> Value {
  for shape in formats
    .get_mut("list")
    .and_then(Value::as_array_mut)
    .into_iter()
    .flatten()
  {
    let Some(shape) = shape.as_object_mut() else {
      continue;
    };
    if let Some(look) = shape.remove("look").and_then(&convert) {
      shape.insert("look".to_owned(), look);
    }
  }
  formats
}

/// One shape a screenshot is made in, and the platforms published in it.
#[allow(dead_code)]
#[derive(TS)]
#[ts(export, rename_all = "camelCase")]
pub struct ScreenshotShape {
  pub id: String,
  /// The size templates of the platforms published in this shape, in the
  /// order they were added; none for the main canvas.
  #[ts(optional)]
  pub platforms: Option<Vec<String>>,
  /// The platform in hand while this shape is; the first when none is.
  #[ts(optional)]
  pub platform: Option<String>,
  /// Whether its layers are set in this shape their own way, moved,
  /// resized, turned, cropped or restyled while the shape was in hand,
  /// rather than laid out from the main canvas's.
  #[ts(optional)]
  pub placed: Option<bool>,
  /// Whether its background is set in this shape its own way, changed
  /// while the shape was in hand, rather than the main canvas's.
  #[ts(optional)]
  pub own_background: Option<bool>,
  /// Kept while another shape is in hand; the one in hand's is the
  /// project's workspace.
  #[ts(optional)]
  pub look: Option<ScreenshotWorkspaceOutputSettings>,
}
