// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GDI rasterisation and D3D11 upload of the counters' numbers.
//!
//! The twin of `gpu_compositor_macos_annotation_text.m`: a counter's number
//! is type, so it is drawn by the text engine rather than approximated by the
//! shader. Every counter in one composition is rasterised at the size it is
//! actually drawn and stacked into one texture, which the annotation shader
//! samples the way it samples the keyboard's artwork.
//!
//! The atlas is rasterised at [`SUPERSAMPLE`] pixels to the drawn pixel and
//! read with four taps, so a counter still reads while it is growing into
//! place rather than crawling with aliasing over its arrival.

use std::{collections::HashMap, sync::Mutex};

use crate::editor::annotations::AnnotationKind;

use windows::{
  core::{Interface, PCWSTR},
  Win32::{
    Foundation::COLORREF,
    Graphics::{
      Direct3D11::{
        ID3D11Device, ID3D11Resource, ID3D11ShaderResourceView, ID3D11Texture2D,
        D3D11_BIND_SHADER_RESOURCE, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_IMMUTABLE,
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

use crate::editor::annotations::MAX_ANNOTATIONS;

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
/// How many atlases are kept before the lot is dropped, as the keyboard
/// artwork's cache does: a counter being dragged or animated walks through
/// its own sizes rather than settling on one.
const CACHE_ENTRIES: usize = 64;

/// Where one counter's number sits in the atlas, in atlas pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct CounterTextRect {
  pub(super) x: f32,
  pub(super) y: f32,
  pub(super) width: f32,
  pub(super) height: f32,
}

/// One rasterised atlas: the texture the shader samples, its size, and where
/// each counter's number landed in it.
pub(crate) struct CounterArtwork {
  _texture: ID3D11Texture2D,
  pub(super) rects: Vec<CounterTextRect>,
  pub(crate) size: (u32, u32),
  pub(crate) view: ID3D11ShaderResourceView,
}

/// Shared by the editor's compositor and by the live overlay, which draws the
/// desktop's counters through the same pipeline.
#[derive(Default)]
pub(crate) struct CounterArtworkCache {
  entries: Mutex<HashMap<String, std::sync::Arc<CounterArtwork>>>,
}

impl CounterArtworkCache {
  /// The atlas for `counters` - one `(value, radius in drawn pixels)` per
  /// annotation, a zero value being an annotation that is not a counter - or
  /// nothing when there is no number to draw at all.
  ///
  /// The result is cached against exactly the numbers and sizes it was drawn
  /// from, so a preview that redraws an unchanged list does no work.
  pub(super) fn resolve(
    &self,
    device: &ID3D11Device,
    counters: &[(u32, f32)],
  ) -> Result<Option<std::sync::Arc<CounterArtwork>>, String> {
    let wanted: Vec<(usize, u32, f64)> = counters
      .iter()
      .take(MAX_ANNOTATIONS)
      .enumerate()
      .filter(|(_, (value, radius))| *value > 0 && *radius > 1.0)
      .map(|(index, (value, radius))| {
        // Quantised to a quarter pixel, so a preview nudged by rounding
        // reuses the atlas it already has.
        (index, *value, f64::from((radius * 4.0).round() / 4.0))
      })
      .collect();
    if wanted.is_empty() {
      return Ok(None);
    }
    let key = wanted
      .iter()
      .map(|(_, value, radius)| format!("{value}@{radius:.2}"))
      .collect::<Vec<_>>()
      .join("|");
    let mut entries = self
      .entries
      .lock()
      .map_err(|_| "The counter artwork cache is poisoned".to_owned())?;
    if let Some(known) = entries.get(&key) {
      return Ok(Some(std::sync::Arc::clone(known)));
    }
    let raster = rasterize(&wanted, counters.len())?;
    let artwork = std::sync::Arc::new(upload(device, &raster)?);
    if entries.len() >= CACHE_ENTRIES {
      entries.clear();
    }
    entries.insert(key, std::sync::Arc::clone(&artwork));
    Ok(Some(artwork))
  }
}

/// The GDI rasterisation the atlas is built by.
#[path = "counter_artwork/rasterize.rs"]
mod rasterize;
use rasterize::{rasterize, CounterRaster};

fn upload(device: &ID3D11Device, raster: &CounterRaster) -> Result<CounterArtwork, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: raster.size.0,
    Height: raster.size.1,
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
  };
  let initial = D3D11_SUBRESOURCE_DATA {
    pSysMem: raster.pixels.as_ptr().cast(),
    SysMemPitch: raster.size.0 * 4,
    SysMemSlicePitch: 0,
  };
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, Some(&initial), Some(&mut texture)) }
    .map_err(|error| error.to_string())?;
  let texture = texture.ok_or_else(|| "D3D11 created no counter artwork texture".to_owned())?;
  let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
  let mut view = None;
  unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
    .map_err(|error| error.to_string())?;
  Ok(CounterArtwork {
    _texture: texture,
    rects: raster.rects.clone(),
    size: raster.size,
    view: view.ok_or_else(|| "D3D11 created no counter artwork view".to_owned())?,
  })
}

/// The atlas one composition needs, and its annotations with the number
/// rectangles written into the slots a counter reads them from.
///
/// Only a counter reads those slots as a rectangle: an arrow with a head at
/// both ends keeps its second head's triangle in them, so the patch is per
/// annotation rather than across the list.
pub(crate) fn numbered_arrows(
  cache: &CounterArtworkCache,
  device: &ID3D11Device,
  prepared: &super::compositor::PreparedArrows,
) -> Result<
  (
    Option<std::sync::Arc<CounterArtwork>>,
    Vec<super::compositor::PreviewArrow>,
  ),
  String,
> {
  let numbers = cache.resolve(device, &prepared.counters)?;
  let mut arrows = prepared.arrows.clone();
  if let Some(atlas) = numbers.as_ref() {
    for (arrow, rect) in arrows.iter_mut().zip(&atlas.rects) {
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
