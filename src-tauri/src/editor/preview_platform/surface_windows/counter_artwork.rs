// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! DirectWrite rasterisation and D3D11 upload of the annotations' type:
//! counters' numbers and text boxes' lines.
//!
//! The twin of `gpu_compositor_macos_annotation_text.m`: type is drawn by the
//! text engine rather than approximated by the shader. Where each piece sits
//! is laid out by the shared
//! [`atlas`](crate::editor::annotations::counter::atlas) module; this draws
//! what the layout asks for into a texture that outlives the composition,
//! which the annotation shader samples the way it samples the keyboard's
//! artwork.
//!
//! The atlas is rasterised at a density that follows how large the canvas is
//! drawn - [`raster_scale`], shared with the Metal backend - and each drawn
//! pixel is read as four filtered taps over the atlas pixels it spans, so type
//! stays smooth on a high-DPI display, zoomed in or out, and while it grows
//! into place.

use std::sync::Mutex;

use super::compositor::PreparedType;
use crate::editor::annotations::counter::atlas::{
  AtlasDraw, AtlasRect, CounterAtlas as AtlasLayout, CounterNumber,
};
use crate::editor::annotations::counter::atlas_scale::raster_scale;
use crate::editor::annotations::AnnotationKind;

use windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D11::{
      ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11ShaderResourceView, ID3D11Texture2D,
      D3D11_BIND_SHADER_RESOURCE, D3D11_BOX, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
    },
    Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
  },
};

/// How much of the disc's diameter a digit's cap height takes, and the widest
/// the number may be drawn, both as shares of the diameter. The twins of
/// `SCREENWIDE_COUNTER_TEXT_CAP_SHARE` and
/// `SCREENWIDE_COUNTER_TEXT_WIDTH_SHARE`.
const CAP_SHARE: f64 = 0.42;
const WIDTH_SHARE: f64 = 0.72;
/// Inter's cap height, in ems: what turns a wanted cap height into a size.
const CAP_HEIGHT: f64 = 0.727;

/// The atlas one composition samples: its texture, the texture's size, and
/// how many atlas pixels it holds per canvas pixel.
pub(crate) struct AtlasBinding {
  pub(crate) size: (u32, u32),
  pub(crate) scale: f32,
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
  /// The atlas for `types` - one per annotation, an empty text being an
  /// annotation with no type - rasterised at `scale` atlas pixels per canvas
  /// pixel and `drawn` atlas pixels per drawn pixel, with each one's
  /// rectangle written into `rects`, or nothing when no type is drawn at
  /// all. Only type the atlas does not hold yet is rasterised.
  fn resolve(
    &self,
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    types: &[PreparedType],
    scale: f32,
    drawn: f64,
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
    let keys: Vec<Vec<u8>> = types.iter().map(|entry| cell_key(entry, drawn)).collect();
    let numbers: Vec<CounterNumber<'_>> = types
      .iter()
      .zip(&keys)
      .map(|(entry, key)| CounterNumber {
        text: key,
        radius: entry.size * scale,
        style: entry.style,
      })
      .collect();
    rects.clear();
    rects.resize(types.len(), AtlasRect::default());
    let placed = layout.frame(
      &numbers,
      |index, size| measure_cell(&types[index], size, drawn),
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
      let pixels = draw_cell(&types[draw.index], draw.radius, drawn, (width, height))?;
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
      scale,
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

/// The DirectWrite rasterisation the atlas is drawn by: a counter's number,
/// and a text box's lines.
#[path = "counter_artwork/rasterize.rs"]
mod rasterize;
#[path = "counter_artwork/text_box.rs"]
mod text_box;

/// What the atlas keys a piece of type by: its text, and for a box being
/// typed into its caret, its selection and the caret's width in atlas pixels
/// too, so each change of them draws afresh. `0xFF` never appears in UTF-8,
/// so no text can collide with that tail.
fn cell_key(entry: &PreparedType, drawn: f64) -> Vec<u8> {
  let mut key = entry.text.as_bytes().to_vec();
  if let Some(marks) = entry.marks {
    key.push(0xFF);
    key.extend_from_slice(&(marks.start as u32).to_le_bytes());
    key.extend_from_slice(&(marks.end as u32).to_le_bytes());
    key.push(u8::from(marks.caret));
    key.extend_from_slice(&((drawn * 4.0).round() as u32).to_le_bytes());
  }
  key
}

/// `size` is in atlas pixels, and `drawn` is how many of them one drawn pixel
/// spans, which a text box's caret is as wide as.
fn measure_cell(entry: &PreparedType, size: f32, drawn: f64) -> Option<(u32, u32)> {
  if entry.style == 0 {
    return rasterize::measure(&entry.text, size).ok();
  }
  text_box::measure(&entry.text, size, drawn, entry.marks)
    .ok()
    .flatten()
}

fn draw_cell(
  entry: &PreparedType,
  size: f32,
  drawn: f64,
  cell: (u32, u32),
) -> Result<Vec<u8>, String> {
  if entry.style == 0 {
    return rasterize::draw(&entry.text, size, cell);
  }
  text_box::draw(&entry.text, size, drawn, entry.style - 1, entry.marks, cell)
}

/// The atlas one composition needs, and its annotations with the type
/// rectangles written into the slots a counter or a text box reads them
/// from.
///
/// Only those two read the slots as a rectangle: an arrow with a head at both
/// ends keeps its second head's triangle in them, so the patch is per
/// annotation rather than across the list.
pub(crate) fn numbered_arrows(
  atlas: &CounterAtlas,
  device: &ID3D11Device,
  context: &ID3D11DeviceContext,
  prepared: &super::compositor::PreparedArrows,
) -> Result<(Option<AtlasBinding>, Vec<super::compositor::PreviewArrow>), String> {
  let mut rects = Vec::new();
  let pixel_scale = if prepared.pixel_scale > 0.0 {
    prepared.pixel_scale
  } else {
    1.0
  };
  let largest = prepared
    .types
    .iter()
    .map(|entry| entry.size)
    .fold(0.0, f32::max);
  let scale = raster_scale(pixel_scale, largest);
  // Atlas pixels per drawn pixel: what a text box's caret is as wide as.
  let drawn = f64::from(scale * pixel_scale);
  let numbers = atlas.resolve(device, context, &prepared.types, scale, drawn, &mut rects)?;
  let mut arrows = prepared.arrows.clone();
  if numbers.is_some() {
    for (arrow, rect) in arrows.iter_mut().zip(&rects) {
      match AnnotationKind::from_raw(arrow.kind) {
        Some(AnnotationKind::Counter | AnnotationKind::Text) => {}
        Some(AnnotationKind::Arrow | AnnotationKind::Redact) | None => continue,
      }
      arrow.geometry.start_head[0] = rect.x;
      arrow.geometry.start_head[1] = rect.y;
      arrow.geometry.start_head[2] = rect.width;
      arrow.geometry.start_head[3] = rect.height;
    }
  }
  Ok((numbers, arrows))
}
