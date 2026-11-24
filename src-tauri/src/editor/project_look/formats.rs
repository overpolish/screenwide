// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording's formats, grouped by shape: each shape (16:9, 9:16, 4:5,
//! 1:1) has its own canvas, camera and layouts, and holds the platforms
//! published in it (TikTok, Shorts and Reels in 9:16), which share them.
//! A format is one file the recording is published as: a platform, or a
//! shape made for none. Everything else is shared: the cuts, what each
//! caption says and when each scene, annotation and sound plays. The editor
//! works out each format's picture from the one edit and these
//! differences; Rust keeps them with the project.

use std::collections::BTreeMap;

use screenwide_annotations::Annotation;
use screenwide_export::recording_model::RecordingOutputSettings;
use screenwide_scenes::model::SceneFormatLayout;
use screenwide_scenes::CameraOverlaySettings;
use screenwide_text::{TextLayout, TextStyle};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A recording's shapes, one of them in hand.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FrameFormats {
  /// The id of the shape in hand, whose look is the project look's own.
  pub active: String,
  pub list: Vec<FrameFormat>,
  /// What scenes, annotations, sounds and captions do differently in some
  /// shapes and formats.
  #[serde(default, skip_serializing_if = "FormatEdits::is_empty")]
  #[ts(optional, as = "Option<FormatEdits>")]
  pub edits: FormatEdits,
  /// The phone tall formats are previewed on while editing, whichever is in
  /// hand; none for the app's interface alone.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub phone: Option<screenwide_platform_overlays::PhoneView>,
  /// Whether the phone, and the interface of the app a format is made for,
  /// are shown over the picture while editing: off until turned on.
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  #[ts(as = "Option<bool>", optional)]
  pub preview: bool,
}

/// One shape a recording is made in, and the platforms published in it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FrameFormat {
  pub id: String,
  /// The size templates of the platforms published in this shape, in the
  /// order they were added; none for a size typed in, which is a format of
  /// its own.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  #[ts(optional, as = "Option<Vec<String>>")]
  pub platforms: Vec<String>,
  /// The platform in hand while this shape is: the one previewed and
  /// played, and named by the Frame island; the first when none is.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub platform: Option<String>,
  /// Whether the screen is set in this shape its own way, moved, resized,
  /// turned, cropped or restyled while the shape was in hand, rather than
  /// following the main canvas's.
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  #[ts(optional, as = "Option<bool>")]
  pub placed: bool,
  /// Kept while another shape is in hand; the one in hand's lives in the
  /// project look.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub look: Option<FrameFormatLook>,
}

/// What a format not in hand keeps of its own canvas.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FrameFormatLook {
  pub camera_overlay: CameraOverlaySettings,
  pub recording_output: RecordingOutputSettings,
}

/// Where something shows: in every format, or in the formats named, each
/// by its platform's template, or by its shape's id for a shape made for
/// no platform.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FormatScope {
  All,
  Formats(Vec<String>),
}

/// What an edit does differently in some shapes and formats, by the id of
/// the scene, annotation, song or sound effect, or caption track it changes.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct FormatEdits {
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[ts(optional, as = "Option<BTreeMap<String, SceneFormats>>")]
  pub scenes: BTreeMap<String, SceneFormats>,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[ts(optional, as = "Option<BTreeMap<String, AnnotationFormats>>")]
  pub annotations: BTreeMap<String, AnnotationFormats>,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[ts(optional, as = "Option<BTreeMap<String, AudioFormats>>")]
  pub audio: BTreeMap<String, AudioFormats>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub captions: Option<CaptionFormats>,
}

impl FormatEdits {
  pub fn is_empty(&self) -> bool {
    self.scenes.is_empty()
      && self.annotations.is_empty()
      && self.audio.is_empty()
      && self.captions.is_none()
  }
}

/// Where a scene shows, and its layout in the shapes it is laid out
/// differently in, by shape id.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct SceneFormats {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub show_in: Option<FormatScope>,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[ts(optional, as = "Option<BTreeMap<String, SceneFormatLayout>>")]
  pub layouts: BTreeMap<String, SceneFormatLayout>,
}

/// Where an annotation shows, and the annotation as it is placed in the
/// shapes it is placed differently in, by shape id.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct AnnotationFormats {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub show_in: Option<FormatScope>,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[ts(optional, as = "Option<BTreeMap<String, Annotation>>")]
  pub layouts: BTreeMap<String, Annotation>,
}

/// Where a song or sound effect plays: a format can have music of its own,
/// or none where the platform's own songs are used.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct AudioFormats {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub show_in: Option<FormatScope>,
}

/// Where the captions show, and how they look and sit in the shapes they
/// look different in, by shape id.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionFormats {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub show_in: Option<FormatScope>,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  #[ts(optional, as = "Option<BTreeMap<String, CaptionLayout>>")]
  pub layouts: BTreeMap<String, CaptionLayout>,
}

/// How the captions look and where they sit, in one shape.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionLayout {
  pub style: TextStyle,
  #[serde(default, skip_serializing_if = "TextLayout::is_empty")]
  #[ts(optional, as = "Option<TextLayout>")]
  pub layout: TextLayout,
}
