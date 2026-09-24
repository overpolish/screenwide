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
    let output_size = crate::screenshots::output_dimensions(first_settings)?;
    let target_description = D3D11_TEXTURE2D_DESC {
      Width: output_size.0,
      Height: output_size.1,
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
        .CreateTexture2D(&target_description, None, Some(&mut target))
    }
    .map_err(|error| format!("The screenshot render target could not be created: {error}"))?;
    let target = target.ok_or_else(|| "D3D11 created no screenshot render target".to_owned())?;

    for (index, (image, settings)) in layers.iter().enumerate() {
      if crate::screenshots::output_dimensions(settings)? != output_size {
        return Err("The screenshot layers do not share a canvas size".to_owned());
      }
      let source = self
        .inner
        .gpu
        .compositor
        .screenshot_source(&self.inner.gpu.device, image)?;
      let prepared = super::annotation::prepared_arrows(
        &settings.annotations,
        (image.width, image.height),
        settings,
        // The export never carries a halo, nor a caret.
        None,
        None,
      )?;
      self.inner.gpu.compositor.draw_with_camera(
        &self.inner.gpu.context,
        &target,
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
    unsafe { self.inner.gpu.context.Flush() };
    self.readback_bgra(&target, target_description, output_size, "screenshot")
  }

  pub(in crate::editor) fn compose_texture_to_image(
    &self,
    texture: &ID3D11Texture2D,
    subresource: u32,
    source_size: (u32, u32),
    settings: &ScreenshotOutputSettings,
    composition: ComposedFrame,
    camera: Option<ClipboardCamera<'_>>,
  ) -> Result<CapturedImage, String> {
    let _state = self
      .inner
      .state
      .lock()
      .map_err(|_| "The Windows preview surface is unavailable".to_owned())?;
    let output_size = crate::screenshots::output_dimensions(settings)?;
    let source = self
      .inner
      .gpu
      .compositor
      .source(&self.inner.gpu.device, source_size)?;
    compositor::Compositor::copy_source(&self.inner.gpu.context, &source, texture, subresource)?;
    let camera_source = camera
      .map(|(_, _, size, _, _, _)| {
        self
          .inner
          .gpu
          .compositor
          .source(&self.inner.gpu.device, size)
      })
      .transpose()?;
    if let (Some(camera_source), Some((texture, subresource, _, _, _, _))) =
      (&camera_source, camera)
    {
      compositor::Compositor::copy_source(
        &self.inner.gpu.context,
        camera_source,
        texture,
        subresource,
      )?;
    }
    let target_description = D3D11_TEXTURE2D_DESC {
      Width: output_size.0,
      Height: output_size.1,
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
        .CreateTexture2D(&target_description, None, Some(&mut target))
    }
    .map_err(|error| format!("The clipboard render target could not be created: {error}"))?;
    let target = target.ok_or_else(|| "D3D11 created no clipboard render target".to_owned())?;
    let prepared =
      super::annotation::prepared_arrows(&settings.annotations, source_size, settings, None, None)?;
    self.inner.gpu.compositor.draw_with_camera(
      &self.inner.gpu.context,
      &target,
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

    self.readback_bgra(&target, target_description, output_size, "clipboard")
  }

  pub(super) fn readback_bgra(
    &self,
    target: &ID3D11Texture2D,
    target_description: D3D11_TEXTURE2D_DESC,
    output_size: (u32, u32),
    purpose: &str,
  ) -> Result<CapturedImage, String> {
    let staging_description = D3D11_TEXTURE2D_DESC {
      Usage: D3D11_USAGE_STAGING,
      BindFlags: 0,
      CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
      ..target_description
    };
    let mut staging = None;
    unsafe {
      self
        .inner
        .gpu
        .device
        .CreateTexture2D(&staging_description, None, Some(&mut staging))
    }
    .map_err(|error| format!("The {purpose} readback texture could not be created: {error}"))?;
    let staging = staging.ok_or_else(|| format!("D3D11 created no {purpose} readback texture"))?;
    let target_resource: windows::Win32::Graphics::Direct3D11::ID3D11Resource =
      target.cast().map_err(|error| error.to_string())?;
    let staging_resource: windows::Win32::Graphics::Direct3D11::ID3D11Resource =
      staging.cast().map_err(|error| error.to_string())?;
    unsafe {
      self
        .inner
        .gpu
        .context
        .CopyResource(&staging_resource, &target_resource);
    }
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    unsafe {
      self
        .inner
        .gpu
        .context
        .Map(&staging_resource, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
    }
    .map_err(|error| format!("The {purpose} frame could not be read back: {error}"))?;
    let row_bytes = output_size.0 as usize * 4;
    let mut rgba = vec![0_u8; row_bytes * output_size.1 as usize];
    if mapped.pData.is_null() || mapped.RowPitch < row_bytes as u32 {
      unsafe { self.inner.gpu.context.Unmap(&staging_resource, 0) };
      return Err(format!("D3D11 returned invalid {purpose} pixels"));
    }
    for row in 0..output_size.1 as usize {
      let source_row = unsafe {
        std::slice::from_raw_parts(
          mapped
            .pData
            .cast::<u8>()
            .add(row * mapped.RowPitch as usize),
          row_bytes,
        )
      };
      let target_row = &mut rgba[row * row_bytes..(row + 1) * row_bytes];
      for (source_pixel, target_pixel) in source_row
        .chunks_exact(4)
        .zip(target_row.chunks_exact_mut(4))
      {
        target_pixel.copy_from_slice(&[
          source_pixel[2],
          source_pixel[1],
          source_pixel[0],
          source_pixel[3],
        ]);
      }
    }
    unsafe { self.inner.gpu.context.Unmap(&staging_resource, 0) };
    Ok(CapturedImage {
      height: output_size.1,
      rgba,
      width: output_size.0,
    })
  }
}
