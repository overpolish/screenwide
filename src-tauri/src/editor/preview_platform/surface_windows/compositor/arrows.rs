// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The structured-buffer elements the arrow shader reads, and the buffers
//! that carry them. Layouts here are pinned against `annotations.hlsl` by
//! the asserts beside each struct, so a field added on one side fails the
//! build rather than drifting silently.

use super::*;
/// One prepared arrow's geometry as scalars, shared by the mark and by each
/// of its exposure samples. The twin of the geometry block in `annotations.hlsl`.
///
/// Every member is a scalar on purpose. HLSL refuses to straddle a vector
/// across a 16-byte boundary and pads to avoid it, so a `[f32; 2]` here would
/// silently disagree with the shader; scalars pack at four bytes with no such
/// rule.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct PreviewGeometry {
  pub(crate) ax: f32,
  pub(crate) ay: f32,
  pub(crate) bx: f32,
  pub(crate) by: f32,
  pub(crate) cx: f32,
  pub(crate) cy: f32,
  pub(crate) width: f32,
  pub(crate) low: f32,
  pub(crate) high: f32,
  pub(crate) start_head: [f32; 6],
  pub(crate) end_head: [f32; 6],
  pub(crate) rounding: f32,
  pub(crate) head: u32,
}

const _: () = assert!(std::mem::size_of::<PreviewGeometry>() == 92);

impl PreviewGeometry {
  pub(crate) fn new(geometry: ArrowGeometry) -> Self {
    let triangle = |triangle: ArrowTriangle| {
      [
        triangle.a[0],
        triangle.a[1],
        triangle.b[0],
        triangle.b[1],
        triangle.c[0],
        triangle.c[1],
      ]
    };
    Self {
      ax: geometry.a[0],
      ay: geometry.a[1],
      bx: geometry.b[0],
      by: geometry.b[1],
      cx: geometry.c[0],
      cy: geometry.c[1],
      width: geometry.width,
      low: geometry.low,
      high: geometry.high,
      start_head: triangle(geometry.start_head),
      end_head: triangle(geometry.end_head),
      rounding: geometry.rounding,
      head: geometry.head,
    }
  }
}

/// One prepared mark as the pixel shader's structured buffer element.
///
/// Every member is a scalar on purpose. HLSL refuses to straddle a vector
/// across a 16-byte boundary and pads to avoid it, so a `[f32; 2]` here would
/// silently disagree with the shader; scalars pack at four bytes with no such
/// rule. The field order is the twin of `PreviewArrow` in `annotations.hlsl`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct PreviewArrow {
  pub(crate) geometry: PreviewGeometry,
  pub(crate) color: [f32; 4],
  /// The hover halo's width in canvas pixels; zero when nothing is hovered.
  pub(crate) hover: f32,
  /// This mark's run in the exposure sample buffer. A count of zero draws
  /// the prepared geometry directly, as a still always does.
  pub(crate) sample_first: u32,
  pub(crate) sample_count: u32,
}

const _: () = assert!(std::mem::size_of::<PreviewArrow>() == 120);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, color) == 92);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, hover) == 108);
const _: () = assert!(std::mem::offset_of!(PreviewArrow, sample_first) == 112);

impl PreviewArrow {
  /// Packs geometry prepared by `annotations::geometry::prepare_arrow`.
  pub(crate) fn new(geometry: ArrowGeometry, color: [f32; 4], hover: f32) -> Self {
    Self {
      geometry: PreviewGeometry::new(geometry),
      color,
      hover,
      sample_first: 0,
      sample_count: 0,
    }
  }
}

/// One exposure sample: the mark part way through the interval this frame
/// covers, and how solid it was then. The twin of `ScreenwideAnnotationSample`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct PreviewSample {
  pub(crate) geometry: PreviewGeometry,
  pub(crate) opacity: f32,
}

const _: () = assert!(std::mem::size_of::<PreviewSample>() == 96);

impl PreviewSample {
  pub(crate) fn new(geometry: ArrowGeometry, opacity: f32) -> Self {
    Self {
      geometry: PreviewGeometry::new(geometry),
      opacity,
    }
  }
}

/// The most exposure samples one mark prepares, matching the Metal cap.
pub(crate) const MAX_EXPOSURE_SAMPLES: usize = 48;

/// Everything one composition draws for its marks.
#[derive(Default)]
pub(crate) struct PreparedArrows {
  pub(crate) arrows: Vec<PreviewArrow>,
  pub(crate) samples: Vec<PreviewSample>,
  /// Where the above-camera run starts, which is what the shader's two
  /// passes are bounded by.
  pub(crate) below_camera: u32,
  /// How many canvas pixels one drawn pixel covers. Every edge is feathered
  /// over this rather than over one canvas pixel: a canvas shown smaller
  /// than its resolution would otherwise take its whole antialiasing band
  /// from inside a single drawn pixel and come out jagged. Zero means one.
  pub(crate) pixel_scale: f32,
}

/// A CPU-written structured buffer of `count` elements of `stride` bytes,
/// with the view the pixel shader reads it through.
pub(crate) fn structured_buffer(
  device: &ID3D11Device,
  stride: usize,
  count: usize,
  name: &str,
) -> Result<(ID3D11Buffer, ID3D11ShaderResourceView), String> {
  let mut buffer = None;
  unsafe {
    device.CreateBuffer(
      &D3D11_BUFFER_DESC {
        ByteWidth: (stride * count) as u32,
        Usage: D3D11_USAGE_DYNAMIC,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
        MiscFlags: D3D11_RESOURCE_MISC_BUFFER_STRUCTURED.0 as u32,
        StructureByteStride: stride as u32,
      },
      None,
      Some(&mut buffer),
    )
  }
  .map_err(|error| error.to_string())?;
  let buffer = buffer.ok_or_else(|| format!("D3D11 created no {name} buffer"))?;
  let mut view = None;
  unsafe {
    device.CreateShaderResourceView(
      &buffer,
      Some(&D3D11_SHADER_RESOURCE_VIEW_DESC {
        Format: DXGI_FORMAT_UNKNOWN,
        ViewDimension: D3D_SRV_DIMENSION_BUFFER,
        Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
          Buffer: D3D11_BUFFER_SRV {
            Anonymous1: D3D11_BUFFER_SRV_0 { FirstElement: 0 },
            Anonymous2: D3D11_BUFFER_SRV_1 {
              NumElements: count as u32,
            },
          },
        },
      }),
      Some(&mut view),
    )
  }
  .map_err(|error| error.to_string())?;
  let view = view.ok_or_else(|| format!("D3D11 created no {name} buffer view"))?;
  Ok((buffer, view))
}
