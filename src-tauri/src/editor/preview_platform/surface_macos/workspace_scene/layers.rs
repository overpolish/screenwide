// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the scene holds a layer as, and the pictures it is drawn from.

use std::sync::Arc;

use cidre::{arc, cv};

use crate::editor::cursor_effects::GpuCursor;
use crate::editor::keyboard_effects::KeyboardOverlay;
use crate::editor::media_preview::BakeGeometry;
use crate::editor::preview_platform::annotation_gpu::SourceAnnotations;
use crate::editor::preview_platform::compositor::{CanvasGeometry, Compositor, SourceTexture};
use crate::screenshots::{CapturedImage, ScreenshotOutputSettings, StillOverlay};

/// A picture a layer is drawn from: a decoded image, or a CoreVideo buffer a
/// decoder vends, which is drawn from without a copy.
#[derive(Clone, Copy)]
pub(crate) enum LayerPicture<'a> {
  Image(&'a CapturedImage),
  Pixels(*mut std::ffi::c_void),
}

/// One layer as a present hands it over.
pub(crate) struct StagedLayer<'a> {
  pub(crate) pane_index: u32,
  /// Which layer a gesture or the crop magnifier addresses.
  pub(crate) layer_id: u32,
  /// The cache key of the picture: a new token is a new picture.
  pub(crate) source_token: u64,
  pub(crate) source: LayerPicture<'a>,
  /// A screenshot's own pixels, which its redactions read their fills from.
  /// A recording frame has none: its fills are held from its clips' frames.
  pub(crate) redaction_picture: Option<&'a CapturedImage>,
  pub(crate) settings: &'a ScreenshotOutputSettings,
  pub(crate) seconds: f64,
  pub(crate) cursor: Option<GpuCursor>,
  pub(crate) keyboard: Option<KeyboardOverlay>,
  /// The camera frame composed into the layer, and where its overlay puts
  /// it; without an overlay the frame is held but not drawn.
  pub(crate) camera: Option<LayerPicture<'a>>,
  pub(crate) overlay: Option<&'a StillOverlay>,
  pub(crate) foreground_only: bool,
  pub(crate) hover: Option<(usize, f32)>,
}

/// A picture on the GPU, and the buffer a CoreVideo one is drawn from: the
/// scene holds the buffer, so its decoder cannot hand it to another frame.
pub(super) struct ScenePicture {
  pub(super) texture: SourceTexture,
  _pixels: Option<arc::R<cv::PixelBuf>>,
}

// SAFETY: the buffer is only held to keep it alive and is never read through
// here; retaining and releasing a CVPixelBuffer is safe from any thread.
unsafe impl Send for ScenePicture {}
unsafe impl Sync for ScenePicture {}

impl ScenePicture {
  pub(super) fn new(compositor: &Compositor, picture: LayerPicture<'_>) -> Result<Self, String> {
    match picture {
      LayerPicture::Image(image) => Ok(Self {
        texture: compositor.screenshot_source(image)?,
        _pixels: None,
      }),
      LayerPicture::Pixels(pointer) => {
        let pixels = unsafe { pointer.cast::<cv::PixelBuf>().as_ref() }
          .ok_or_else(|| "The workspace frame is missing".to_owned())?;
        let texture =
          crate::gpu::macos::bgra_buffer_texture(compositor.gpu(), pixels, "workspace frame")?;
        Ok(Self {
          texture: SourceTexture {
            size: (texture.width(), texture.height()),
            view: texture.create_view(&Default::default()),
            texture,
            picture: None,
          },
          _pixels: Some(pixels.retained()),
        })
      }
    }
  }
}

impl LayerPicture<'_> {
  pub(super) fn size(self) -> (u32, u32) {
    match self {
      Self::Image(image) => (image.width, image.height),
      Self::Pixels(pointer) => unsafe { pointer.cast::<cv::PixelBuf>().as_ref() }
        .map_or((0, 0), |pixels| {
          (pixels.width() as u32, pixels.height() as u32)
        }),
    }
  }
}

/// A layer's camera bubble: its frame, the part of it shown and where that
/// sits on the canvas.
#[derive(Clone)]
pub(super) struct LayerCamera {
  pub(super) picture: Option<Arc<ScenePicture>>,
  /// Its output size is the canvas's, filled in when the layer is drawn.
  pub(super) geometry: BakeGeometry,
  pub(super) drop_shadow: bool,
  pub(super) on_top: bool,
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
    }
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
  pub(super) camera: Option<LayerCamera>,
  pub(super) foreground_only: bool,
  pub(super) hover: Option<(usize, f32)>,
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
