// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the scene holds a layer as, and the pictures it is drawn from.

use screenwide_imaging::CapturedImage;
use std::sync::Arc;

use super::picture::{LayerPicture, ScenePicture};
use screenwide_annotations::pin::LumaFrame;
use screenwide_compositor::annotation_gpu::SourceAnnotations;
use screenwide_compositor::compositor::still_overlay::StillOverlay;
use screenwide_compositor::compositor::BakeGeometry;
use screenwide_compositor::compositor::{
  CameraComposition, CanvasGeometry, Compositor, SourceTexture,
};
use screenwide_cursor_effects::GpuCursor;
use screenwide_export::camera_effects::preview::PreviewEffects;
use screenwide_keyboard_effects::KeyboardOverlay;
use screenwide_output::ScreenshotOutputSettings;

/// One layer as a present hands it over.
pub struct StagedLayer<'a> {
  pub pane_index: u32,
  /// Which layer a gesture or the crop magnifier addresses.
  pub layer_id: u32,
  /// The cache key of the picture: a new token is a new picture.
  pub source_token: u64,
  pub source: LayerPicture<'a>,
  /// A screenshot's own pixels, which its redactions read their fills from.
  /// A recording frame has none: its fills are held from its clips' frames.
  pub redaction_picture: Option<&'a CapturedImage>,
  pub settings: &'a ScreenshotOutputSettings,
  pub seconds: f64,
  pub cursor: Option<GpuCursor>,
  pub keyboard: Option<KeyboardOverlay>,
  /// The captions drawn over the canvas, and the moment of the edited video
  /// they are drawn at.
  pub captions: Option<(
    Arc<screenwide_text::captions::CaptionTrack>,
    screenwide_text::captions::CaptionMoment,
  )>,
  /// The camera frame composed into the layer, and where its overlay puts
  /// it; without an overlay the frame is held but not drawn.
  pub camera: Option<LayerPicture<'a>>,
  pub overlay: Option<&'a StillOverlay>,
  /// The camera's own settings, carrying the annotations drawn into its
  /// frame, and the camera's source size, which they are placed in.
  pub camera_settings: Option<(&'a ScreenshotOutputSettings, (u32, u32))>,
  pub foreground_only: bool,
  /// Whether `source` is the camera, which the camera's effects draw on.
  pub source_is_camera: bool,
  pub hover: Option<(usize, f32)>,
}

/// A layer's camera bubble: its frame, the part of it shown and where that
/// sits on the canvas, and the annotations composed into the frame first.
#[derive(Clone)]
pub(super) struct LayerCamera {
  pub(super) picture: Option<Arc<ScenePicture>>,
  /// Its output size is the canvas's, filled in when the layer is drawn.
  pub(super) geometry: BakeGeometry,
  pub(super) drop_shadow: bool,
  pub(super) on_top: bool,
  pub(super) composition: Option<Arc<CameraComposition>>,
  /// The halo on one of the composed annotations, its width in canvas
  /// pixels.
  pub(super) hover: Option<(usize, f32)>,
  /// The matte of the frame, where the camera's effects take its background
  /// away.
  pub(super) matte: Option<Arc<LumaFrame>>,
}

impl LayerCamera {
  pub(super) fn new(overlay: &StillOverlay, picture: Option<Arc<ScenePicture>>) -> Self {
    Self {
      picture,
      geometry: BakeGeometry {
        crop_x: overlay.camera_crop_x,
        crop_y: overlay.camera_crop_y,
        crop_width: overlay.camera_crop_width,
        crop_height: overlay.camera_crop_height,
        frame_x: overlay.camera_frame_x,
        frame_y: overlay.camera_frame_y,
        frame_width: overlay.camera_frame_width,
        frame_height: overlay.camera_frame_height,
        output_width: 0,
        output_height: 0,
        radius: overlay.camera_radius,
      },
      drop_shadow: overlay.camera_drop_shadow != 0,
      on_top: overlay.camera_on_top != 0,
      composition: None,
      hover: None,
      matte: None,
    }
  }

  /// The frame the canvas draws: the camera's own, the frame `seconds` into
  /// the recording, drawn with its effects, with the composed annotations
  /// drawn into it after where it has any. `slot` is the layer's.
  pub(super) fn drawn(
    &self,
    compositor: &Compositor,
    picture: &ScenePicture,
    (effects, slot, seconds): (&PreviewEffects, usize, f64),
  ) -> Result<SourceTexture, String> {
    let effected = effects.drawn(
      compositor,
      (&picture.texture, seconds),
      self.matte.as_deref(),
      slot * 2 + 1,
    )?;
    let frame = effected.as_ref().unwrap_or(&picture.texture);
    let Some(composition) = &self.composition else {
      return Ok(frame.clone());
    };
    // The halo's width arrives in canvas pixels; the frame is drawn into the
    // canvas at the crop's size over the frame's.
    let hover = self.hover.map(|(index, width)| {
      let frame_pixels = self.geometry.crop_width as f32 / self.geometry.frame_width.max(1) as f32;
      (index, width * frame_pixels)
    });
    // The effects' picture is drawn again each time, and a refilled one
    // changes under its texture, so neither is retained.
    compositor.composed_camera(
      frame,
      composition,
      hover,
      slot,
      effected.is_none() && picture.retained,
    )
  }
}

#[derive(Clone)]
pub(super) struct SceneLayer {
  pub(super) pane_index: u32,
  pub(super) layer_id: u32,
  pub(super) source_token: u64,
  pub(super) source: Arc<ScenePicture>,
  pub(super) settings: Arc<ScreenshotOutputSettings>,
  pub(super) annotations: Arc<SourceAnnotations>,
  /// Where the picture sits and how the canvas is rounded, which a native
  /// gesture moves ahead of the settings.
  pub(super) geometry: CanvasGeometry,
  pub(super) seconds: f64,
  pub(super) cursor: Option<GpuCursor>,
  pub(super) keyboard: Option<KeyboardOverlay>,
  pub(super) captions: Option<(
    Arc<screenwide_text::captions::CaptionTrack>,
    screenwide_text::captions::CaptionMoment,
  )>,
  pub(super) camera: Option<LayerCamera>,
  pub(super) foreground_only: bool,
  pub(super) hover: Option<(usize, f32)>,
  /// Whether the layer's own picture is the camera, which the camera's
  /// effects are drawn on, and the matte of its frame where they read one.
  pub(super) source_is_camera: bool,
  pub(super) source_matte: Option<Arc<LumaFrame>>,
}

impl SceneLayer {
  /// Moves everything attached to the clip by `(dx, dy)` canvas pixels, as a
  /// Frame gesture moves the canvas origin: the crop and image placement, the
  /// visible source region the shader gates clip coverage on, and the cursor.
  pub(super) fn shift_content(&mut self, dx: f64, dy: f64) {
    let placement = &mut self.geometry.placement;
    placement.crop_x = (f64::from(placement.crop_x) + dx).round() as i32;
    placement.crop_y = (f64::from(placement.crop_y) + dy).round() as i32;
    placement.image_x += dx;
    placement.image_y += dy;
    placement.source_crop_x += dx.round() as i32;
    placement.source_crop_y += dy.round() as i32;
    if let Some(cursor) = self.cursor.as_mut() {
      cursor.x += dx as f32;
      cursor.y += dy as f32;
    }
  }
}
