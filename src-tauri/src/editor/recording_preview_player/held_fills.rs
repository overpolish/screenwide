// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The preview's fills for redactions on a recording: a secure pixelation's
//! zones from its clip's first frame, and the surface timeline across the
//! clip.
//!
//! Both are read from full-resolution decodes, which are slow things to do
//! in the middle of drawing a frame. The last few first frames are kept, so
//! dragging a box or changing its style reads its fill again straight away.
//! What is not yet read is read on a thread of its own: until a first frame
//! lands the box is drawn flat in its own colour, which hides as much, and
//! until its timeline lands it holds the first frame's surface. A paused
//! preview is drawn again the moment either does.

use super::held_surfaces::{first_box, reads_picture};
use super::held_timelines::{ready, HeldTimelines, ReadySlot};
use crate::editor::annotations::redact::held::HeldFill;
use crate::editor::annotations::redact::native::{held_fill, RedactPicture};
use crate::editor::annotations::redact::surface_timeline::surface_at;
use crate::editor::annotations::timing::RecordingAnnotationClip;
use crate::editor::annotations::Annotation;
use crate::screenshots::CapturedImage;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// How many decoded first frames are kept: enough for the box being worked
/// on and its neighbour on the timeline.
const FRAMES: usize = 2;
/// How many fills are kept before they are all let go.
const KEPT: usize = 64;

/// What a fill is read from: the frame, the box, how it is read, and how many
/// logical points the capture is wide, which sizes a pixelation's blocks.
#[derive(Clone, Hash, PartialEq, Eq)]
struct FillKey {
  start_ms: u64,
  corners: [u64; 4],
  redaction: u8,
  width: u64,
  capture_width_points: u64,
}

/// The box a fill is read from, where it is on its clip's first frame.
fn corners(clip: &RecordingAnnotationClip) -> Option<[u64; 4]> {
  let (start, end) = first_box(clip)?;
  Some([start.x, start.y, end.x, end.y].map(f64::to_bits))
}

#[derive(Default)]
struct State {
  frames: VecDeque<(u64, Arc<CapturedImage>)>,
  decoding: HashSet<u64>,
  fills: HashMap<FillKey, Arc<HeldFill>>,
}

pub(crate) struct HeldFills {
  path: PathBuf,
  duration_ms: u64,
  state: Mutex<State>,
  timelines: Arc<HeldTimelines>,
  on_ready: ReadySlot,
}

impl HeldFills {
  pub(crate) fn new(path: PathBuf, duration_ms: u64) -> Self {
    let on_ready = ReadySlot::default();
    Self {
      timelines: Arc::new(HeldTimelines::new(
        path.clone(),
        duration_ms,
        Arc::clone(&on_ready),
      )),
      path,
      duration_ms,
      state: Mutex::new(State::default()),
      on_ready,
    }
  }

  /// What to do once something read on a thread lands: draw a paused
  /// preview again.
  pub(crate) fn set_on_ready(&self, on_ready: Arc<dyn Fn() + Send + Sync>) {
    if let Ok(mut slot) = self.on_ready.lock() {
      *slot = Some(on_ready);
    }
  }

  /// Hands each of `annotations` that reads the picture its fill as it
  /// stands `source_ms` into the recording, where one is to hand. `clips`
  /// are the frame's clips by id; `capture_width_points` is how many logical
  /// points the recording is wide.
  pub(crate) fn attach(
    self: &Arc<Self>,
    annotations: &mut [Annotation],
    clips: &[RecordingAnnotationClip],
    capture_width_points: f64,
    source_ms: u64,
  ) {
    for annotation in annotations {
      if !reads_picture(annotation.style.redaction) {
        continue;
      }
      let Some(clip) = clips
        .iter()
        .find(|clip| clip.annotation.id == annotation.id)
      else {
        continue;
      };
      let Some(fill) = self.fill(clip, capture_width_points) else {
        continue;
      };
      let elapsed = source_ms.saturating_sub(clip.start_ms) as f32;
      let surface = self
        .timelines
        .get(clip)
        .and_then(|timeline| surface_at(&timeline, elapsed));
      annotation.held = Some(match surface {
        Some(surface) => Arc::new(HeldFill {
          surface: Some(surface.map(|channel| (channel * 255.0).round() as u8)),
          ..(*fill).clone()
        }),
        None => fill,
      });
    }
  }

  fn fill(
    self: &Arc<Self>,
    clip: &RecordingAnnotationClip,
    capture_width_points: f64,
  ) -> Option<Arc<HeldFill>> {
    let style = &clip.annotation.style;
    let key = FillKey {
      start_ms: clip.start_ms,
      corners: corners(clip)?,
      redaction: style.redaction as u8,
      width: style.width.to_bits(),
      capture_width_points: capture_width_points.to_bits(),
    };
    let mut state = self.state.lock().ok()?;
    if let Some(fill) = state.fills.get(&key) {
      return Some(Arc::clone(fill));
    }
    let Some(frame) = state
      .frames
      .iter()
      .find(|(start_ms, _)| *start_ms == clip.start_ms)
      .map(|(_, frame)| Arc::clone(frame))
    else {
      if state.decoding.insert(clip.start_ms) {
        self.decode(clip.start_ms);
      }
      return None;
    };
    let (start, end) = first_box(clip)?;
    let picture = RedactPicture::new(&frame.rgba, frame.width, frame.height, capture_width_points);
    let fill = Arc::new(held_fill(start, end, style, &picture));
    if state.fills.len() >= KEPT {
      state.fills.clear();
    }
    state.fills.insert(key, Arc::clone(&fill));
    Some(fill)
  }

  fn decode(self: &Arc<Self>, start_ms: u64) {
    let fills = Arc::clone(self);
    let spawned = std::thread::Builder::new()
      .name("recording-preview-held-frame".to_owned())
      .spawn(move || {
        let frame = super::platform::source_frame_image(&fills.path, start_ms, fills.duration_ms);
        if let Ok(mut state) = fills.state.lock() {
          state.decoding.remove(&start_ms);
          if let Ok(frame) = frame {
            if state.frames.len() >= FRAMES {
              state.frames.pop_front();
            }
            state.frames.push_back((start_ms, Arc::new(frame)));
          }
        }
        ready(&fills.on_ready);
      });
    if spawned.is_err() {
      if let Ok(mut state) = self.state.lock() {
        state.decoding.remove(&start_ms);
      }
    }
  }
}
