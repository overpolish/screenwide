// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod background;
#[cfg(not(target_os = "macos"))]
mod compose;
mod crop_preview;
mod settings;

pub use background::{BackgroundTurn, CanvasBackground};
#[cfg(not(target_os = "macos"))]
pub use compose::compose_screenshot;
pub use crop_preview::CropPreviewRect;
pub use settings::output_dimensions;

use serde::{Deserialize, Serialize};

use super::NormalizedSourceRect;
use super::{mesh::MeshGradientPoint, mesh_generator::default_generator};
use screenwide_annotations::Annotation;

const MAX_OUTPUT_PIXELS: u64 = 120_000_000;

/// One layer's canvas and its placement in it.
///
/// Placement is in output pixels rather than in shares of the canvas, so the
/// canvas can be resized without moving or rescaling anything placed in it.
/// The crop is the visible rectangle; the image behind it is given by its top
/// left corner and its width, its height following the source's aspect.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotOutputSettings {
  /// Annotations drawn over this layer, in the source's own pixel space.
  #[serde(default)]
  pub annotations: Vec<Annotation>,
  pub background_color: String,
  /// A picture of your own behind the layers. Absent for every background
  /// that is painted rather than loaded, and for settings written before
  /// pictures could be chosen.
  #[serde(default)]
  pub background_image_path: Option<String>,
  pub background_type: String,
  pub background_radius_percent: f64,
  /// The whole source's width in the logical points it was captured at: its
  /// pixel width over the display's scale. Redactions size their cells in
  /// these points, so they hide as much on a 2x capture as on a 1x one.
  /// Stamped by Rust from the capture it belongs to; never sent by the
  /// webview or saved with a preference, where it would follow the look onto
  /// another capture. Zero where unknown, which reads as one point per
  /// source pixel.
  #[serde(skip)]
  pub capture_width_points: f64,
  /// Source pixels per logical point: annotation sizes are points, drawn this
  /// many output pixels to the point. Stamped with `capture_width_points`.
  #[serde(skip)]
  pub capture_scale: f64,
  /// How round the captured source's own corners are, in its pixels, which
  /// the picture is rounded to underneath the look's corners. Stamped with
  /// `capture_width_points`; zero for a square source.
  #[serde(skip)]
  pub capture_corner_radius: f64,
  /// The visible rectangle, in output pixels. Zero width or height marks a
  /// settings blob written before placement moved into pixels.
  #[serde(default)]
  pub crop_height: f64,
  /// The committed crop, in output pixels, drawn over the uncropped ghost
  /// while the crop tool is open; only in that preview, never saved.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub crop_preview: Option<CropPreviewRect>,
  #[serde(default)]
  pub crop_width: f64,
  #[serde(default)]
  pub crop_x: f64,
  #[serde(default)]
  pub crop_y: f64,
  pub drop_shadow: bool,
  /// How strong the shadow is drawn, zero to one, while the preview fades it
  /// in or out; only in that preview, never saved. Absent, a shadow is drawn
  /// whole.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub drop_shadow_strength: Option<f64>,
  pub height: u32,
  /// The whole image's top left corner and width, in output pixels.
  #[serde(default)]
  pub image_width: f64,
  #[serde(default)]
  pub image_x: f64,
  #[serde(default)]
  pub image_y: f64,
  /// The padding as a share of the picture's shorter side, which the crop's
  /// frame is grown by; absent where only the frame carried it (older saves).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub inset: Option<f64>,
  #[serde(default, rename = "mode", skip_serializing)]
  pub legacy_mode: Option<String>,
  /// How far a scene moved this layer while the shutter was open, blurred
  /// over; set per drawn frame, never sent or saved.
  #[serde(skip)]
  pub scene_motion: Option<screenwide_scenes::SceneMotion>,
  /// How opaque a scene draws the screen and the camera as it hides or shows
  /// them, and how far it draws the camera in front, one to zero. Set per
  /// drawn frame by the scene arrangement, never sent or saved; unset, both
  /// are drawn whole in the composition's own order.
  #[serde(skip)]
  pub scene_opacity: Option<[f32; 2]>,
  #[serde(skip)]
  pub scene_camera_front: Option<f32>,
  /// Where a padding scene shows the picture in the crop, as left, top, width
  /// and height in output pixels; set per drawn frame, never sent or saved.
  #[serde(skip)]
  pub scene_window: Option<[f64; 4]>,
  /// A scene's background blooming in over the one before it; set per drawn
  /// frame by the scene arrangement, never sent or saved.
  #[serde(skip)]
  pub background_turn: Option<BackgroundTurn>,
  /// How strong the camera drawn into this picture casts its shadow, zero to
  /// one, which is the camera's own settings' to say: stamped from them with
  /// the captures (`RecordingOutputSettings::stamp_captures`), never sent or
  /// saved. Unset, a camera with a shadow casts it whole.
  #[serde(skip)]
  pub camera_shadow_strength: Option<f32>,
  pub mesh_colors: Vec<String>,
  /// Which picture the mesh background paints. Settings written before the
  /// ported generators existed carry none, and are the app's own blob mesh.
  #[serde(default = "default_generator")]
  pub mesh_generator: String,
  #[serde(default)]
  pub mesh_locked_colors: Vec<bool>,
  pub mesh_points: Vec<MeshGradientPoint>,
  pub mesh_seed: u32,
  pub mesh_warp_percent: f64,
  pub radius_percent: f64,
  #[serde(default)]
  pub recenter_inset_color: Option<String>,
  /// How this layer is turned in depth about its crop's middle, with its
  /// annotations and cursor; absent where it stands upright. A scene replaces
  /// it in every frame the scene arranges.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub rotation: Option<screenwide_scenes::SceneRotation>,
  pub source_crop: NormalizedSourceRect,
  pub width: u32,
}
