// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// One independently editable image in a screenshot workspace.
/// Pixels remain owned by Rust and are uploaded to the native renderer once;
/// the webview only ever needs this identity and the scene metadata added in
/// the next slice. `annotations` are the live annotations the shot covered,
/// in its pixels: the item's first annotations, seeded into its layer once.
#[derive(Clone)]
pub struct ScreenshotItem {
  pub annotations: Vec<Annotation>,
  pub id: u64,
  pub image: CapturedImage,
  /// The display scale the image was captured at: how many of its pixels one
  /// logical point spans. One for a picture with no display behind it, such
  /// as one pasted from the clipboard.
  pub scale_factor: f64,
}

impl ScreenshotItem {
  /// The image's width in logical points, which its redactions are sized in.
  pub(crate) fn capture_width_points(&self) -> f64 {
    let scale = if self.scale_factor.is_finite() && self.scale_factor > 0.0 {
      self.scale_factor
    } else {
      1.0
    };
    f64::from(self.image.width) / scale
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotWorkspaceItemOutput {
  pub id: u64,
  pub output: ScreenshotOutputSettings,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotWorkspaceOutputSettings {
  #[serde(flatten)]
  pub canvas: ScreenshotOutputSettings,
  #[serde(default)]
  pub items: Vec<ScreenshotWorkspaceItemOutput>,
}

impl ScreenshotWorkspaceOutputSettings {
  pub(crate) fn output_for(&self, item: &ScreenshotItem) -> ScreenshotOutputSettings {
    self.output_for_id(item.id, item.capture_width_points())
  }

  /// The layer `id`'s settings, for an image `capture_width_points` logical
  /// points wide.
  pub(crate) fn output_for_id(
    &self,
    id: u64,
    capture_width_points: f64,
  ) -> ScreenshotOutputSettings {
    let mut output = self
      .items
      .iter()
      .find(|candidate| candidate.id == id)
      .map_or_else(
        || {
          // A layer the workspace holds no entry for takes the canvas as its
          // template, and a template carries no annotations: they were drawn on
          // a layer, never on the canvas behind it.
          let mut canvas = self.canvas.clone();
          canvas.annotations.clear();
          canvas
        },
        |candidate| candidate.output.clone(),
      );
    output.background_color = self.canvas.background_color.clone();
    output.background_image_path = self.canvas.background_image_path.clone();
    output.background_type = self.canvas.background_type.clone();
    output.background_radius_percent = self.canvas.background_radius_percent;
    output.height = self.canvas.height;
    output.mesh_colors = self.canvas.mesh_colors.clone();
    output.mesh_generator = self.canvas.mesh_generator.clone();
    output.mesh_locked_colors = self.canvas.mesh_locked_colors.clone();
    output.mesh_points = self.canvas.mesh_points.clone();
    output.mesh_seed = self.canvas.mesh_seed;
    output.mesh_warp_percent = self.canvas.mesh_warp_percent;
    output.capture_width_points = capture_width_points;
    output.width = self.canvas.width;
    output
  }
}
