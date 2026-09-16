// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pointer and key events turned into annotations.
//!
//! A stroke is an arrow from where the pointer went down to where it is now,
//! drawn in the dress the settings carry. It reaches [`super::live_clips`]
//! only when the button comes up: an unfinished stroke is on screen but not in
//! the recording, so a drag that is abandoned costs nothing.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::Instant;

use crate::editor::annotations::model::new_arrow;
use crate::editor::annotations::{Annotation, AnnotationHead, AnnotationPoint, AnnotationStyle};

/// Virtual key codes, which AppKit reports by physical position.
const KEY_Z: u16 = 6;
const KEY_BACKSPACE: u16 = 51;
const KEY_FORWARD_DELETE: u16 = 117;

/// The modifier bits `native_overlay_macos.h` sends.
const MODIFIER_COMMAND: u32 = 1;
const MODIFIER_SHIFT: u32 = 2;

/// Pointer phases as the native overlay reports them.
const PHASE_DOWN: u32 = 0;
const PHASE_DRAG: u32 = 1;

/// The stroke in hand: when and where it started, and where the pointer is
/// now. The start time is what the annotation is timed from, so a clip covers the
/// drawing rather than beginning once it is over.
struct Stroke {
  id: String,
  started_at: Instant,
  start: AnnotationPoint,
  end: AnnotationPoint,
}

static DRAWING: LazyLock<Mutex<Option<Stroke>>> = LazyLock::new(|| Mutex::new(None));
static NEXT_ANNOTATION: AtomicU64 = AtomicU64::new(1);

fn drawing() -> MutexGuard<'static, Option<Stroke>> {
  DRAWING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn style() -> AnnotationStyle {
  let settings = super::settings::current();
  AnnotationStyle {
    color: settings.default_color,
    head: AnnotationHead::End,
    width: settings.default_width,
  }
}

/// The arrow a stroke currently describes. Its control point and bend come
/// from the editor's own `new_arrow`, so a stroke drawn live and one drawn in
/// the editor are the same shape from the first frame.
fn arrow(stroke: &Stroke, style: &AnnotationStyle) -> Annotation {
  new_arrow(stroke.id.clone(), stroke.start, stroke.end, Some(style))
}

/// One pointer step, and the annotation it completed with the moment its stroke
/// began. Kept apart from the live annotation list so the gesture can be driven a
/// step at a time.
fn step(
  drawing: &mut Option<Stroke>,
  phase: u32,
  at: Instant,
  point: AnnotationPoint,
  style: &AnnotationStyle,
) -> Option<(Annotation, Instant)> {
  match phase {
    PHASE_DOWN => {
      *drawing = Some(Stroke {
        id: format!("live-{}", NEXT_ANNOTATION.fetch_add(1, Ordering::Relaxed)),
        started_at: at,
        start: point,
        end: point,
      });
      None
    }
    PHASE_DRAG => {
      if let Some(stroke) = drawing.as_mut() {
        stroke.end = point;
      }
      None
    }
    _ => {
      let mut stroke = drawing.take()?;
      stroke.end = point;
      // A click that never travelled is not an annotation. Otherwise every stray
      // click while the overlay is up would leave a dot on screen, and a clip
      // in the recording.
      (stroke.start.x != stroke.end.x || stroke.start.y != stroke.end.y)
        .then(|| (arrow(&stroke, style), stroke.started_at))
    }
  }
}

/// The stroke in hand, for the overlay to draw. Annotations already on screen come
/// from [`super::live_clips`].
pub(super) fn in_progress() -> Option<Annotation> {
  let style = style();
  drawing().as_ref().map(|stroke| arrow(stroke, &style))
}

/// A pointer step in global desktop points: 0 down, 1 drag, 2 up.
pub(super) fn pointer(phase: u32, x: f64, y: f64) {
  let completed = step(
    &mut drawing(),
    phase,
    Instant::now(),
    AnnotationPoint { x, y },
    &style(),
  );
  if let Some((annotation, started_at)) = completed {
    super::live_clips::add(annotation, started_at);
  }
}

/// A key press. Reports whether the overlay acted on it, which is what tells
/// the native side to redraw. Escape and the activation shortcut arrive
/// through their own global registrations and are never seen here.
pub(super) fn key(key_code: u16, modifiers: u32) -> bool {
  match key_code {
    // Undo takes the last annotation off the screen. While a recording runs its
    // clip is kept: the annotation was visible for exactly that long, and the
    // editor is where an unwanted clip is deleted.
    KEY_Z if modifiers & MODIFIER_COMMAND != 0 && modifiers & MODIFIER_SHIFT == 0 => {
      let had_stroke = drawing().take().is_some();
      super::live_clips::remove_last() || had_stroke
    }
    KEY_BACKSPACE | KEY_FORWARD_DELETE => {
      drawing().take();
      super::live_clips::clear();
      true
    }
    _ => false,
  }
}

/// Drops the stroke in hand. The overlay closing must not leave half a drag
/// for the next session to finish.
pub(super) fn cancel() {
  drawing().take();
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
