// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A screenshot's layers composed onto one canvas, a band at a time where
//! the canvas or a picture is larger than the device takes (`bands.rs`).

use super::super::bands::{self, Band, LayerNeeds};
use super::super::layer::CanvasBand;
use super::*;

/// One layer, ready to draw into any band: its picture, its settings, where
/// the picture sits on the canvas, and its annotations prepared once.
struct Layer<'a> {
  image: &'a CapturedImage,
  settings: &'a ScreenshotOutputSettings,
  geometry: CanvasGeometry,
  prepared: PreparedArrows,
  /// The whole picture, uploaded at the first band that reads it whole.
  whole: Option<SourceTexture>,
}

/// Screenshot items composed onto one canvas, the first with the canvas's
/// background and the rest laid over it, as the editor shows them. The
/// canvas keeps its alpha past its rounded corners.
pub fn compose_layers(
  layers: &[(&CapturedImage, ScreenshotOutputSettings)],
) -> Result<CapturedImage, String> {
  compose_layers_within(layers, None)
}

/// [`compose_layers`], drawn in bands no larger than `limit` on a side
/// where one is given, else no larger than the device takes.
pub(in super::super) fn compose_layers_within(
  layers: &[(&CapturedImage, ScreenshotOutputSettings)],
  limit: Option<u32>,
) -> Result<CapturedImage, String> {
  let (_, first) = layers
    .first()
    .ok_or_else(|| "The screenshot workspace is empty".to_owned())?;
  let stills = stills()?;
  let compositor = stills
    .lock()
    .map_err(|_| "The still compositor is unavailable".to_owned())?;
  let gpu = compositor.gpu();
  let size = screenwide_output::output_dimensions(first)?;
  let mut layers = layers
    .iter()
    .map(|(image, settings)| {
      if screenwide_output::output_dimensions(settings)? != size {
        return Err("The screenshot layers do not share a canvas size".to_owned());
      }
      let source = (image.width, image.height);
      // The export never carries a halo, nor a caret.
      let prepared = crate::annotation_gpu::prepared_arrows(
        &settings.annotations,
        source,
        settings,
        Some(*image),
        None,
        None,
      )?;
      Ok(Layer {
        image,
        settings,
        geometry: CanvasGeometry::of(source, settings)?,
        prepared,
        whole: None,
      })
    })
    .collect::<Result<Vec<_>, String>>()?;
  let needs = layers
    .iter()
    .map(|layer| {
      LayerNeeds::of(
        (layer.image.width, layer.image.height),
        layer.settings,
        &layer.geometry,
        &layer.prepared,
      )
    })
    .collect::<Vec<_>>();
  let largest = gpu.device.limits().max_texture_dimension_2d;
  let limit = limit.map_or(largest, |limit| limit.min(largest));
  let mut rgba = canvas_pixels(size)?;
  for band in bands::plan(size, limit, &needs) {
    let target = canvas_target(gpu, band.target.size);
    draw_band(&compositor, &target, &band, size, &mut layers)?;
    keep_band(gpu, &target, &band, size, &mut rgba)?;
  }
  Ok(CapturedImage {
    width: size.0,
    height: size.1,
    rgba,
  })
}

/// Every layer drawn into `target`, which holds `band` of the canvas.
fn draw_band(
  compositor: &Compositor,
  target: &wgpu::Texture,
  band: &Band,
  canvas: (u32, u32),
  layers: &mut [Layer<'_>],
) -> Result<(), String> {
  let view = target.create_view(&Default::default());
  let whole_canvas = band.target
    == CanvasBand {
      origin: (0, 0),
      size: canvas,
    };
  for (index, (layer, window)) in layers.iter_mut().zip(&band.windows).enumerate() {
    let windowed;
    let source = match window {
      Some(window) => {
        windowed = compositor.screenshot_window(layer.image, *window, 1)?;
        &windowed
      }
      None => match &mut layer.whole {
        Some(whole) => &*whole,
        empty => &*empty.insert(compositor.screenshot_source(layer.image)?),
      },
    };
    compositor.draw_layer(
      &view,
      source,
      layer.settings,
      &layer.geometry,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: index > 0,
        seconds: 0.0,
      },
      None,
      None,
      &layer.prepared,
      LayerDraw {
        placement: None,
        band: (!whole_canvas).then_some(band.target),
        source_window: window.map(|[x0, y0, _, _]| SourceWindow {
          origin: (x0, y0),
          scale: 1,
        }),
        clear: index == 0,
        redaction_slot: 0,
        retained_source: true,
      },
    )?;
  }
  Ok(())
}

/// The canvas's RGBA pixels, zeroed, or an error where there is not the
/// memory to hold them.
fn canvas_pixels((width, height): (u32, u32)) -> Result<Vec<u8>, String> {
  let length = (width as usize)
    .checked_mul(height as usize)
    .and_then(|pixels| pixels.checked_mul(4))
    .ok_or_else(|| "The screenshot is too large to hold".to_owned())?;
  let mut rgba = Vec::new();
  rgba
    .try_reserve_exact(length)
    .map_err(|_| "There is not enough memory to hold the screenshot".to_owned())?;
  rgba.resize(length, 0);
  Ok(rgba)
}

/// The part of `band` it keeps, read back from `target` and written into the
/// canvas's `rgba`, its BGRA turned to RGBA.
fn keep_band(
  gpu: &Gpu,
  target: &wgpu::Texture,
  band: &Band,
  canvas: (u32, u32),
  rgba: &mut [u8],
) -> Result<(), String> {
  let bgra = gpu.read_texture(target)?;
  let [x0, y0, x1, y1] = band.kept;
  let (origin_x, origin_y) = band.target.origin;
  let target_width = band.target.size.0 as usize;
  let row_bytes = (x1 - x0) as usize * 4;
  for y in y0..y1 {
    let from = ((y - origin_y) as usize * target_width + (x0 - origin_x) as usize) * 4;
    let to = (y as usize * canvas.0 as usize + x0 as usize) * 4;
    let (from, to) = (&bgra[from..from + row_bytes], &mut rgba[to..to + row_bytes]);
    for (from, to) in from
      .as_chunks::<4>()
      .0
      .iter()
      .zip(to.as_chunks_mut::<4>().0)
    {
      *to = [from[2], from[1], from[0], from[3]];
    }
  }
  Ok(())
}
