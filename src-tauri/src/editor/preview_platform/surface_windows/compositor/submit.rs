// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  #[allow(clippy::too_many_arguments)]
  pub(super) fn submit(
    &self,
    context: &ID3D11DeviceContext,
    device: &ID3D11Device,
    target: &ID3D11Texture2D,
    source: &SourceTexture,
    settings: &ScreenshotOutputSettings,
    composition: super::super::ComposedFrame,
    camera: Option<(&SourceTexture, BakeGeometry, bool, bool)>,
    picture: Option<std::sync::Arc<super::super::background_image::BackgroundImage>>,
    mut values: Constants,
    // The prepared annotations, their exposure samples, and what each counter's
    // number is: the numbers are rasterised here, at the size they are drawn.
    prepared: &PreparedArrows,
  ) -> Result<(), String> {
    let (numbers, annotations) =
      super::super::counter_artwork::numbered_arrows(&self.counter_cache, device, prepared)?;
    let samples = &prepared.samples;
    values.annotation_options[2] = numbers.as_ref().map_or(0, |atlas| atlas.size.0);
    values.annotation_options[3] = numbers.as_ref().map_or(0, |atlas| atlas.size.1);
    let target_resource: ID3D11Resource = target.cast().map_err(|error| error.to_string())?;
    let mut render_target: Option<ID3D11RenderTargetView> = None;
    let keyboard = composition
      .keyboard
      .map(|overlay| {
        self
          .keyboard_cache
          .resolve(device, &overlay, settings.height)
      })
      .transpose()?
      .flatten();
    let keyboard_values = keyboard
      .as_ref()
      .map_or_else(KeyboardConstants::default, |(_, values)| *values);
    unsafe {
      self
        .keyboard_constants
        .cast::<ID3D11Resource>()
        .map_err(|error| error.to_string())
        .map(|resource| {
          context.UpdateSubresource(
            &resource,
            0,
            None,
            (&raw const keyboard_values).cast::<c_void>(),
            0,
            0,
          );
        })?;
    }
    unsafe {
      self
        .constants
        .cast::<ID3D11Resource>()
        .map_err(|error| error.to_string())
        .map(|resource| {
          context.UpdateSubresource(
            &resource,
            0,
            None,
            (&raw const values).cast::<c_void>(),
            0,
            0,
          );
        })?;
    }
    unsafe { device.CreateRenderTargetView(&target_resource, None, Some(&mut render_target)) }
      .map_err(|error| error.to_string())?;
    let render_target =
      render_target.ok_or_else(|| "D3D11 created no preview render target".to_owned())?;
    let viewport = D3D11_VIEWPORT {
      Width: settings.width as f32,
      Height: settings.height as f32,
      MaxDepth: 1.0,
      ..Default::default()
    };
    // The caps are enforced above the facade, but a longer list must clamp
    // rather than run past the buffer its view describes.
    upload(
      context,
      &self.annotation_buffer,
      &annotations[..annotations.len().min(MAX_ANNOTATIONS)],
    )?;
    upload(
      context,
      &self.sample_buffer,
      &samples[..samples.len().min(MAX_ANNOTATIONS * MAX_EXPOSURE_SAMPLES)],
    )?;
    unsafe {
      context.OMSetRenderTargets(Some(&[Some(render_target)]), None);
      context.OMSetBlendState(
        composition.foreground_only.then_some(&self.layer_blend),
        None,
        u32::MAX,
      );
      context.RSSetViewports(Some(&[viewport]));
      context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      context.VSSetShader(&self.vertex_shader, None);
      context.PSSetShader(&self.pixel_shader, None);
      context.PSSetConstantBuffers(
        0,
        Some(&[
          Some(self.constants.clone()),
          Some(self.keyboard_constants.clone()),
        ]),
      );
      context.PSSetShaderResources(
        0,
        Some(&[
          Some(source.view.clone()),
          Some(self.cursor_view.clone()),
          camera.map(|(camera, _, _, _)| camera.view.clone()),
          Some(keyboard.as_ref().map_or_else(
            || self.fallback_view.clone(),
            |(artwork, _)| artwork.view.clone(),
          )),
          Some(picture.as_ref().map_or_else(
            || self.fallback_view.clone(),
            |picture| picture.view.clone(),
          )),
          Some(self.annotation_view.clone()),
          Some(self.sample_view.clone()),
          Some(
            numbers
              .as_ref()
              .map_or_else(|| self.fallback_view.clone(), |atlas| atlas.view.clone()),
          ),
        ]),
      );
      context.PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));
      context.PSSetSamplers(1, Some(&[Some(self.point_sampler.clone())]));
      context.Draw(3, 0);
      context.PSSetShaderResources(0, Some(&[None, None, None, None, None, None, None, None]));
      context.OMSetBlendState(None::<&ID3D11BlendState>, None, u32::MAX);
      context.OMSetRenderTargets(None, None);
    }
    Ok(())
  }
}

/// Writes `items` to the front of a CPU-written structured buffer. Nothing
/// is written for an empty list; the shader never reads past its counts.
fn upload<T: Copy>(
  context: &ID3D11DeviceContext,
  buffer: &ID3D11Buffer,
  items: &[T],
) -> Result<(), String> {
  if items.is_empty() {
    return Ok(());
  }
  let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
  unsafe {
    context
      .Map(buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut mapped))
      .map_err(|error| error.to_string())?;
    std::ptr::copy_nonoverlapping(items.as_ptr(), mapped.pData.cast::<T>(), items.len());
    context.Unmap(buffer, 0);
  }
  Ok(())
}
