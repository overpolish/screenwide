// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The platform text engine's rasterisation of the annotations' type,
//! counters' numbers and text boxes' lines, into an atlas texture a pipeline
//! keeps: Core Text on macOS and DirectWrite on Windows, behind
//! `type_device`. Type is drawn by the text engine rather than approximated
//! by the shader. Where each piece sits
//! is laid out by the shared
//! [`atlas`](crate::editor::annotations::counter::atlas) module; this draws
//! what the layout asks for into a texture that outlives the composition,
//! which the annotation shader samples the way it samples the keyboard's
//! artwork.
//!
//! The atlas is rasterised at a density that follows how large the canvas is
//! drawn - [`raster_scale`] - and each drawn pixel is read as four filtered
//! taps over the atlas pixels it spans, so type stays smooth on a high-DPI
//! display, zoomed in or out, and while it grows into place.

use std::sync::Mutex;

use super::{PreparedArrows, PreparedType, PreviewArrow};
use crate::editor::annotations::counter::atlas::{
  AtlasDraw, AtlasRect, CounterAtlas as AtlasLayout, CounterNumber,
};
use crate::editor::annotations::counter::atlas_scale::raster_scale;
use crate::editor::annotations::AnnotationKind;

/// How much of the disc's diameter a digit's cap height takes, and the widest
/// the number may be drawn, both as shares of the diameter.
const CAP_SHARE: f64 = 0.42;
const WIDTH_SHARE: f64 = 0.72;
/// Inter's cap height, in ems: what turns a wanted cap height into a size.
const CAP_HEIGHT: f64 = 0.727;

/// The atlas one composition samples: its view, the texture's size, and how
/// many atlas pixels it holds per canvas pixel.
pub(crate) struct AtlasBinding {
  pub(crate) size: (u32, u32),
  pub(crate) scale: f32,
  pub(crate) view: wgpu::TextureView,
}

#[derive(Default)]
struct State {
  layout: AtlasLayout,
  storage: Option<GpuAtlas>,
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
    gpu: &crate::gpu::Gpu,
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
    // A fresh layout gets a new texture rather than being drawn over;
    // otherwise the new cells land in space no earlier cell used.
    if placed.fresh
      || storage
        .as_ref()
        .is_none_or(|storage| storage.size() != placed.size)
    {
      *storage = Some(GpuAtlas::new(gpu, placed.size));
    }
    let Some(storage) = storage.as_ref() else {
      return Ok(None);
    };
    for draw in draws.iter() {
      let rect = rects[draw.index];
      let origin = (rect.x as u32, rect.y as u32);
      let size = (rect.width as u32, rect.height as u32);
      let pixels = draw_cell(&types[draw.index], draw.radius, drawn, size)?;
      storage.write(gpu, origin, size, &pixels);
    }
    Ok(Some(AtlasBinding {
      size: storage.size(),
      scale,
      view: storage.view.clone(),
    }))
  }
}

/// The atlas's texture.
mod gpu_atlas;
/// The DirectWrite rasterisation the atlas is drawn by: a counter's number,
/// and a text box's lines.
mod rasterize;
mod text_box;
use gpu_atlas::GpuAtlas;

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

/// The atlas one composition samples, if any type is drawn, and its
/// annotations with their type rectangles in place.
pub(crate) type NumberedArrows = (Option<AtlasBinding>, Vec<PreviewArrow>);

/// The atlas one composition needs, and its annotations with the type
/// rectangles written into the slots a counter or a text box reads them
/// from.
///
/// Only those two read the slots as a rectangle: an arrow with a head at both
/// ends keeps its second head's triangle in them, so the patch is per
/// annotation rather than across the list.
pub(crate) fn numbered_arrows(
  atlas: &CounterAtlas,
  gpu: &crate::gpu::Gpu,
  prepared: &PreparedArrows,
) -> Result<NumberedArrows, String> {
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
  let numbers = atlas.resolve(gpu, &prepared.types, scale, drawn, &mut rects)?;
  let mut arrows = prepared.arrows.clone();
  if numbers.is_some() {
    for (arrow, rect) in arrows.iter_mut().zip(&rects) {
      match AnnotationKind::from_raw(arrow.kind) {
        Some(AnnotationKind::Counter | AnnotationKind::Text) => {}
        Some(
          AnnotationKind::Arrow
          | AnnotationKind::Redact
          | AnnotationKind::Highlight
          | AnnotationKind::Shape
          | AnnotationKind::Spotlight
          | AnnotationKind::Draw
          | AnnotationKind::Magnify,
        )
        | None => continue,
      }
      arrow.geometry.start_head[0] = rect.x;
      arrow.geometry.start_head[1] = rect.y;
      arrow.geometry.start_head[2] = rect.width;
      arrow.geometry.start_head[3] = rect.height;
    }
  }
  Ok((numbers, arrows))
}
