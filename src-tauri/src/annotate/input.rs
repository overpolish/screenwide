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
use std::time::{Duration, Instant};

use crate::editor::annotations::AnnotationKind;
use crate::editor::annotations::{Annotation, AnnotationPoint};

#[path = "input_keys.rs"]
mod keys;
#[path = "input_stroke.rs"]
mod stroke;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use keys::{KEY_BACKSPACE, KEY_FORWARD_DELETE, KEY_Z, TOOL_KEYS};
use stroke::{pen_down, Stroke};

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

static DRAWING: LazyLock<Mutex<Option<Stroke>>> = LazyLock::new(|| Mutex::new(None));
static NEXT_ANNOTATION: AtomicU64 = AtomicU64::new(1);
/// How often a pen stroke is asked whether the hand has rested on it: well
/// inside the hold, so a stroke is read soon after its rest has lasted.
const HOLD_TICK: Duration = Duration::from_millis(40);

fn drawing() -> MutexGuard<'static, Option<Stroke>> {
  DRAWING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What a press starts: the tool in hand, what a counter dropped by it would
/// be numbered and aimed at, and whether a highlight is laid by hand.
struct Tool {
  shape: AnnotationKind,
  value: u32,
  angle: f64,
  manual: bool,
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
      let id = format!("live-{}", NEXT_ANNOTATION.fetch_add(1, Ordering::Relaxed));
      let (line, hold) = if tool.shape == AnnotationKind::Draw {
        pen_down(&id, point, at)
      } else {
        (None, None)
      };
      *drawing = Some(Stroke {
        id,
        started_at: at,
        start: point,
        end: point,
        shape: tool.shape,
        value: tool.value,
        angle: tool.angle,
        manual: tool.shape == AnnotationKind::Highlight && tool.manual,
        seed: crate::editor::annotations::highlight::model::fresh_seed(),
        line,
        hold,
      });
      None
    }
    PHASE_DRAG => {
      if let Some(stroke) = drawing.as_mut() {
        stroke.carry(point, at);
      }
      None
    }
    _ => {
      let mut stroke = drawing.take()?;
      stroke.carry(point, at);
      // A click that never travelled is not an arrow. Otherwise every stray
      // click while the overlay is up would leave a dot on screen, and a clip
      // in the recording. A counter is dropped by that very click.
      let drawn = stroke.is_drawn().then(|| stroke.annotation()).flatten()?;
      Some((drawn, stroke.started_at))
    }
  }
}

/// The stroke in hand, for the overlay to draw. Annotations already on screen
/// come from [`super::live_clips`].
pub(super) fn in_progress() -> Option<Annotation> {
  let drawing = drawing();
  drawing
    .as_ref()
    .filter(|stroke| stroke.is_drawn())?
    .annotation()
}

/// Whether a spotlight on screen, or the one in hand, blurs: what the
/// overlays soften their underlay for.
pub(super) fn spotlight_blurs() -> bool {
  let blurs = |annotation: &Annotation| {
    annotation.style.blur
      && matches!(
        annotation.shape,
        crate::editor::annotations::AnnotationShape::Spotlight { .. }
      )
  };
  super::live_clips::annotations().iter().any(blurs) || in_progress().is_some_and(|a| blurs(&a))
}

/// A pointer step in global desktop points: 0 down, 1 drag, 2 up. `app` is
/// what a highlight's press, or a blurring spotlight's, captures the desktop
/// with.
pub(super) fn pointer(app: Option<&tauri::AppHandle>, phase: u32, x: f64, y: f64) {
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
    manual: settings.highlight_manual,
  };
  let point = AnnotationPoint { x, y };
  if phase != PHASE_DOWN && phase != PHASE_DRAG {
    // A quick flick lets go before the desktop under it has been read.
    let highlight = |stroke: &&Stroke| stroke.shape == AnnotationKind::Highlight;
    let flicked = drawing()
      .as_ref()
      .filter(highlight)
      .map(|stroke| stroke.id.clone());
    if let Some(stroke) = flicked {
      super::highlight::wait_ready(&stroke, Duration::from_millis(400));
    }
  }
  let completed = step(&mut drawing(), phase, Instant::now(), point, &tool);
  if phase == PHASE_DOWN {
    let stroke = drawing().as_ref().map(|stroke| stroke.id.clone());
    match (tool.shape, stroke, app) {
      (AnnotationKind::Highlight, Some(stroke), Some(app)) => {
        super::highlight::begin(app, &stroke, point);
      }
      (AnnotationKind::Spotlight, Some(_), Some(app)) if settings.spotlight_blur => {
        super::highlight::refresh(app, point);
      }
      (AnnotationKind::Draw, Some(stroke), Some(app)) => watch_hold(app.clone(), stroke),
      _ => {}
    }
  }
  if let Some((annotation, started_at)) = completed {
    if annotation.shape.kind() == AnnotationKind::Highlight {
      super::highlight::commit(&annotation);
    }
    super::live_clips::add(annotation, started_at);
  }
}

/// Watches the pen stroke `id` until it ends. The pointer reports nothing
/// while it rests, so no pointer step can notice a rest; the stroke itself
/// decides whether it has rested long enough, and what it is taken for.
fn watch_hold(app: tauri::AppHandle, id: String) {
  std::thread::spawn(move || loop {
    std::thread::sleep(HOLD_TICK);
    let changed = match drawing().as_mut() {
      Some(stroke) if stroke.id == id => stroke.hold(Instant::now()),
      _ => return,
    };
    if changed {
      super::native_overlay::request_redraw(&app);
    }
  });
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
  super::highlight::cancel();
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
