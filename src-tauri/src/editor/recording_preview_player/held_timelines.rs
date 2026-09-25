// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The preview's surface timelines for redactions on a recording, read one
//! at a time on a single thread.
//!
//! A timeline decodes its whole clip, and a box that moves or resizes asks
//! for a fresh one at every step. Only the latest ask for each box is worth
//! reading: an older one still queued is dropped and one under way is
//! stopped, so however often a box changes, one decoder runs.

use super::held_surfaces::clip_surfaces;
use crate::editor::annotations::timing::RecordingAnnotationClip;
use crate::editor::annotations::AnnotationShape;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// How many timelines are kept before they are all let go.
const KEPT: usize = 64;

pub(super) type Timeline = Arc<Vec<[f32; 2]>>;
/// What to do once a read lands; shared with the first-frame decodes.
pub(super) type ReadySlot = Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>;

pub(super) fn ready(slot: &ReadySlot) {
  let on_ready = slot.lock().ok().and_then(|slot| slot.clone());
  if let Some(on_ready) = on_ready {
    on_ready();
  }
}

/// What a timeline is read from: the box, its clip's span and where it sits.
#[derive(Clone, Hash, PartialEq, Eq)]
struct TimelineKey {
  id: String,
  start_ms: u64,
  end_ms: u64,
  corners: [u64; 4],
}

impl TimelineKey {
  fn of(clip: &RecordingAnnotationClip) -> Option<Self> {
    let AnnotationShape::Redact { start, end, .. } = clip.annotation.shape else {
      return None;
    };
    Some(Self {
      id: clip.annotation.id.clone(),
      start_ms: clip.start_ms,
      end_ms: clip.end_ms,
      corners: [start.x, start.y, end.x, end.y].map(f64::to_bits),
    })
  }
}

type Read = (TimelineKey, RecordingAnnotationClip, Arc<AtomicBool>);

#[derive(Default)]
struct State {
  timelines: HashMap<TimelineKey, Timeline>,
  /// The read under way and its stop flag. `None` exactly while no thread
  /// is reading, since the thread takes its next read under the same lock
  /// it lands the last one in.
  reading: Option<(TimelineKey, Arc<AtomicBool>)>,
  queued: VecDeque<(TimelineKey, RecordingAnnotationClip)>,
}

pub(super) struct HeldTimelines {
  path: PathBuf,
  duration_ms: u64,
  on_ready: ReadySlot,
  state: Mutex<State>,
}

impl HeldTimelines {
  pub(super) fn new(path: PathBuf, duration_ms: u64, on_ready: ReadySlot) -> Self {
    Self {
      path,
      duration_ms,
      on_ready,
      state: Mutex::new(State::default()),
    }
  }

  /// `clip`'s timeline, or `None` while it waits to be read.
  pub(super) fn get(self: &Arc<Self>, clip: &RecordingAnnotationClip) -> Option<Timeline> {
    let key = TimelineKey::of(clip)?;
    let mut state = self.state.lock().ok()?;
    if let Some(timeline) = state.timelines.get(&key) {
      return Some(Arc::clone(timeline));
    }
    if let Some((reading, stop)) = &state.reading {
      if *reading == key {
        return None;
      }
      if reading.id == key.id {
        stop.store(true, Ordering::Relaxed);
      }
    }
    if state.queued.iter().any(|(queued, _)| *queued == key) {
      return None;
    }
    state.queued.retain(|(queued, _)| queued.id != key.id);
    state.queued.push_back((key, clip.clone()));
    if state.reading.is_none() {
      self.start(&mut state);
    }
    None
  }

  fn start(self: &Arc<Self>, state: &mut State) {
    let Some(read) = Self::next(state) else {
      return;
    };
    let timelines = Arc::clone(self);
    let spawned = std::thread::Builder::new()
      .name("recording-preview-held-surface".to_owned())
      .spawn(move || timelines.work(read));
    if spawned.is_err() {
      state.reading = None;
      state.queued.clear();
    }
  }

  fn next(state: &mut State) -> Option<Read> {
    let Some((key, clip)) = state.queued.pop_front() else {
      state.reading = None;
      return None;
    };
    let stop = Arc::new(AtomicBool::new(false));
    state.reading = Some((key.clone(), Arc::clone(&stop)));
    Some((key, clip, stop))
  }

  fn work(&self, mut read: Read) {
    loop {
      let (key, clip, stop) = read;
      let timeline = clip_surfaces(&self.path, self.duration_ms, &clip, &stop);
      let landed = !stop.load(Ordering::Relaxed);
      let next = {
        let Ok(mut state) = self.state.lock() else {
          return;
        };
        if landed {
          if state.timelines.len() >= KEPT {
            state.timelines.clear();
          }
          state.timelines.insert(key, Arc::new(timeline));
        }
        Self::next(&mut state)
      };
      if landed {
        ready(&self.on_ready);
      }
      let Some(next) = next else {
        return;
      };
      read = next;
    }
  }
}
