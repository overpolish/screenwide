// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's pipeline: one device, one shader, one buffer per list.

use super::*;

use windows::core::Interface;
use windows::Win32::Graphics::Direct3D11::{
  ID3D11Resource, ID3D11Texture2D, D3D11_BIND_SHADER_RESOURCE, D3D11_SUBRESOURCE_DATA,
  D3D11_TEXTURE2D_DESC, D3D11_USAGE_IMMUTABLE,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/annotate_overlay_vs.cso"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/annotate_overlay_ps.cso"));

/// The shader's own constants. A constant buffer is a multiple of sixteen
/// bytes wide, which the atlas size fills out.
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct Constants {
  pub(super) count: u32,
  /// How wide an edge is smoothed. One layer pixel: the overlay draws at the
  /// display's own resolution, so there is nothing to widen the band for.
  pub(super) feather: f32,
  /// The size of the texture the counters' numbers were rasterised into, in
  /// pixels, or zeroes when no counter is on screen to rasterise one.
  pub(super) atlas: [u32; 2],
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
  /// The counters' numbers, rasterised at the size they are drawn. Cached
  /// against the numbers and sizes it was drawn from, so a frame that redraws
  /// an unchanged screen does no work.
  counters: arrows::CounterArtworkCache,
  /// Bound where the numbers go when there is none: a shader resource slot
  /// left empty is a debug-layer complaint, and the number pass returns on
  /// the zero atlas size before it would sample this.
  empty_numbers: ID3D11ShaderResourceView,
  _empty_texture: ID3D11Texture2D,
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
    let (empty_texture, empty_numbers) = empty_atlas(handle)?;
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
      counters: arrows::CounterArtworkCache::default(),
      empty_numbers,
      _empty_texture: empty_texture,
    })
  }

  pub(super) fn device(&self) -> &Arc<overlay_surface::Device> {
    &self.device
  }

  /// Draws the prepared annotations over a cleared target of `size` physical
  /// pixels. Shared by a display's frame and by a still that leaves the app as
  /// pixels, so both draw the same annotations the same way.
  ///
  /// A counter's number is type rather than a shape the shader can solve, so
  /// it is rasterised here, at the size it is drawn, and handed over as one
  /// texture - the same pass the editor's compositor makes.
  pub(super) fn draw_arrows(
    &self,
    target: &ID3D11RenderTargetView,
    size: (u32, u32),
    prepared: &arrows::PreparedArrows,
  ) -> Result<(), String> {
    let (numbers, numbered) =
      arrows::numbered_arrows(&self.counters, self.device.device(), prepared)?;
    let drawn = &numbered[..numbered.len().min(MAX_ANNOTATIONS)];
    let context = self.device.context();
    upload(context, &self.arrows, drawn)?;
    upload(context, &self.samples, &prepared.samples)?;
    let constants = Constants {
      count: drawn.len() as u32,
      feather: 1.0,
      atlas: numbers.as_ref().map_or([0, 0], |atlas| {
        let (width, height) = atlas.size;
        [width, height]
      }),
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
      // The included shader reads its two lists at t5 and t6 and the numbers
      // at t7, the slots the editor's compositor binds them at.
      context.PSSetShaderResources(
        5,
        Some(&[
          Some(self.arrow_view.clone()),
          Some(self.sample_view.clone()),
          Some(
            numbers
              .as_ref()
              .map_or_else(|| self.empty_numbers.clone(), |atlas| atlas.view.clone()),
          ),
        ]),
      );
      // The result is already premultiplied, so it is written rather than
      // blended: one pass over a cleared target has nothing to blend with.
      context.Draw(3, 0);
      context.PSSetShaderResources(5, Some(&[None, None, None]));
      context.OMSetRenderTargets(None, None);
    }
    Ok(())
  }
}

/// A single transparent pixel, bound where the counters' numbers go on a
/// frame that has none. The shader's number pass returns on the zero atlas
/// size before it reads this, so what it holds never shows.
fn empty_atlas(
  device: &ID3D11Device,
) -> Result<(ID3D11Texture2D, ID3D11ShaderResourceView), String> {
  let mut texture = None;
  unsafe {
    device.CreateTexture2D(
      &D3D11_TEXTURE2D_DESC {
        Width: 1,
        Height: 1,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
          Count: 1,
          Quality: 0,
        },
        Usage: D3D11_USAGE_IMMUTABLE,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        ..Default::default()
      },
      Some(&D3D11_SUBRESOURCE_DATA {
        pSysMem: [0_u8; 4].as_ptr().cast::<c_void>(),
        SysMemPitch: 4,
        SysMemSlicePitch: 0,
      }),
      Some(&mut texture),
    )
  }
  .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "D3D11 created no annotate number texture".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  Ok((
    texture,
    view.ok_or_else(|| "D3D11 created no annotate number view".to_owned())?,
  ))
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
