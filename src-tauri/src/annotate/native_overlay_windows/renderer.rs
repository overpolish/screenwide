// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's pipeline: one device, one shader, one buffer per list.

use super::*;

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/annotate_overlay_vs.cso"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/annotate_overlay_ps.cso"));

/// The shader's own constants. A constant buffer is a multiple of sixteen
/// bytes wide, which is what the padding is for.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct Constants {
  pub(super) count: u32,
  /// How wide an edge is smoothed. One layer pixel: the overlay draws at the
  /// display's own resolution, so there is nothing to widen the band for.
  pub(super) feather: f32,
  pub(super) padding: [u32; 2],
}

const _: () = assert!(std::mem::size_of::<Constants>() == 16);

/// Everything every surface of one session shares.
pub(super) struct Renderer {
  device: Arc<overlay_surface::Device>,
  pub(super) vertex_shader: ID3D11VertexShader,
  pub(super) pixel_shader: ID3D11PixelShader,
  pub(super) constants: ID3D11Buffer,
  pub(super) arrows: ID3D11Buffer,
  pub(super) arrow_view: ID3D11ShaderResourceView,
  pub(super) samples: ID3D11Buffer,
  pub(super) sample_view: ID3D11ShaderResourceView,
}

impl Renderer {
  pub(super) fn new() -> Result<Self, String> {
    let device = overlay_surface::Device::new()?;
    let handle = device.device();
    let mut vertex_shader = None;
    let mut pixel_shader = None;
    unsafe {
      handle
        .CreateVertexShader(VERTEX_SHADER, None, Some(&mut vertex_shader))
        .map_err(|error| error.to_string())?;
      handle
        .CreatePixelShader(PIXEL_SHADER, None, Some(&mut pixel_shader))
        .map_err(|error| error.to_string())?;
    }
    let mut constants = None;
    unsafe {
      handle.CreateBuffer(
        &D3D11_BUFFER_DESC {
          ByteWidth: size_of::<Constants>() as u32,
          Usage: D3D11_USAGE_DEFAULT,
          BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
          ..Default::default()
        },
        None,
        Some(&mut constants),
      )
    }
    .map_err(|error| error.to_string())?;
    // Fixed-capacity dynamic buffers, as the editor's compositor uses: the
    // annotation cap is small and known, so every frame maps into the same
    // allocation rather than building a buffer.
    let (arrows, arrow_view) = arrows::structured_buffer(
      handle,
      size_of::<arrows::PreviewArrow>(),
      MAX_ANNOTATIONS,
      "annotate arrow",
    )?;
    let (samples, sample_view) = arrows::structured_buffer(
      handle,
      size_of::<arrows::PreviewSample>(),
      MAX_ANNOTATIONS * arrows::MAX_EXPOSURE_SAMPLES,
      "annotate arrow exposure",
    )?;
    Ok(Self {
      device,
      vertex_shader: vertex_shader
        .ok_or_else(|| "D3D11 created no annotate vertex shader".to_owned())?,
      pixel_shader: pixel_shader
        .ok_or_else(|| "D3D11 created no annotate pixel shader".to_owned())?,
      constants: constants.ok_or_else(|| "D3D11 created no annotate constants".to_owned())?,
      arrows,
      arrow_view,
      samples,
      sample_view,
    })
  }

  pub(super) fn device(&self) -> &Arc<overlay_surface::Device> {
    &self.device
  }

  /// Draws the prepared arrows over a cleared target of `size` physical
  /// pixels. Shared by a display's frame and by a still that leaves the app as
  /// pixels, so both draw the same arrows the same way.
  pub(super) fn draw_arrows(
    &self,
    target: &ID3D11RenderTargetView,
    size: (u32, u32),
    prepared: &arrows::PreparedArrows,
  ) -> Result<(), String> {
    let drawn = &prepared.arrows[..prepared.arrows.len().min(MAX_ANNOTATIONS)];
    let context = self.device.context();
    upload(context, &self.arrows, drawn)?;
    upload(context, &self.samples, &prepared.samples)?;
    let constants = Constants {
      count: drawn.len() as u32,
      feather: 1.0,
      padding: [0; 2],
    };
    unsafe {
      context.UpdateSubresource(
        &self.constants,
        0,
        None,
        (&raw const constants).cast::<c_void>(),
        0,
        0,
      );
      // Flip-discard back buffers are undefined after a present, and the
      // shader composes over transparent rather than over what was there.
      context.ClearRenderTargetView(target, &[0.0; 4]);
      context.OMSetRenderTargets(Some(&[Some(target.clone())]), None);
      context.RSSetViewports(Some(&[D3D11_VIEWPORT {
        Width: size.0 as f32,
        Height: size.1 as f32,
        MaxDepth: 1.0,
        ..Default::default()
      }]));
      context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      context.VSSetShader(&self.vertex_shader, None);
      context.PSSetShader(&self.pixel_shader, None);
      context.PSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
      // The included shader reads its two lists at t5 and t6, the slots the
      // editor's compositor binds them at.
      context.PSSetShaderResources(
        5,
        Some(&[
          Some(self.arrow_view.clone()),
          Some(self.sample_view.clone()),
        ]),
      );
      // The result is already premultiplied, so it is written rather than
      // blended: one pass over a cleared target has nothing to blend with.
      context.Draw(3, 0);
      context.PSSetShaderResources(5, Some(&[None, None]));
      context.OMSetRenderTargets(None, None);
    }
    Ok(())
  }
}

/// Writes `items` to the front of a CPU-written structured buffer. Nothing is
/// written for an empty list; the shader never reads past its counts.
pub(super) fn upload<T: Copy>(
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
