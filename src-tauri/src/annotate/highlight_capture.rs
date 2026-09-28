// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading the desktop at a highlight's press, and keeping what each
//! highlight still on screen was drawn over.

use std::sync::Arc;

use super::{state, underlay, Capture, Kept, Pending, State, Underlay, LANDED};
use crate::editor::annotations::highlight::model::{fresh_seed, HighlightBand};
use crate::editor::annotations::highlight::picture::HighlightPicture;
use crate::editor::annotations::AnnotationPoint;

/// A highlight stroke pressed at `point`: captures the display under it. A
/// capture that fails - Screen Recording not allowed, the display gone - leaves
/// the stroke undrawn, which is all the overlay can honestly show.
pub(in crate::annotate) fn begin(app: &tauri::AppHandle, stroke: &str, point: AnnotationPoint) {
  let Some((display, found)) = super::super::native_overlay::display_at(point.x, point.y) else {
    return;
  };
  state().pending = Some(Pending {
    stroke: stroke.to_owned(),
    seed: fresh_seed(),
    capture: None,
    drawn: None,
  });
  let app = app.clone();
  let stroke = stroke.to_owned();
  tauri::async_runtime::spawn(async move {
    let image = match crate::windows::monitor_capture::capture_monitor_screenshot(
      app.clone(),
      found.id,
    )
    .await
    {
      Ok(image) => Arc::new(image),
      Err(error) => {
        eprintln!("Could not read the desktop under a highlight: {error}");
        return;
      }
    };
    let scale = (
      f64::from(image.width) / found.size.0.max(1.0),
      f64::from(image.height) / found.size.1.max(1.0),
    );
    let Some(picture) = HighlightPicture::scaled(Arc::clone(&image), scale) else {
      return;
    };
    let capture = Arc::new(Capture {
      display,
      origin: found.origin,
      scale,
      picture: Arc::new(picture),
      image,
    });
    {
      let mut state = state();
      match &mut state.pending {
        Some(pending) if pending.stroke == stroke => {
          pending.capture = Some(Arc::clone(&capture));
          pending.drawn = None;
        }
        _ => return,
      }
      rebuild(&mut state, display, Some(&capture));
    }
    LANDED.notify_all();
    super::super::native_overlay::request_redraw(&app);
  });
}

/// The part of `capture` under `bands`, with a margin for a hand-drawn
/// stroke's reach past them.
pub(super) fn keep(id: &str, capture: &Capture, bands: &[HighlightBand]) -> Option<Kept> {
  let first = bands.first()?;
  let joined = bands.iter().fold(*first, |joined, band| HighlightBand {
    left: joined.left.min(band.left),
    top: joined.top.min(band.top),
    right: joined.right.max(band.right),
    bottom: joined.bottom.max(band.bottom),
  });
  let margin = (joined.bottom - joined.top).max(8.0);
  let (left, top) = capture.to_pixels(AnnotationPoint {
    x: joined.left - margin,
    y: joined.top - margin,
  });
  let (right, bottom) = capture.to_pixels(AnnotationPoint {
    x: joined.right + margin,
    y: joined.bottom + margin,
  });
  let image = &capture.image;
  let clamp_x = |value: f64| value.clamp(0.0, f64::from(image.width)) as u32;
  let clamp_y = |value: f64| value.clamp(0.0, f64::from(image.height)) as u32;
  let (left, top, right, bottom) = (clamp_x(left), clamp_y(top), clamp_x(right), clamp_y(bottom));
  let crop = underlay::crop(image, left, top, right, bottom)?;
  Some(Kept {
    id: id.to_owned(),
    display: capture.display,
    left,
    top,
    crop,
  })
}

/// Makes `display`'s underlay again from `latest` and every highlight still on
/// screen there. A highlight that has since been taken off is forgotten.
fn rebuild(state: &mut State, display: usize, latest: Option<&Capture>) {
  let showing: Vec<String> = super::super::live_clips::annotations()
    .iter()
    .map(|annotation| annotation.id.clone())
    .collect();
  state.kept.retain(|kept| showing.contains(&kept.id));
  let Some(latest) = latest else {
    return;
  };
  let pieces: Vec<_> = state
    .kept
    .iter()
    .filter(|kept| kept.display == display)
    .map(|kept| (kept.left, kept.top, &kept.crop))
    .collect();
  let underlay = Underlay::compose(&latest.image, &pieces);
  if state.underlays.len() <= display {
    state.underlays.resize(display + 1, None);
  }
  state.underlays[display] = Some(Arc::new(underlay));
}
