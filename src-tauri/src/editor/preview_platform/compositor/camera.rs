// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A baked camera composed on its own before the screen's canvas draws it:
//! the camera's redactions and highlights drawn into a picture the size of
//! the camera frame, which the canvas then samples where it would have
//! sampled the raw frame. They change the camera's pixels, so they end at
//! its edge as its pixels do. See `annotations/camera_baked.rs` for why
//! nothing else is drawn here.

use super::*;
use crate::editor::annotations::Annotation;
use crate::editor::annotations::AnnotationPoint;
use crate::editor::preview_platform::annotation_gpu::SourceAnnotations;
use crate::screenshots::NormalizedSourceRect;

/// Where a camera composition's redactions are copied: clear of every slot
/// a layer takes, which counts up from zero.
const REDACTION_SLOTS: usize = 1 << 16;

/// The annotations drawn into one camera frame, resolved against it once.
pub(crate) struct CameraComposition {
  annotations: SourceAnnotations,
  settings: ScreenshotOutputSettings,
  geometry: CanvasGeometry,
}

impl CameraComposition {
  /// `camera` is the camera's own settings: the annotations drawn into its
  /// picture, in the camera's `source` pixels, and in its capture scale how
  /// many of those pixels a point of size covers. `decoded` is the size of
  /// the frame they are drawn into, which a proxy decodes smaller. `None`
  /// where there is nothing to draw, and the raw frame is drawn instead.
  pub(crate) fn new(
    camera: &ScreenshotOutputSettings,
    source: (u32, u32),
    decoded: (u32, u32),
  ) -> Result<Option<Self>, String> {
    if camera.annotations.is_empty()
      || source.0 == 0
      || source.1 == 0
      || decoded.0 < 64
      || decoded.1 < 64
    {
      return Ok(None);
    }
    let sx = f64::from(decoded.0) / f64::from(source.0);
    let sy = f64::from(decoded.1) / f64::from(source.1);
    let annotations = camera
      .annotations
      .iter()
      .map(|annotation| Annotation {
        shape: annotation.shape.mapped(|point| AnnotationPoint {
          x: point.x * sx,
          y: point.y * sy,
        }),
        ..annotation.clone()
      })
      .collect::<Vec<_>>();
    let (width, height) = (f64::from(decoded.0), f64::from(decoded.1));
    let size_scale = camera.size_scale() * sx;
    let settings = ScreenshotOutputSettings {
      annotations: Vec::new(),
      background_image_path: None,
      background_radius_percent: 0.0,
      background_type: "solid".to_owned(),
      capture_scale: size_scale,
      capture_width_points: width / size_scale,
      crop_height: height,
      crop_preview: None,
      crop_width: width,
      crop_x: 0.0,
      crop_y: 0.0,
      drop_shadow: false,
      height: decoded.1,
      image_width: width,
      image_x: 0.0,
      image_y: 0.0,
      radius_percent: 0.0,
      recenter_inset_color: None,
      scene_motion: None,
      scene_opacity: None,
      scene_camera_front: None,
      source_crop: NormalizedSourceRect {
        height: 1.0,
        width: 1.0,
        x: 0.0,
        y: 0.0,
      },
      width: decoded.0,
      ..camera.clone()
    };
    Ok(Some(Self {
      annotations: SourceAnnotations::new(&annotations, decoded, &settings, None),
      geometry: CanvasGeometry::of(decoded, &settings)?,
      settings,
    }))
  }
}

impl Compositor {
  /// `camera` with `composition` drawn into it, in a picture of its own size
  /// that stays readable until `slot` composes again. `hover` is the halo on
  /// one of the annotations, its width in the frame's pixels; `retained`
  /// says the frame's pixels never change under it.
  pub(crate) fn composed_camera(
    &self,
    camera: &SourceTexture,
    composition: &CameraComposition,
    hover: Option<(usize, f32)>,
    slot: usize,
    retained: bool,
  ) -> Result<SourceTexture, String> {
    if composition.geometry.size != camera.size {
      return Err("The camera composition does not match its frame".to_owned());
    }
    let canvas = self.camera_canvas(slot, camera.size)?;
    let prepared = composition
      .annotations
      .placed(composition.geometry.placement, hover, None);
    self.draw_layer(
      &canvas.view,
      camera,
      &composition.settings,
      &composition.geometry,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: true,
        seconds: 0.0,
      },
      None,
      None,
      &prepared,
      LayerDraw {
        placement: None,
        clear: true,
        redaction_slot: REDACTION_SLOTS + slot,
        retained_source: retained,
      },
    )?;
    Ok(canvas)
  }

  /// `slot`'s camera picture, made again only when the frame's size changes.
  fn camera_canvas(&self, slot: usize, size: (u32, u32)) -> Result<SourceTexture, String> {
    let mut canvases = self
      .camera_canvases
      .lock()
      .map_err(|_| "The camera pictures are poisoned".to_owned())?;
    if let Some(canvas) = canvases.get(&slot).filter(|canvas| canvas.size == size) {
      return Ok(canvas.clone());
    }
    let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Screenwide composed camera"),
      size: wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: FORMAT,
      usage: wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let canvas = SourceTexture {
      size,
      view: texture.create_view(&Default::default()),
      texture,
      #[cfg(target_os = "windows")]
      shared: None,
      picture: None,
    };
    canvases.insert(slot, canvas.clone());
    Ok(canvas)
  }
}
