// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  pub(crate) fn export_compositor(
    &self,
    source_size: (u32, u32),
    output_size: (u32, u32),
  ) -> Result<WindowsExportCompositor, String> {
    let gpu = &self.inner.gpu;
    let source = gpu.compositor.source(gpu.d3d11, source_size)?;
    let target = gpu.d3d11.shared_texture(
      gpu.shared,
      output_size,
      wgpu::TextureFormat::Bgra8Unorm,
      "export target",
    )?;
    Ok(WindowsExportCompositor {
      camera: None,
      inner: std::sync::Arc::clone(&self.inner),
      output_size,
      source,
      target,
    })
  }

  pub(crate) fn export_compositor_with_camera(
    &self,
    source_size: (u32, u32),
    camera_size: (u32, u32),
    output_size: (u32, u32),
  ) -> Result<WindowsExportCompositor, String> {
    let mut compositor = self.export_compositor(source_size, output_size)?;
    let gpu = &self.inner.gpu;
    compositor.camera = Some(gpu.compositor.source(gpu.d3d11, camera_size)?);
    Ok(compositor)
  }
}

impl WindowsExportCompositor {
  /// Compiles the canvas pipelines for each set of annotation kinds an
  /// export draws, before its first frame; see
  /// [`compositor::Compositor::prepare_annotation_kinds`].
  pub(in crate::editor) fn prepare_annotation_kinds(&self, sets: &[compositor::KindMask]) {
    self.inner.gpu.compositor.prepare_annotation_kinds(sets);
  }

  /// `camera_composition` is what the baked camera has composed into its
  /// frame before the canvas draws it.
  #[allow(clippy::too_many_arguments)]
  pub(in crate::editor) fn compose_with_camera(
    &self,
    texture: &ID3D11Texture2D,
    subresource: u32,
    settings: &ScreenshotOutputSettings,
    composition: ComposedFrame,
    camera: Option<(&ID3D11Texture2D, u32, BakeGeometry, bool, bool)>,
    camera_composition: Option<&compositor::CameraComposition>,
    // What this frame draws. A timed annotation has already had its reveal
    // resolved for this frame by the caller, which owns the timeline.
    annotations: &[crate::editor::annotations::Annotation],
  ) -> Result<ID3D11Texture2D, String> {
    let _state = self
      .inner
      .state
      .lock()
      .map_err(|_| "The Windows GPU compositor is unavailable".to_owned())?;
    let gpu = &self.inner.gpu;
    gpu
      .compositor
      .copy_source(gpu.d3d11, &self.source, texture, subresource)?;
    let mut drawn_camera = None;
    if let (Some(camera_source), Some((camera_texture, camera_subresource, _, _, _))) =
      (&self.camera, camera)
    {
      gpu
        .compositor
        .copy_source(gpu.d3d11, camera_source, camera_texture, camera_subresource)?;
      // The preview composes its camera in slot 0; the export keeps its own.
      drawn_camera = camera_composition
        .map(|composed| {
          gpu
            .compositor
            .composed_camera(camera_source, composed, None, 1, false)
        })
        .transpose()?;
    }
    // Sink Writer retains DXGI surfaces and feeds the hardware encoder
    // asynchronously. A single repainted render target therefore lets a later
    // frame overwrite an earlier sample before Media Foundation consumes it.
    // Give each submitted sample its own texture; MF's sample owns that texture
    // until encoding completes and naturally bounds outstanding allocations
    // through Sink Writer backpressure.
    let description = D3D11_TEXTURE2D_DESC {
      Width: self.output_size.0,
      Height: self.output_size.1,
      MipLevels: 1,
      ArraySize: 1,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      Usage: D3D11_USAGE_DEFAULT,
      BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
      ..Default::default()
    };
    let mut copy = None;
    unsafe {
      gpu
        .d3d11
        .device
        .CreateTexture2D(&description, None, Some(&mut copy))
    }
    .map_err(|error| format!("The Windows export target could not be created: {error}"))?;
    let copy = copy.ok_or_else(|| "D3D11 created no Windows export target".to_owned())?;
    let prepared = crate::editor::preview_platform::annotation_gpu::prepared_arrows(
      annotations,
      self.source.size,
      settings,
      None,
      None,
      None,
    )?;
    gpu.compositor.draw_with_camera(
      &self.target.view,
      &self.source,
      settings,
      composition,
      camera.and_then(|(_, _, geometry, drop_shadow, camera_on_top)| {
        drawn_camera
          .as_ref()
          .or(self.camera.as_ref())
          .map(|source| (source, geometry, drop_shadow, camera_on_top))
      }),
      None,
      // Each frame bakes the annotations its settings carry. A timed annotation
      // has already had its reveal resolved for this frame, so the export draws
      // the same arrow the preview showed at that moment.
      &prepared,
    )?;
    // The frame was submitted to the shared queue above, so the copy the
    // encoder reads, submitted after it, sees it whole.
    let destination: ID3D11Resource = copy.cast().map_err(|error| error.to_string())?;
    gpu.d3d11.with(&[&self.target], |context, textures| {
      let drawn: ID3D11Resource = textures[0].cast().map_err(|error| error.to_string())?;
      unsafe { context.CopyResource(&destination, &drawn) };
      Ok::<_, String>(())
    })??;
    Ok(copy)
  }
}
