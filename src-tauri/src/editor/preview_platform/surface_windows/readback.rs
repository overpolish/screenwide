// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  /// Renders one explicit clipboard frame through the exact preview shader,
  /// then performs the single unavoidable GPU readback required by the
  /// Windows clipboard. Live preview never calls this path.
  pub(in crate::editor) fn compose_screenshot_layers_to_image(
    &self,
    layers: &[(&CapturedImage, ScreenshotOutputSettings)],
  ) -> Result<CapturedImage, String> {
    let (_, first_settings) = layers
      .first()
      .ok_or_else(|| "The screenshot workspace is empty".to_owned())?;
    let _state = self
      .inner
      .state
      .lock()
      .map_err(|_| "The Windows preview surface is unavailable".to_owned())?;
    let gpu = &self.inner.gpu;
    let output_size = crate::screenshots::output_dimensions(first_settings)?;
    let target = readback_target(gpu.shared, output_size);
    let view = target.create_view(&Default::default());
    for (index, (image, settings)) in layers.iter().enumerate() {
      if crate::screenshots::output_dimensions(settings)? != output_size {
        return Err("The screenshot layers do not share a canvas size".to_owned());
      }
      let source = gpu.compositor.screenshot_source(image)?;
      let prepared = crate::editor::preview_platform::annotation_gpu::prepared_arrows(
        &settings.annotations,
        (image.width, image.height),
        settings,
        Some(*image),
        // The export never carries a halo, nor a caret.
        None,
        None,
      )?;
      gpu.compositor.draw_with_camera(
        &view,
        &source,
        settings,
        ComposedFrame {
          cursor: None,
          keyboard: None,
          foreground_only: index > 0,
          seconds: 0.0,
        },
        None,
        None,
        // The export bakes the same prepared arrows the preview draws, so the
        // file matches what the editor showed. The halo never reaches here:
        // it is preview chrome the flattening does not carry.
        &prepared,
      )?;
    }
    read_rgba(gpu.shared, &target)
  }

  /// The decoded frame `decoded` composed as the preview draws it, with
  /// the camera's effects `effects`, read back.
  pub(in crate::editor) fn compose_texture_to_image(
    &self,
    (texture, subresource, source_size): DecodedTexture<'_>,
    settings: &ScreenshotOutputSettings,
    composition: ComposedFrame,
    camera: Option<ClipboardCamera<'_>>,
    effects: Option<crate::editor::camera_effects::preview::StillCameraEffects<'_>>,
  ) -> Result<CapturedImage, String> {
    let _state = self
      .inner
      .state
      .lock()
      .map_err(|_| "The Windows preview surface is unavailable".to_owned())?;
    let gpu = &self.inner.gpu;
    let output_size = crate::screenshots::output_dimensions(settings)?;
    let source = gpu.compositor.source(gpu.d3d11, source_size)?;
    gpu
      .compositor
      .copy_source(gpu.d3d11, &source, texture, subresource)?;
    let camera_source = camera
      .map(|(_, _, size, _, _, _)| gpu.compositor.source(gpu.d3d11, size))
      .transpose()?;
    if let (Some(camera_source), Some((texture, subresource, _, _, _, _))) =
      (&camera_source, camera)
    {
      gpu
        .compositor
        .copy_source(gpu.d3d11, camera_source, texture, subresource)?;
    }
    // The still draws into slots of its own, beside the preview's frames.
    let (source, camera_source) = match effects {
      Some(effects) => {
        let mut pass = self
          .inner
          .camera_effects
          .lock()
          .map_err(|_| "The camera's effects are poisoned".to_owned())?;
        let mut draw = |picture: &compositor::SourceTexture, slot| {
          pass.drawn_once(
            &gpu.compositor,
            (effects.live, effects.matte),
            picture,
            slot,
          )
        };
        if effects.on_screen {
          (draw(&source, 2)?, camera_source)
        } else {
          let camera = camera_source
            .as_ref()
            .map(|camera| draw(camera, 3))
            .transpose()?;
          (source, camera)
        }
      }
      None => (source, camera_source),
    };
    let target = readback_target(gpu.shared, output_size);
    let prepared = crate::editor::preview_platform::annotation_gpu::prepared_arrows(
      &settings.annotations,
      source_size,
      settings,
      None,
      None,
      None,
    )?;
    gpu.compositor.draw_with_camera(
      &target.create_view(&Default::default()),
      &source,
      settings,
      composition,
      camera.and_then(|(_, _, _, geometry, drop_shadow, camera_on_top)| {
        camera_source
          .as_ref()
          .map(|source| (source, geometry, drop_shadow, camera_on_top))
      }),
      None,
      &prepared,
    )?;
    read_rgba(gpu.shared, &target)
  }
}

fn readback_target(gpu: &crate::gpu::Gpu, size: (u32, u32)) -> wgpu::Texture {
  gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("Screenwide clipboard frame"),
    size: wgpu::Extent3d {
      width: size.0,
      height: size.1,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: wgpu::TextureFormat::Bgra8Unorm,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  })
}

/// The BGRA frame drawn into `target`, as RGBA.
fn read_rgba(gpu: &crate::gpu::Gpu, target: &wgpu::Texture) -> Result<CapturedImage, String> {
  let mut rgba = gpu.read_texture(target)?;
  for pixel in rgba.chunks_exact_mut(4) {
    pixel.swap(0, 2);
  }
  Ok(CapturedImage {
    height: target.height(),
    rgba,
    width: target.width(),
  })
}
