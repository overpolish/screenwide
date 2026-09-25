// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  pub(crate) fn export_compositor(
    &self,
    source_size: (u32, u32),
    output_size: (u32, u32),
  ) -> Result<WindowsExportCompositor, String> {
    let source = self
      .inner
      .gpu
      .compositor
      .source(&self.inner.gpu.device, source_size)?;
    Ok(WindowsExportCompositor {
      camera: None,
      inner: std::sync::Arc::clone(&self.inner),
      output_size,
      source,
    })
  }

  pub(crate) fn export_compositor_with_camera(
    &self,
    source_size: (u32, u32),
    camera_size: (u32, u32),
    output_size: (u32, u32),
  ) -> Result<WindowsExportCompositor, String> {
    let mut compositor = self.export_compositor(source_size, output_size)?;
    compositor.camera = Some(
      self
        .inner
        .gpu
        .compositor
        .source(&self.inner.gpu.device, camera_size)?,
    );
    Ok(compositor)
  }
}

impl WindowsExportCompositor {
  pub(in crate::editor) fn compose_with_camera(
    &self,
    texture: &ID3D11Texture2D,
    subresource: u32,
    settings: &ScreenshotOutputSettings,
    composition: ComposedFrame,
    camera: Option<(&ID3D11Texture2D, u32, BakeGeometry, bool, bool)>,
    // What this frame draws. A timed annotation has already had its reveal
    // resolved for this frame by the caller, which owns the timeline.
    annotations: &[crate::editor::annotations::Annotation],
  ) -> Result<ID3D11Texture2D, String> {
    let _state = self
      .inner
      .state
      .lock()
      .map_err(|_| "The Windows GPU compositor is unavailable".to_owned())?;
    compositor::Compositor::copy_source(
      &self.inner.gpu.context,
      &self.source,
      texture,
      subresource,
    )?;
    if let (Some(camera_source), Some((camera_texture, camera_subresource, _, _, _))) =
      (&self.camera, camera)
    {
      compositor::Compositor::copy_source(
        &self.inner.gpu.context,
        camera_source,
        camera_texture,
        camera_subresource,
      )?;
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
    let mut target = None;
    unsafe {
      self
        .inner
        .gpu
        .device
        .CreateTexture2D(&description, None, Some(&mut target))
    }
    .map_err(|error| format!("The Windows export target could not be created: {error}"))?;
    let target = target.ok_or_else(|| "D3D11 created no Windows export target".to_owned())?;
    let prepared = super::annotation::prepared_arrows(
      annotations,
      self.source.size,
      settings,
      None,
      None,
      None,
    )?;
    self.inner.gpu.compositor.draw_with_camera(
      &self.inner.gpu.context,
      &target,
      &self.source,
      settings,
      composition,
      camera.and_then(|(_, _, geometry, drop_shadow, camera_on_top)| {
        self
          .camera
          .as_ref()
          .map(|source| (source, geometry, drop_shadow, camera_on_top))
      }),
      None,
      // Each frame bakes the annotations its settings carry. A timed annotation
      // has already had its reveal resolved for this frame, so the export draws
      // the same arrow the preview showed at that moment.
      &prepared,
    )?;
    unsafe { self.inner.gpu.context.Flush() };
    Ok(target)
  }
}
