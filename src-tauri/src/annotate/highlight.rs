// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Highlights drawn live, over a desktop the overlay cannot see.
//!
//! The overlay's surfaces are laid over the desktop by the window server,
//! which never hands them what is underneath. A highlight needs those pixels
//! twice: to find the lines it covers, and to recolour them. So its press
//! captures the display it lands on with the overlay left out, the stroke
//! selects from that capture, and the highlight is drawn from it for as long
//! as it stays on screen. It does not follow the desktop afterwards: like
//! every live annotation it is a drawing on glass.
//!
//! What each display draws highlights over is its underlay: the latest
//! capture taken on it, with every highlight still on screen pasted back over
//! it from its own capture, so an earlier highlight keeps the pixels it was
//! drawn on.

use std::sync::{Arc, Condvar, LazyLock, Mutex, MutexGuard};
use std::time::Duration;

use crate::editor::annotations::highlight::detect::{box_tone, select, Selection};
use crate::editor::annotations::highlight::manual::strokes;
use crate::editor::annotations::highlight::model::{new_highlight, HighlightTone};
use crate::editor::annotations::highlight::picture::HighlightPicture;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape, AnnotationStyle};
use crate::screenshots::CapturedImage;

#[path = "highlight_capture.rs"]
mod capture;
#[path = "highlight_underlay.rs"]
mod underlay;
pub(super) use capture::{begin, refresh};
pub(super) use underlay::Underlay;

/// One display's pixels at a press, and where they sit on the desktop.
struct Capture {
  display: usize,
  origin: (f64, f64),
  /// Capture pixels per desktop point, across and down.
  scale: (f64, f64),
  picture: Arc<HighlightPicture>,
  image: Arc<CapturedImage>,
}

impl Capture {
  fn to_pixels(&self, point: AnnotationPoint) -> (f64, f64) {
    (
      (point.x - self.origin.0) * self.scale.0,
      (point.y - self.origin.1) * self.scale.1,
    )
  }
}

/// A highlight on screen, and the part of its capture it covers.
struct Kept {
  id: String,
  display: usize,
  left: u32,
  top: u32,
  crop: CapturedImage,
}

/// The stroke in hand: its seed, held for the whole stroke so a hand-drawn
/// one does not wobble anew each frame, its capture once that has landed, and
/// the highlight it last described, which every display draws again until the
/// pointer moves.
struct Pending {
  stroke: String,
  seed: u32,
  capture: Option<Arc<Capture>>,
  drawn: Option<(AnnotationPoint, AnnotationPoint, Annotation)>,
}

