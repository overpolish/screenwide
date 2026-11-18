// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A screenshot larger than the device takes, such as a long scrolling
//! capture, drawn from a window of it: the whole picture scaled down while
//! the view shows it no finer than that, and a sharper window of the part in
//! view once zoomed in, uploaded again only as the view leaves it.

use std::sync::{Arc, Mutex};

use super::picture::ScenePicture;
use screenwide_compositor::annotation_gpu::PreparedArrows;
use screenwide_compositor::compositor::{
  picture_needed, CanvasGeometry, Compositor, SourceTexture, SourceWindow,
};
use screenwide_imaging::CapturedImage;
use screenwide_output::ScreenshotOutputSettings;

/// A sharper window, the part of the picture it holds, x0, y0, x1, y1, and
/// its texture.
type Detail = (SourceWindow, [u32; 4], SourceTexture);

impl ScenePicture {
  /// The texture to draw this picture from, and what part of it that holds
  /// where it holds less than all of it at its own size, for a layer drawn
  /// with `settings`, `geometry` and `prepared` annotations at `rect`, x, y,
  /// width and height in a `target` drawable pixels in size.
  pub(super) fn drawn_from(
    &self,
    compositor: &Compositor,
    (settings, geometry, prepared): (&ScreenshotOutputSettings, &CanvasGeometry, &PreparedArrows),
    rect: [f32; 4],
    target: (u32, u32),
  ) -> Result<(SourceTexture, Option<SourceWindow>), String> {
    let Some(large) = &self.large else {
      return Ok((self.texture.clone(), None));
    };
    let [x, y, width, height] = rect.map(f64::from);
    // Canvas pixels per drawn pixel, and the part of the canvas the
    // drawable shows, past its edge too, where the picture is drawn faded.
    let across = f64::from(geometry.size.0) / width.max(1e-6);
    let down = f64::from(geometry.size.1) / height.max(1e-6);
    let part = [
      -x * across,
      -y * down,
      (f64::from(target.0) - x) * across,
      (f64::from(target.1) - y) * down,
    ];
    let source = self.texture.size;
    let needed = picture_needed(source, settings, geometry, prepared, part);
    let per_drawn = across * f64::from(source.0) / geometry.placement.image_width.max(1e-6);
    let (texture, window) = large.window(compositor, &self.texture, needed, per_drawn)?;
    Ok((texture, Some(window)))
  }
}

pub(super) struct LargePicture {
  image: Arc<CapturedImage>,
  /// How many of the picture's pixels a texel of the whole picture's
  /// texture spans: as few as the device takes.
  overview_scale: u32,
  detail: Mutex<Option<Detail>>,
}

impl LargePicture {
  /// `image` and its whole picture's texture, where it is larger than the
  /// device takes; `None` where it fits whole.
  pub(super) fn new(
    compositor: &Compositor,
    image: &Arc<CapturedImage>,
  ) -> Result<Option<(Self, SourceTexture)>, String> {
    let limit = compositor.gpu().device.limits().max_texture_dimension_2d;
    let longest = image.width.max(image.height);
    if longest <= limit {
      return Ok(None);
    }
    let overview_scale = longest.div_ceil(limit);
    let overview =
      compositor.screenshot_window(image, [0, 0, image.width, image.height], overview_scale)?;
    Ok(Some((
      Self {
        image: Arc::clone(image),
        overview_scale,
        detail: Mutex::new(None),
      },
      overview,
    )))
  }

  /// The texture to draw the picture from, and what part of it that holds,
  /// where the view needs the part `needed` of the picture, x0, y0, x1, y1,
  /// and `per_drawn` of its pixels fall on a drawn pixel. `overview` is the
  /// whole picture's texture.
  pub(super) fn window(
    &self,
    compositor: &Compositor,
    overview: &SourceTexture,
    needed: [u32; 4],
    per_drawn: f64,
  ) -> Result<(SourceTexture, SourceWindow), String> {
    let whole = (
      overview.clone(),
      SourceWindow {
        origin: (0, 0),
        scale: self.overview_scale,
      },
    );
    let limit = compositor.gpu().device.limits().max_texture_dimension_2d;
    // No finer than a drawn pixel, and no coarser than the device needs.
    let fine = if per_drawn.is_finite() {
      per_drawn.floor().clamp(1.0, f64::from(u32::MAX)) as u32
    } else {
      1
    };
    let longest = (needed[2] - needed[0]).max(needed[3] - needed[1]);
    let scale = fine.max(longest.div_ceil(limit));
    if scale >= self.overview_scale {
      return Ok(whole);
    }
    let mut detail = self
      .detail
      .lock()
      .map_err(|_| "The picture's window is poisoned".to_owned())?;
    if let Some((window, held, texture)) = detail.as_ref() {
      if window.scale == scale && contains(*held, needed) {
        return Ok((texture.clone(), *window));
      }
    }
    let held = grown(needed, limit * scale, (self.image.width, self.image.height));
    let texture = compositor.screenshot_window(&self.image, held, scale)?;
    let window = SourceWindow {
      origin: (held[0], held[1]),
      scale,
    };
    *detail = Some((window, held, texture.clone()));
    Ok((texture, window))
  }
}

fn contains(outer: [u32; 4], inner: [u32; 4]) -> bool {
  outer[0] <= inner[0] && outer[1] <= inner[1] && outer[2] >= inner[2] && outer[3] >= inner[3]
}

/// `needed` with as much again round it as `longest` and the picture's
/// `size` allow, so the view can pan a while before the window is uploaded
/// again.
fn grown(needed: [u32; 4], longest: u32, size: (u32, u32)) -> [u32; 4] {
  let axis = |low: u32, high: u32, length: u32| {
    let room = longest.min(length).max(high - low);
    let extra = (room - (high - low)).min(high - low);
    let start = low.saturating_sub(extra / 2);
    let end = (start + (high - low) + extra).min(length);
    (end.saturating_sub((high - low) + extra).min(start), end)
  };
  let (x0, x1) = axis(needed[0], needed[2], size.0);
  let (y0, y1) = axis(needed[1], needed[3], size.1);
  [x0, y0, x1, y1]
}

#[cfg(test)]
mod tests {
  use super::{contains, grown};

  #[test]
  fn a_window_grows_round_what_the_view_needs_within_the_device_and_the_picture() {
    // Twice what is needed, centred on it.
    assert_eq!(
      grown([0, 1000, 800, 2000], 8192, (1000, 50_000)),
      [0, 500, 1000, 2500]
    );
    // Held to what the device takes.
    let held = grown([0, 1000, 800, 8000], 8192, (1000, 50_000));
    assert!(held[3] - held[1] <= 8192 && contains(held, [0, 1000, 800, 8000]));
    // Kept inside the picture at its end.
    let end = grown([0, 49_000, 800, 50_000], 8192, (1000, 50_000));
    assert_eq!(end[3], 50_000);
    assert!(contains(end, [0, 49_000, 800, 50_000]) && end[3] - end[1] == 2000);
  }
}
