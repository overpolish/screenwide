// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GDI rasterisation and D3D11 upload of the counters' numbers.
//!
//! The twin of `gpu_compositor_macos_annotation_text.m`: a counter's number
//! is type, so it is drawn by the text engine rather than approximated by the
//! shader. Where each number sits is laid out by the shared
//! [`atlas`](crate::editor::annotations::counter::atlas) module; this draws the
//! numbers the layout asks for into a texture that outlives the composition,
//! which the annotation shader samples the way it samples the keyboard's
//! artwork.
//!
//! The atlas is rasterised at [`SUPERSAMPLE`] pixels to the drawn pixel and
//! read with four taps, so a counter still reads while it is growing into
//! place rather than crawling with aliasing over its arrival.

use std::sync::Mutex;

use crate::editor::annotations::counter::atlas::{
  AtlasDraw, AtlasRect, CounterAtlas as AtlasLayout, CounterNumber,
};
use crate::editor::annotations::AnnotationKind;

use windows::{
  core::{Interface, PCWSTR},
  Win32::{
    Foundation::COLORREF,
    Graphics::{
      Direct3D11::{
        ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11ShaderResourceView,
        ID3D11Texture2D, D3D11_BIND_SHADER_RESOURCE, D3D11_BOX, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_DEFAULT,
      },
      Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
      Gdi::{
        CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject,
        GetTextExtentPoint32W, SelectObject, SetBkMode, SetTextColor, TextOutW,
        ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS,
        DEFAULT_CHARSET, DIB_RGB_COLORS, FF_SWISS, FW_SEMIBOLD, HDC, HFONT, HGDIOBJ,
        OUT_DEFAULT_PRECIS, TRANSPARENT, VARIABLE_PITCH,
      },
    },
  },
};

/// How many atlas pixels are rasterised per drawn pixel.
const SUPERSAMPLE: f64 = 2.0;
/// How much of the disc's diameter a digit's cap height takes, and the widest
/// the number may be drawn, both as shares of the diameter. The twins of
/// `SCREENWIDE_COUNTER_TEXT_CAP_SHARE` and
/// `SCREENWIDE_COUNTER_TEXT_WIDTH_SHARE`.
const CAP_SHARE: f64 = 0.42;
const WIDTH_SHARE: f64 = 0.72;
/// Inter's cap height, in ems: what turns a wanted cap height into a size.
const CAP_HEIGHT: f64 = 0.727;

/// The atlas one composition samples: its texture and the texture's size.
pub(crate) struct AtlasBinding {
  pub(crate) size: (u32, u32),
  pub(crate) view: ID3D11ShaderResourceView,
}

struct Storage {
  size: (u32, u32),
  texture: ID3D11Texture2D,
  view: ID3D11ShaderResourceView,
}

#[derive(Default)]
struct State {
  layout: AtlasLayout,
  storage: Option<Storage>,
  draws: Vec<AtlasDraw>,
}

/// One atlas per pipeline: the editor's compositor keeps one and the live
/// overlay, which draws the desktop's counters through the same shader,
/// keeps its own.
#[derive(Default)]
pub(crate) struct CounterAtlas {
  state: Mutex<State>,
}

impl CounterAtlas {
  /// The atlas for `counters` - one `(value, radius in drawn pixels)` per
  /// annotation, an empty value being an annotation that is not a counter -
  /// with each one's rectangle written into `rects`, or nothing when no number
  /// is drawn at all. Only numbers the atlas does not hold yet are rasterised.
  fn resolve(
    &self,
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    counters: &[(String, f32)],
    rects: &mut Vec<AtlasRect>,
  ) -> Result<Option<AtlasBinding>, String> {
    let mut state = self
      .state
      .lock()
      .map_err(|_| "The counter atlas is poisoned".to_owned())?;
    let State {
      layout,
      storage,
      draws,
    } = &mut *state;
    let numbers: Vec<CounterNumber<'_>> = counters
      .iter()
      .map(|(value, radius)| CounterNumber {
        text: value.as_bytes(),
        radius: *radius,
      })
      .collect();
    rects.clear();
    rects.resize(counters.len(), AtlasRect::default());
    let placed = layout.frame(
      &numbers,
      |index, radius| rasterize::measure(&counters[index].0, radius).ok(),
      rects,
      draws,
    );
    if rects.iter().all(|rect| rect.width == 0.0) {
      return Ok(None);
    }
    // A fresh layout gets a new texture rather than being drawn over, the way
    // the Metal atlas gets a new buffer; otherwise the new cells land in space
    // no earlier cell used.
    if placed.fresh
      || storage
        .as_ref()
        .is_none_or(|storage| storage.size != placed.size)
    {
      *storage = Some(create_storage(device, placed.size)?);
    }
    let Some(storage) = storage.as_ref() else {
      return Ok(None);
    };
    let resource: ID3D11Resource = storage.texture.cast().map_err(|error| error.to_string())?;
    for draw in draws.iter() {
      let rect = rects[draw.index];
      let (left, top) = (rect.x as u32, rect.y as u32);
      let (width, height) = (rect.width as u32, rect.height as u32);
      let pixels = rasterize::draw(&counters[draw.index].0, draw.radius, (width, height))?;
      unsafe {
        context.UpdateSubresource(
          &resource,
          0,
          Some(&D3D11_BOX {
            left,
            top,
            front: 0,
            right: left + width,
            bottom: top + height,
            back: 1,
          }),
          pixels.as_ptr().cast(),
          width * 4,
          0,
        );
      }
    }
    Ok(Some(AtlasBinding {
      size: storage.size,
      view: storage.view.clone(),
    }))
  }
}

/// A texture for a fresh layout. Its contents start undefined, which is safe
/// because every cell is drawn whole, margin included, before it is sampled,
/// and the shader reads nowhere else.
fn create_storage(device: &ID3D11Device, size: (u32, u32)) -> Result<Storage, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: size.0,
    Height: size.1,
    MipLevels: 1,
    ArraySize: 1,
    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_DEFAULT,
    BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
    ..Default::default()
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "D3D11 created no counter atlas texture".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  Ok(Storage {
    size,
    texture,
    view: view.ok_or_else(|| "D3D11 created no counter atlas view".to_owned())?,
  })
}

/// The GDI rasterisation the atlas is drawn by.
#[path = "counter_artwork/rasterize.rs"]
mod rasterize;

/// The atlas one composition needs, and its annotations with the number
/// rectangles written into the slots a counter reads them from.
///
/// Only a counter reads those slots as a rectangle: an arrow with a head at
/// both ends keeps its second head's triangle in them, so the patch is per
/// annotation rather than across the list.
pub(crate) fn numbered_arrows(
  atlas: &CounterAtlas,
  device: &ID3D11Device,
  context: &ID3D11DeviceContext,
  prepared: &super::compositor::PreparedArrows,
) -> Result<(Option<AtlasBinding>, Vec<super::compositor::PreviewArrow>), String> {
  let mut rects = Vec::new();
  let numbers = atlas.resolve(device, context, &prepared.counters, &mut rects)?;
  let mut arrows = prepared.arrows.clone();
  if numbers.is_some() {
    for (arrow, rect) in arrows.iter_mut().zip(&rects) {
      match AnnotationKind::from_raw(arrow.kind) {
        Some(AnnotationKind::Counter) => {}
        Some(AnnotationKind::Arrow) | None => continue,
      }
      arrow.geometry.start_head[0] = rect.x;
      arrow.geometry.start_head[1] = rect.y;
      arrow.geometry.start_head[2] = rect.width;
      arrow.geometry.start_head[3] = rect.height;
    }
  }
  Ok((numbers, arrows))
}
