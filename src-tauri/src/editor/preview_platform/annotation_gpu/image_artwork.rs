// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The images' pictures, drawn into an atlas texture the compositor keeps
//! and the annotation shader samples in colour.
//!
//! Each picture is drawn at about the size it is shown at: its longer side
//! rounded up to a power of two of drawn pixels, so a drag that resizes it
//! draws it afresh only at each doubling, and the shader's filtered read
//! takes in the one or two atlas pixels a drawn pixel spans. An image that
//! casts a shadow has a second cell: its silhouette, blurred, drawn small
//! whatever its size. Where each sits is laid out by the shared
//! [`atlas`](crate::editor::annotations::counter::atlas) module, as the type
//! atlas's cells are, keyed by asset, size and which of the two it is.
//!
//! A picture that moves keeps one cell for as long as it is shown, and that
//! cell is drawn over each time its frame changes. Keying cells by frame
//! instead would fill the atlas with frames already past, and laying it out
//! again draws every image afresh: during export, a long animation did so
//! every few frames. Its frames are drawn no larger than they are held, as
//! one is drawn at the pace of the animation rather than once.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::gpu_atlas::GpuAtlas;
use super::{PreparedArrows, PreviewArrow};
use crate::editor::annotations::counter::atlas::{
  AtlasDraw, AtlasRect, CounterAtlas as AtlasCells, CounterNumber,
};
use crate::editor::annotations::flags::SHADOW;
use crate::editor::images::{rasterize, rasterize_shadow, shadow_margin, StoredImage, SHADOW_SIDE};

/// The longest side a picture is drawn at, in atlas pixels: past it a
/// zoomed-in view magnifies the raster instead, and a handful of images
/// still fits the texture side every device accepts.
const MAX_RASTER: u32 = 2_048;
/// The shortest, so an image shown tiny still has pixels to filter.
const MIN_RASTER: u32 = 16;

#[derive(Default)]
struct State {
  cells: AtlasCells,
  storage: Option<GpuAtlas>,
  draws: Vec<AtlasDraw>,
  /// The frame each moving picture's cell holds, by where the cell sits.
  shown: HashMap<(u32, u32), u32>,
}

/// The editor compositor's image atlas.
#[derive(Default)]
pub(crate) struct ImageAtlas {
  state: Mutex<State>,
}

/// The longer side a picture `long` canvas pixels long is drawn at, where one
/// drawn pixel covers `pixel_scale` canvas pixels.
fn raster_side(long: f32, pixel_scale: f32) -> u32 {
  let scale = if pixel_scale.is_finite() && pixel_scale > 0.0 {
    pixel_scale
  } else {
    1.0
  };
  let drawn = (long / scale).ceil();
  if !drawn.is_finite() || drawn <= 0.0 {
    return 0;
  }
  (drawn as u32)
    .next_power_of_two()
    .clamp(MIN_RASTER, MAX_RASTER)
}

/// The cell a picture of `size` canvas pixels takes with its longer side
/// `side` atlas pixels long, a pixel of margin all round included.
fn cell_of(size: [f32; 2], side: u32) -> Option<(u32, u32)> {
  let (width, height) = (size[0], size[1]);
  if !(width > 0.0 && height > 0.0) {
    return None;
  }
  let side = side as f32;
  let (cell_width, cell_height) = if width >= height {
    (side, (side * height / width).round().max(1.0))
  } else {
    ((side * width / height).round().max(1.0), side)
  };
  Some((cell_width as u32 + 2, cell_height as u32 + 2))
}

/// One cell a composition asks for: the picture of the image at `image`
/// in the prepared list, or its shadow.
#[derive(Clone, Copy)]
struct Entry {
  image: usize,
  shadow: bool,
}

