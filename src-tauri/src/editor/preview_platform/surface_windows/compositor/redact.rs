// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The redaction pre-pass. A redaction is applied to a copy of the source
//! rather than drawn over the canvas: the canvas, the crop ghost and the
//! magnifier all sample the source, and each of them must find the covered
//! pixels already gone. The source itself stays untouched, so a redaction
//! that moves, or is removed, uncovers exactly the pixels that were there.
//! The twin of `gpu_compositor_macos+presenter_redact.m`.
//!
//! Each redaction draws over a fresh copy of what the last one left, so a
//! box drawn over another averages what the first left, as on macOS. Two
//! targets take turns being that copy.

use super::*;
use crate::editor::annotations::redact::records::{
  RedactRecord, RedactRecords, REDACT_BLUR, REDACT_MOSAIC,
};
use windows::Win32::Graphics::Direct3D11::D3D11_BIND_RENDER_TARGET;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;

const VERTEX_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/preview_redact_paint_vs.cso"));
const CELLS_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/preview_redact_cells_ps.cso"));
const PAINT_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/preview_redact_paint_ps.cso"));

/// A texture a pass draws into and the next one reads.
struct Target {
  texture: ID3D11Texture2D,
  view: ID3D11ShaderResourceView,
  target: ID3D11RenderTargetView,
}

fn target(device: &ID3D11Device, size: (u32, u32), format: DXGI_FORMAT) -> Result<Target, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: size.0,
    Height: size.1,
    MipLevels: 1,
    ArraySize: 1,
    Format: format,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_DEFAULT,
    BindFlags: (D3D11_BIND_SHADER_RESOURCE.0 | D3D11_BIND_RENDER_TARGET.0) as u32,
    ..Default::default()
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }
    .map_err(|error| format!("The redaction target could not be created: {error}"))?;
  let texture = texture.ok_or_else(|| "D3D11 created no redaction target".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let (mut view, mut target) = (None, None);
  unsafe {
    device
      .CreateShaderResourceView(&resource, None, Some(&mut view))
      .and_then(|()| device.CreateRenderTargetView(&resource, None, Some(&mut target)))
  }
  .map_err(|error| error.to_string())?;
  Ok(Target {
    texture,
    view: view.ok_or_else(|| "D3D11 created no redaction view".to_owned())?,
    target: target.ok_or_else(|| "D3D11 created no redaction target view".to_owned())?,
  })
}

/// The two copies redactions take turns drawing over, sized and formatted
/// like the source, and the last still they redacted.
struct Work {
  size: (u32, u32),
  format: DXGI_FORMAT,
  copies: [Target; 2],
  /// The retained source the last pass redacted, what it redacted, and the
  /// copy it left the result in. A still redrawn for its chrome keeps it.
  kept: Option<(ID3D11Texture2D, RedactRecords, usize)>,
}

pub(crate) struct Redactor {
  vertex_shader: ID3D11VertexShader,
  cells_shader: ID3D11PixelShader,
  paint_shader: ID3D11PixelShader,
  constants: ID3D11Buffer,
  zones: StructuredBuffer,
  work: std::sync::Mutex<Option<Work>>,
  /// The cells a blurred or classically pixelated box averages, one pixel a
  /// cell; grown to fit the largest grid yet.
  cells: std::sync::Mutex<Option<((u32, u32), Target)>>,
}

impl Redactor {
  pub(super) fn new(device: &ID3D11Device) -> Result<Self, String> {
    let (mut vertex_shader, mut cells_shader, mut paint_shader) = (None, None, None);
    unsafe {
      device
        .CreateVertexShader(VERTEX_SHADER, None, Some(&mut vertex_shader))
        .and_then(|()| device.CreatePixelShader(CELLS_SHADER, None, Some(&mut cells_shader)))
        .and_then(|()| device.CreatePixelShader(PAINT_SHADER, None, Some(&mut paint_shader)))
    }
    .map_err(|error| format!("The redaction shaders could not be created: {error}"))?;
    let mut constants = None;
    unsafe {
      device.CreateBuffer(
        &D3D11_BUFFER_DESC {
          ByteWidth: size_of::<RedactRecord>() as u32,
          Usage: D3D11_USAGE_DEFAULT,
          BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
          ..Default::default()
        },
        None,
        Some(&mut constants),
      )
    }
    .map_err(|error| error.to_string())?;
    Ok(Self {
      vertex_shader: vertex_shader.ok_or("D3D11 created no redaction vertex shader")?,
      cells_shader: cells_shader.ok_or("D3D11 created no redaction cells shader")?,
      paint_shader: paint_shader.ok_or("D3D11 created no redaction paint shader")?,
      constants: constants.ok_or("D3D11 created no redaction constant buffer")?,
      zones: StructuredBuffer::new(device, size_of::<[f32; 2]>(), "redaction zones")?,
      work: std::sync::Mutex::new(None),
      cells: std::sync::Mutex::new(None),
    })
  }