#[derive(Default)]
struct State {
  pending: Option<Pending>,
  kept: Vec<Kept>,
  underlays: Vec<Option<Arc<Underlay>>>,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(Mutex::default);
/// Signalled when a stroke's capture lands, for a release that waits on it.
static LANDED: Condvar = Condvar::new();

fn state() -> MutexGuard<'static, State> {
  STATE
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Whether `stroke`'s capture has landed: until it has, there is nothing to
/// recolour and the highlight is not drawn at all.
pub(super) fn ready(stroke: &str) -> bool {
  state()
    .pending
    .as_ref()
    .is_some_and(|pending| pending.stroke == stroke && pending.capture.is_some())
}

/// Holds a release for `stroke`'s capture, for up to `timeout`: a quick flick
/// lets go before the desktop has been read, and would otherwise leave nothing.
pub(super) fn wait_ready(stroke: &str, timeout: Duration) -> bool {
  let state = state();
  let waiting = |state: &mut State| {
    state
      .pending
      .as_ref()
      .is_some_and(|pending| pending.stroke == stroke && pending.capture.is_none())
  };
  let (mut state, _) = LANDED
    .wait_timeout_while(state, timeout, waiting)
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let landed = !waiting(&mut state);
  landed
    && state
      .pending
      .as_ref()
      .is_some_and(|pending| pending.stroke == stroke)
}

/// The highlight a stroke from `start` to `end` describes, selected from its
/// capture once that has landed, or laid by hand over the box between them.
/// `style`'s width is in desktop points.
pub(super) fn annotation(
  id: &str,
  start: AnnotationPoint,
  end: AnnotationPoint,
  style: &AnnotationStyle,
) -> Annotation {
  let (capture, seed) = {
    let state = state();
    let Some(pending) = state
      .pending
      .as_ref()
      .filter(|pending| pending.stroke == id)
    else {
      return new_highlight(id.to_owned(), start, Some(style), 1.0);
    };
    if let Some((from, to, drawn)) = &pending.drawn {
      if (*from, *to) == (start, end) && drawn.style == *style {
        return drawn.clone();
      }
    }
    (pending.capture.clone(), pending.seed)
  };
  let mut annotation = new_highlight(id.to_owned(), start, Some(style), 1.0);
  let height = style.width.max(1.0);
  let selection = match &capture {
    // A box's strokes are the box's own; the capture is read only for the
    // page's tone under it, which it recolours unless it tints.
    _ if style.manual => {
      let bands = strokes(start, end, height);
      let tone = capture.as_ref().map_or(HighlightTone::UNREAD, |capture| {
        let local: Vec<_> = bands
          .iter()
          .map(|band| {
            band.mapped(|at| AnnotationPoint {
              x: at.x - capture.origin.0,
              y: at.y - capture.origin.1,
            })
          })
          .collect();
        box_tone(
          Some((capture.picture.pixels(), capture.picture.scale())),
          AnnotationPoint {
            x: start.x - capture.origin.0,
            y: start.y - capture.origin.1,
          },
          &local,
          height,
        )
      });
      Selection { bands, tone }
    }
    Some(capture) => {
      // The capture is read in its own pixels from the display's corner.
      let local = |point: AnnotationPoint| AnnotationPoint {
        x: point.x - capture.origin.0,
        y: point.y - capture.origin.1,
      };
      let mut selection = select(
        Some((capture.picture.pixels(), capture.picture.scale())),
        local(start),
        local(end),
        height,
      );
      for band in &mut selection.bands {
        *band = band.mapped(|at| AnnotationPoint {
          x: at.x + capture.origin.0,
          y: at.y + capture.origin.1,
        });
      }
      selection
    }
    None => select(None, start, end, height),
  };
  if let AnnotationShape::Highlight {
    end: shape_end,
    bands,
    tone,
    seed: shape_seed,
    ..
  } = &mut annotation.shape
  {
    *shape_end = end;
    *bands = selection.bands;
    *tone = selection.tone;
    *shape_seed = seed;
  }
  if let Some(pending) = state()
    .pending
    .as_mut()
    .filter(|pending| pending.stroke == id)
  {
    pending.drawn = Some((start, end, annotation.clone()));
  }
  annotation
}

/// A highlight stroke finished as `annotation`: keeps the part of its capture
/// it covers, for as long as it stays on screen.
pub(super) fn commit(annotation: &Annotation) {
  let mut state = state();
  let Some(Pending {
    stroke, capture, ..
  }) = state.pending.take()
  else {
    return;
  };
  let (Some(capture), AnnotationShape::Highlight { bands, .. }) = (capture, &annotation.shape)
  else {
    return;
  };
  if stroke != annotation.id {
    return;
  }
  if let Some(kept) = capture::keep(&annotation.id, &capture, bands) {
    state.kept.push(kept);
  }
}

/// Drops the stroke in hand's capture.
pub(super) fn cancel() {
  state().pending = None;
}

/// Forgets every capture, once nothing is drawn on any display.
pub(super) fn forget() {
  *state() = State::default();
}

/// What display `display` draws highlights over, if any highlight has been
/// drawn on it.
pub(super) fn underlay(display: usize) -> Option<Arc<Underlay>> {
  state().underlays.get(display).cloned().flatten()
}