/// The atlas this composition's images are drawn from, if it shows any, and
/// each image in `arrows` with where its picture is written into the slots
/// it reads it from: `start_head` the picture's cell, and for one that casts
/// a shadow, `end_head` its shadow's cell and how far in from its edge the
/// picture's own box starts. An image whose picture cannot be found keeps a
/// zero rectangle, which the shader draws as a placeholder.
pub(crate) fn place_images(
  atlas: &ImageAtlas,
  gpu: &crate::gpu::Gpu,
  prepared: &PreparedArrows,
  arrows: &mut [PreviewArrow],
) -> Result<Option<wgpu::TextureView>, String> {
  if prepared.images.is_empty() {
    return Ok(None);
  }
  let mut state = atlas
    .state
    .lock()
    .map_err(|_| "The image atlas is poisoned".to_owned())?;
  let State {
    cells,
    storage,
    draws,
    shown,
  } = &mut *state;
  // A moving picture is drawn at the frame this moment shows.
  let pictures: Vec<Option<(StoredImage, Option<u32>)>> = prepared
    .images
    .iter()
    .map(|image| {
      Some(match crate::editor::images::picture(&image.asset)? {
        StoredImage::Animation(animation) => {
          let index = animation.frame_index(image.frame, image.clock_ms, image.once);
          let frame = Arc::clone(animation.frames.get(index)?);
          (StoredImage::Still(frame), Some(index as u32))
        }
        still => (still, None),
      })
    })
    .collect();
  let entries: Vec<Entry> = prepared
    .images
    .iter()
    .enumerate()
    .flat_map(|(image, prepared_image)| {
      let shadowed = arrows
        .get(prepared_image.index)
        .is_some_and(|arrow| arrow.flags & SHADOW != 0);
      [Entry {
        image,
        shadow: false,
      }]
      .into_iter()
      .chain(shadowed.then_some(Entry {
        image,
        shadow: true,
      }))
    })
    .collect();
  let numbers: Vec<CounterNumber<'_>> = entries
    .iter()
    .map(|entry| {
      let image = &prepared.images[entry.image];
      let picture = pictures[entry.image].as_ref();
      // Two images showing one moving picture may be at different frames,
      // so each has a cell of its own, told apart by which of them it is.
      let slot = match picture {
        Some((_, Some(_))) => prepared.images[..entry.image]
          .iter()
          .filter(|earlier| earlier.asset == image.asset)
          .count() as u32,
        _ => 0,
      };
      let side = raster_side(image.size[0].max(image.size[1]), prepared.pixel_scale);
      CounterNumber {
        text: image.asset.as_bytes(),
        radius: match picture {
          _ if entry.shadow => SHADOW_SIDE,
          Some((StoredImage::Still(frame), Some(_))) => side.min(
            frame
              .width()
              .max(frame.height())
              .next_power_of_two()
              .max(MIN_RASTER),
          ),
          _ => side,
        } as f32,
        style: slot << 1 | u32::from(entry.shadow),
      }
    })
    .collect();
  // A shadow's cell is its silhouette's, grown by the blur's reach.
  let shadow_cell = |image: usize| {
    cell_of(prepared.images[image].size, SHADOW_SIDE).map(|cell| {
      let margin = shadow_margin(cell);
      (cell, margin)
    })
  };
  let mut rects = vec![AtlasRect::default(); numbers.len()];
  let placed = cells.frame(
    &numbers,
    |index, side| {
      let entry = entries[index];
      pictures[entry.image].as_ref()?;
      if entry.shadow {
        let (cell, margin) = shadow_cell(entry.image)?;
        Some((cell.0 + 2 * margin, cell.1 + 2 * margin))
      } else {
        cell_of(prepared.images[entry.image].size, side as u32)
      }
    },
    &mut rects,
    draws,
  );
  if rects.iter().all(|rect| rect.width == 0.0) {
    return Ok(None);
  }
  if placed.fresh
    || storage
      .as_ref()
      .is_none_or(|storage| storage.size() != placed.size)
  {
    *storage = Some(GpuAtlas::new(gpu, "Screenwide image atlas", placed.size));
    shown.clear();
  }
  // A moving picture's cell that holds another frame than this moment's is
  // drawn over, as well as the cells that are new.
  for (index, entry) in entries.iter().enumerate() {
    let (Some((_, Some(frame))), rect) = (&pictures[entry.image], rects[index]) else {
      continue;
    };
    let fresh_frame =
      rect.width > 0.0 && shown.insert((rect.x as u32, rect.y as u32), *frame) != Some(*frame);
    if fresh_frame && !draws.iter().any(|draw| draw.index == index) {
      draws.push(AtlasDraw {
        index,
        radius: numbers[index].radius,
      });
    }
  }
  let Some(storage) = storage.as_ref() else {
    return Ok(None);
  };
  for draw in draws.iter() {
    let (rect, entry) = (rects[draw.index], entries[draw.index]);
    let Some((picture, _)) = pictures[entry.image].as_ref() else {
      continue;
    };
    let pixels = if entry.shadow {
      shadow_cell(entry.image).and_then(|(cell, _)| rasterize_shadow(picture, cell))
    } else {
      rasterize(picture, (rect.width as u32, rect.height as u32))
    };
    if let Some(pixels) = pixels {
      let size = (rect.width as u32, rect.height as u32);
      storage.write(gpu, (rect.x as u32, rect.y as u32), size, &pixels);
    }
  }
  for (entry, rect) in entries.iter().zip(&rects) {
    let Some(arrow) = arrows.get_mut(prepared.images[entry.image].index) else {
      continue;
    };
    let cell = [rect.x, rect.y, rect.width, rect.height];
    if entry.shadow {
      let inset = shadow_cell(entry.image).map_or(0, |(_, margin)| margin + 1);
      arrow.geometry.end_head[..4].copy_from_slice(&cell);
      arrow.geometry.end_head[4] = inset as f32;
    } else {
      arrow.geometry.start_head[..4].copy_from_slice(&cell);
      // The preparation leaves the turning grip here, which only the chrome
      // reads; a shadow's cell, which follows its picture's, replaces it.
      arrow.geometry.end_head = [0.0; 6];
    }
  }
  Ok(Some(storage.view.clone()))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_picture_is_drawn_at_the_doubling_above_its_drawn_size() {
    assert_eq!(raster_side(100.0, 1.0), 128);
    // A Retina display at the canvas's own size draws it twice as large.
    assert_eq!(raster_side(100.0, 0.5), 256);
    assert_eq!(raster_side(2.0, 1.0), MIN_RASTER);
    assert_eq!(raster_side(100_000.0, 1.0), MAX_RASTER);
    assert_eq!(raster_side(0.0, 1.0), 0);
  }

  #[test]
  fn a_cell_keeps_the_pictures_proportions_inside_its_margin() {
    assert_eq!(cell_of([200.0, 100.0], 128), Some((130, 66)));
    assert_eq!(cell_of([50.0, 100.0], 64), Some((34, 66)));
    assert_eq!(cell_of([0.0, 100.0], 64), None);
  }
}