  /// The view the canvas samples `source` through: a redacted copy where
  /// `redactions` cover anything, else `None` and the source's own.
  /// `retained` says the source's pixels never change under it, which lets
  /// a still redrawn with the same redactions reuse the copy it left.
  pub(super) fn apply(
    &self,
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    source: &SourceTexture,
    redactions: &RedactRecords,
    retained: bool,
  ) -> Result<Option<ID3D11ShaderResourceView>, String> {
    if redactions.records.is_empty() {
      return Ok(None);
    }
    let mut description = D3D11_TEXTURE2D_DESC::default();
    unsafe { source.texture.GetDesc(&mut description) };
    let size = (description.Width, description.Height);
    let mut work = self
      .work
      .lock()
      .map_err(|_| "The redaction copies are poisoned".to_owned())?;
    if work
      .as_ref()
      .is_none_or(|work| work.size != size || work.format != description.Format)
    {
      *work = Some(Work {
        size,
        format: description.Format,
        copies: [
          target(device, size, description.Format)?,
          target(device, size, description.Format)?,
        ],
        kept: None,
      });
    }
    let work = work.as_mut().expect("the redaction copies were just made");
    if let Some((kept, records, index)) = &work.kept {
      if retained && *kept == source.texture && records == redactions {
        return Ok(Some(work.copies[*index].view.clone()));
      }
    }
    // Whatever is kept is about to be drawn over.
    work.kept = None;
    let zones = self.zones.write(device, context, &redactions.zones)?;
    let source_resource: ID3D11Resource =
      source.texture.cast().map_err(|error| error.to_string())?;
    let mut last = (source_resource, source.view.clone());
    let mut index = 0;
    for (pass, record) in redactions.records.iter().enumerate() {
      index = pass % 2;
      let copy = &work.copies[index];
      let copy_resource: ID3D11Resource = copy.texture.cast().map_err(|error| error.to_string())?;
      unsafe {
        context.CopyResource(&copy_resource, &last.0);
        context.UpdateSubresource(
          &self
            .constants
            .cast::<ID3D11Resource>()
            .map_err(|error| error.to_string())?,
          0,
          None,
          std::ptr::from_ref(record).cast::<c_void>(),
          0,
          0,
        );
      }
      let cells = self.average(device, context, record, &last.1)?;
      self.paint(context, record, copy, &last.1, &zones, cells.as_ref());
      last = (copy_resource, copy.view.clone());
    }
    if retained {
      work.kept = Some((source.texture.clone(), redactions.clone(), index));
    }
    Ok(Some(last.1))
  }

  /// Averages `record`'s cells from `from` into the cells target, whose view
  /// it answers; `None` for a box that averages none.
  fn average(
    &self,
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    record: &RedactRecord,
    from: &ID3D11ShaderResourceView,
  ) -> Result<Option<ID3D11ShaderResourceView>, String> {
    if !matches!(record.mode, REDACT_BLUR | REDACT_MOSAIC) {
      return Ok(None);
    }
    let grid = (record.grid[0], record.grid[1]);
    let mut cells = self
      .cells
      .lock()
      .map_err(|_| "The redaction cells are poisoned".to_owned())?;
    if cells
      .as_ref()
      .is_none_or(|(size, _)| size.0 < grid.0 || size.1 < grid.1)
    {
      let size = cells
        .as_ref()
        .map_or(grid, |(size, _)| (size.0.max(grid.0), size.1.max(grid.1)));
      *cells = Some((size, target(device, size, DXGI_FORMAT_R8G8B8A8_UNORM)?));
    }
    let (_, cells) = cells.as_ref().expect("the redaction cells were just made");
    self.draw(
      context,
      &cells.target,
      [0.0, 0.0, grid.0 as f32, grid.1 as f32],
      &self.cells_shader,
      [Some(from.clone()), None, None],
    );
    Ok(Some(cells.view.clone()))
  }

  /// Paints `record`'s box over `copy`, reading what the last pass left
  /// from `from`.
  fn paint(
    &self,
    context: &ID3D11DeviceContext,
    record: &RedactRecord,
    copy: &Target,
    from: &ID3D11ShaderResourceView,
    zones: &ID3D11ShaderResourceView,
    cells: Option<&ID3D11ShaderResourceView>,
  ) {
    let [x0, y0, x1, y1] = record.bounds;
    self.draw(
      context,
      &copy.target,
      [x0 as f32, y0 as f32, (x1 - x0) as f32, (y1 - y0) as f32],
      &self.paint_shader,
      [Some(from.clone()), Some(zones.clone()), cells.cloned()],
    );
  }

  /// One triangle through `shader` into `target`, over the viewport
  /// `[x, y, width, height]`, then everything it bound let go again so the
  /// next pass can copy into or read from what this one wrote.
  fn draw(
    &self,
    context: &ID3D11DeviceContext,
    target: &ID3D11RenderTargetView,
    viewport: [f32; 4],
    shader: &ID3D11PixelShader,
    views: [Option<ID3D11ShaderResourceView>; 3],
  ) {
    unsafe {
      context.OMSetRenderTargets(Some(&[Some(target.clone())]), None);
      context.OMSetBlendState(None::<&ID3D11BlendState>, None, u32::MAX);
      context.RSSetViewports(Some(&[D3D11_VIEWPORT {
        TopLeftX: viewport[0],
        TopLeftY: viewport[1],
        Width: viewport[2],
        Height: viewport[3],
        MinDepth: 0.0,
        MaxDepth: 1.0,
      }]));
      context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      context.VSSetShader(&self.vertex_shader, None);
      context.PSSetShader(shader, None);
      context.PSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
      context.PSSetShaderResources(0, Some(&views));
      context.Draw(3, 0);
      context.PSSetShaderResources(0, Some(&[None, None, None]));
      context.OMSetRenderTargets(None, None);
    }
  }
}
