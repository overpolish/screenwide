// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pointer and key events turned into annotations.
//!
//! A stroke is the annotation the tool in hand draws, in the dress the settings
//! carry: an arrow from where the pointer went down to where it is now, or a
//! counter dropped at the press and carried by the drag. It reaches
//! [`super::live_clips`] only when the button comes up: an unfinished stroke
//! is on screen but not in the recording, so a drag that is abandoned costs
//! nothing.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::Instant;

use crate::editor::annotations::arrow::new_arrow;
use crate::editor::annotations::counter::new_counter;
use crate::editor::annotations::AnnotationKind;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationStyle};

/// Virtual key codes, which AppKit reports by physical position.
#[cfg(target_os = "macos")]
mod key_codes {
  pub(super) const KEY_A: u16 = 0;
  pub(super) const KEY_Z: u16 = 6;
  pub(super) const KEY_N: u16 = 45;
  pub(super) const KEY_BACKSPACE: u16 = 51;
  pub(super) const KEY_FORWARD_DELETE: u16 = 117;
}

/// Windows virtual key codes, which name the character rather than the
/// position it sits at.
#[cfg(target_os = "windows")]
mod key_codes {
  pub(super) const KEY_A: u16 = 0x41;
  pub(super) const KEY_Z: u16 = 0x5A;
  pub(super) const KEY_N: u16 = 0x4E;
  pub(super) const KEY_BACKSPACE: u16 = 0x08;
  pub(super) const KEY_FORWARD_DELETE: u16 = 0x2E;
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
use key_codes::{KEY_A, KEY_BACKSPACE, KEY_FORWARD_DELETE, KEY_N, KEY_Z};

/// The tool each unmodified letter picks up, the twin of the toolbar's own
/// hints. A shape added to the overlay takes its letter here.
#[cfg(any(target_os = "macos", target_os = "windows"))]
const TOOL_KEYS: &[(u16, AnnotationKind)] = &[
  (KEY_A, AnnotationKind::Arrow),
  (KEY_N, AnnotationKind::Counter),
];

/// The modifier bits the native overlay sends. Windows reports Ctrl as
/// `MODIFIER_COMMAND`: undo is the same gesture under a different name.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) const MODIFIER_COMMAND: u32 = 1;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) const MODIFIER_SHIFT: u32 = 2;

/// Pointer phases as the native overlay reports them. Up is the fallback
/// phase in [`step`], so only the Windows overlay names it.
pub(super) const PHASE_DOWN: u32 = 0;
pub(super) const PHASE_DRAG: u32 = 1;
#[cfg(target_os = "windows")]
pub(super) const PHASE_UP: u32 = 2;

/// The stroke in hand: the annotation it makes, when and where it started, and
/// where the pointer is now. The start time is what the annotation is timed
/// from, so a clip covers the drawing rather than beginning once it is over.
///
/// The shape, the number and the aim are taken at the press and held: a tool
/// picked up mid-drag chooses what the *next* annotation is, rather than
/// reshaping the one being drawn. The dress is read every frame, so a colour
/// changed mid-drag shows on the stroke in hand.
struct Stroke {
  id: String,
  started_at: Instant,
  start: AnnotationPoint,
  end: AnnotationPoint,
  shape: AnnotationKind,
  /// The counter's place in the order it was dropped in, and where its tail
  /// points. Neither is read for an arrow.
  value: u32,
  angle: f64,
}

static DRAWING: LazyLock<Mutex<Option<Stroke>>> = LazyLock::new(|| Mutex::new(None));
static NEXT_ANNOTATION: AtomicU64 = AtomicU64::new(1);

fn drawing() -> MutexGuard<'static, Option<Stroke>> {
  DRAWING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The dress an annotation of `shape` is drawn in. An arrow's stroke and a
/// counter's disc are different measurements of different things, so the width
/// comes from the setting that belongs to the shape. The overlay offers no
/// text tool, and its settings refuse one, so a text box never reaches here.
fn style(shape: AnnotationKind) -> AnnotationStyle {
  let settings = super::settings::current();
  AnnotationStyle {
    align: Default::default(),
    color: settings.default_color,
    head: settings.default_head,
    width: match shape {
      AnnotationKind::Arrow | AnnotationKind::Text => settings.default_width,
      AnnotationKind::Counter => settings.default_counter_size,
    },
  }
}

/// The annotation a stroke currently describes, built by the editor's own
/// constructors so an annotation drawn live and one drawn in the editor are the
/// same shape from the first frame.
///
/// A counter sits where the pointer is rather than where the press landed: it
/// is dropped whole, and the same drag carries it, exactly as a fresh counter
/// in the editor is carried.
fn annotation(stroke: &Stroke, style: &AnnotationStyle) -> Option<Annotation> {
  match stroke.shape {
    AnnotationKind::Arrow => Some(new_arrow(
      stroke.id.clone(),
      stroke.start,
      stroke.end,
      Some(style),
    )),
    AnnotationKind::Counter => Some(new_counter(
      stroke.id.clone(),
      stroke.end,
      stroke.value,
      Some(style),
      Some(stroke.angle),
    )),
    AnnotationKind::Text => None,
  }
}

/// Whether a stroke has anything to show. An arrow with both ends in one
/// place is a blob, and the press that starts every stroke would flash one
/// before the drag begins; a counter is an annotation the moment it is dropped.
fn is_drawn(stroke: &Stroke) -> bool {
  match stroke.shape {
    AnnotationKind::Arrow => stroke.start != stroke.end,
    AnnotationKind::Counter => true,
    AnnotationKind::Text => false,
  }
}

/// What a press starts: the tool in hand, and what a counter dropped by it
/// would be numbered and aimed at.
struct Tool {
  shape: AnnotationKind,
  value: u32,
  angle: f64,
}

/// One pointer step, and the annotation it completed with the moment its stroke
/// began. Kept apart from the live annotation list so the gesture can be driven
/// a step at a time.
///
/// `tool` is the annotation a press starts: the shape in hand, the number a
/// counter takes, and where its tail points. A drag or a release reads none of
/// it - the stroke carries its own.
fn step(
  drawing: &mut Option<Stroke>,
  phase: u32,
  at: Instant,
  point: AnnotationPoint,
  tool: &Tool,
) -> Option<(Annotation, Instant)> {
  match phase {
    PHASE_DOWN => {
      *drawing = Some(Stroke {
        id: format!("live-{}", NEXT_ANNOTATION.fetch_add(1, Ordering::Relaxed)),
        started_at: at,
        start: point,
        end: point,
        shape: tool.shape,
        value: tool.value,
        angle: tool.angle,
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
      // A click that never travelled is not an arrow. Otherwise every stray
      // click while the overlay is up would leave a dot on screen, and a clip
      // in the recording. A counter is dropped by that very click.
      let drawn = is_drawn(&stroke)
        .then(|| annotation(&stroke, &style(stroke.shape)))
        .flatten()?;
      Some((drawn, stroke.started_at))
    }
  }
}

/// The stroke in hand, for the overlay to draw. Annotations already on screen
/// come from [`super::live_clips`].
pub(super) fn in_progress() -> Option<Annotation> {
  let drawing = drawing();
  let stroke = drawing.as_ref().filter(|stroke| is_drawn(stroke))?;
  annotation(stroke, &style(stroke.shape))
}

/// A pointer step in global desktop points: 0 down, 1 drag, 2 up.
pub(super) fn pointer(phase: u32, x: f64, y: f64) {
  let settings = super::settings::current();
  let tool = Tool {
    shape: settings.default_shape,
    // The number a counter takes is its place among the counters already on
    // screen, which only a press has to know. Clearing the screen starts the
    // count again, and undo only ever takes the newest, so the numbers stay
    // contiguous without being rewritten.
    value: if phase == PHASE_DOWN {
      super::live_clips::next_counter_value()
    } else {
      0
    },
    angle: settings.default_counter_angle,
  };
  let completed = step(
    &mut drawing(),
    phase,
    Instant::now(),
    AnnotationPoint { x, y },
    &tool,
  );
  if let Some((annotation, started_at)) = completed {
    super::live_clips::add(annotation, started_at);
  }
}

/// A key press. Reports whether the overlay acted on it, which is what tells
/// the native side to redraw. Escape and the activation shortcut arrive
/// through their own global registrations and are never seen here.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) fn key(app: &tauri::AppHandle, key_code: u16, modifiers: u32) -> bool {
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
    _ if modifiers == 0 => match TOOL_KEYS.iter().find(|(key, _)| *key == key_code) {
      Some((_, shape)) => {
        // A settings write, like the toolbar's: the toolbar and the Settings
        // page follow the change event. Nothing on screen changes.
        if let Err(error) = super::settings::store_default_shape(app, *shape) {
          eprintln!("Could not choose the annotate tool: {error}");
        }
        false
      }
      None => false,
    },
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
