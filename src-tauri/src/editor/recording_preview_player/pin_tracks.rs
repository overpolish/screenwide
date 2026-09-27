// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pinned annotations' paths in the preview, worked out on one thread.
//!
//! A path is joined from legs, each followed from one keyframe towards the
//! next, and both are kept by what they were worked out from, so a
//! correction only asks again for the two legs that meet at it. Only the
//! latest ask for each annotation is worth working on: an older one still
//! queued is dropped and one under way is stopped. Until its path lands an
//! annotation keeps the path it had, so it does not jump while a correction
//! is followed through.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::held_timelines::ReadySlot;
use super::pin_cache::cached_path;
use super::pin_paths::PinStatus;
use super::platform::LumaReader;
use crate::editor::annotations::pin::find::find_again;
use crate::editor::annotations::pin::{PinRequest, PinnedPath, TRACKING_SIDE};
use crate::editor::annotations::snap::source_per_output;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};

/// Working through the queue on the tracking thread.
#[path = "pin_tracks_work.rs"]
mod work;

/// How far a path's progress moves before the timeline is told again.
const PROGRESS_STEP: f32 = 0.04;

type StatusSlot = Arc<Mutex<Option<Arc<dyn Fn(PinStatus) + Send + Sync>>>>;

struct Job {
  id: String,
  request: PinRequest,
  /// Where the playhead was when it was asked for.
  near_ms: u64,
}

#[derive(Default)]
struct State {
  queue: VecDeque<Job>,
  /// The job under way: its annotation, its key and its stop flag. `None`
  /// exactly while no thread is working, since the thread takes its next job
  /// under the same lock it lands the last one in.
  running: Option<(String, u64, Arc<AtomicBool>)>,
  /// Each annotation's latest path, which it keeps while a newer one is
  /// worked out.
  latest: HashMap<String, Arc<PinnedPath>>,
  /// The path each annotation's status was last reported for.
  reported: HashMap<String, u64>,
}

pub(crate) struct PinTracks {
  recording: PathBuf,
  duration_ms: u64,
  source: (u32, u32),
  frames_per_second: f64,
  state: Mutex<State>,
  on_ready: ReadySlot,
  on_status: StatusSlot,
}

impl PinTracks {
  pub(crate) fn new(
    recording: PathBuf,
    duration_ms: u64,
    source: (u32, u32),
    frames_per_second: Option<f64>,
  ) -> Self {
    Self {
      recording,
      duration_ms,
      source,
      frames_per_second: frames_per_second.filter(|fps| *fps > 0.0).unwrap_or(60.0),
      state: Mutex::new(State::default()),
      on_ready: ReadySlot::default(),
      on_status: StatusSlot::default(),
    }
  }

  /// What to do once a path lands: attach it and draw a paused preview again.
  pub(crate) fn set_on_ready(&self, on_ready: Arc<dyn Fn() + Send + Sync>) {
    if let Ok(mut slot) = self.on_ready.lock() {
      *slot = Some(on_ready);
    }
  }

  /// Who hears how each pin's path is coming along.
  pub(crate) fn set_on_status(&self, on_status: Arc<dyn Fn(PinStatus) + Send + Sync>) {
    if let Ok(mut slot) = self.on_status.lock() {
      *slot = Some(on_status);
    }
  }

  fn report(&self, status: PinStatus) {
    let listener = self.on_status.lock().ok().and_then(|slot| slot.clone());
    if let Some(listener) = listener {
      listener(status);
    }
  }

  /// Every pin's latest landed status, for a timeline that starts listening
  /// after they were first told.
  pub(crate) fn statuses(&self) -> Vec<PinStatus> {
    self.state.lock().map_or_else(
      |_| Vec::new(),
      |state| {
        state
          .latest
          .iter()
          .map(|(id, path)| PinStatus::landed(id, path))
          .collect()
      },
    )
  }

  /// [`PinnedPath::last_seen`] on `annotation_id`'s latest path.
  pub(crate) fn last_seen(&self, annotation_id: &str, ms: u64) -> Option<[f64; 2]> {
    self
      .state
      .lock()
      .ok()?
      .latest
      .get(annotation_id)?
      .last_seen(ms)
  }

  /// Where `clip`'s annotation goes when the hand says its content is back in
  /// view at `ms`: found again near where it was last shown, by how it looked
  /// on the last keyframe before that it was in view on, or else on the frame
  /// it was pinned on - one keyframe may have been made while it was half
  /// covered - and where it was last shown when neither finds it. Nothing
  /// where it is not hidden. `image_width` places the tips counters and text
  /// boxes are followed by, as [`PinTracks::attach`] does.
  pub(crate) fn back_in_view(
    &self,
    clip: &RecordingAnnotationClip,
    ms: u64,
    image_width: f64,
  ) -> Option<[f64; 2]> {
    let pin = clip.pin.as_ref()?;
    let near = self.last_seen(&clip.annotation.id, ms)?;
    let keyframes = pin.sorted();
    let before = keyframes
      .iter()
      .rev()
      .find(|keyframe| !keyframe.out_of_view && keyframe.ms <= ms);
    let pinned = keyframes
      .iter()
      .find(|keyframe| keyframe.ms == pin.pinned_ms)
      .filter(|pinned| before.is_none_or(|before| before.ms != pinned.ms));
    let scale = source_per_output(self.source, image_width);
    let target = PinRequest::of(clip, pin, self.source, scale).target;
    let mut reader = LumaReader::open(&self.recording, self.duration_ms, TRACKING_SIDE).ok()?;
    let found = [before, pinned].into_iter().flatten().find_map(|keyframe| {
      find_again(&mut reader, target, *keyframe, near, ms)
        .ok()
        .flatten()
    });
    Some(found.unwrap_or(near))
  }

  /// Hands each pinned clip among `clips` its path, or the one it had while
  /// its new one is asked for. `image_width` is how wide the screen is drawn
  /// on the canvas, in canvas pixels, or zero where that is not known yet.
  pub(crate) fn attach(
    self: &Arc<Self>,
    clips: &mut [RecordingAnnotationClip],
    position_ms: u64,
    image_width: f64,
  ) {
    let scale = source_per_output(self.source, image_width);
    for clip in clips.iter_mut() {
      if clip.track_id != AnnotationTrack::Primary {
        continue;
      }
      let Some(pin) = clip.pin.as_ref() else {
        continue;
      };
      let request = PinRequest::of(clip, pin, self.source, scale);
      let key = request.key();
      let id = clip.annotation.id.clone();
      let path = match cached_path(&self.recording, key) {
        Some(path) => {
          self.landed(&id, &path);
          Some(path)
        }
        None => {
          self.ask(Job {
            id: id.clone(),
            request,
            near_ms: position_ms,
          });
          self
            .state
            .lock()
            .ok()
            .and_then(|state| state.latest.get(&id).cloned())
        }
      };
      if let Some(pin) = clip.pin.as_mut() {
        pin.path = path;
      }
    }
  }

  /// Keeps `path` as `id`'s latest and tells the timeline, once per path.
  fn landed(&self, id: &str, path: &Arc<PinnedPath>) {
    let fresh = self.state.lock().is_ok_and(|mut state| {
      state.latest.insert(id.to_owned(), Arc::clone(path));
      state.reported.insert(id.to_owned(), path.key) != Some(path.key)
    });
    if fresh {
      self.report(PinStatus::landed(id, path));
    }
  }

  fn ask(self: &Arc<Self>, job: Job) {
    let key = job.request.key();
    let id = job.id.clone();
    let Ok(mut state) = self.state.lock() else {
      return;
    };
    if let Some((running, running_key, stop)) = &state.running {
      if *running == id {
        if *running_key == key {
          return;
        }
        stop.store(true, Ordering::Relaxed);
      }
    }
    if state
      .queue
      .iter()
      .any(|job| job.id == id && job.request.key() == key)
    {
      return;
    }
    state.queue.retain(|job| job.id != id);
    state.queue.push_back(job);
    if state.running.is_none() {
      let Some(job) = Self::next(&mut state) else {
        return;
      };
      let tracks = Arc::clone(self);
      let spawned = std::thread::Builder::new()
        .name("recording-preview-pin".to_owned())
        .spawn(move || tracks.work(job));
      if spawned.is_err() {
        state.running = None;
        state.queue.clear();
      }
    }
  }

  fn next(state: &mut State) -> Option<(Job, Arc<AtomicBool>)> {
    let Some(job) = state.queue.pop_front() else {
      state.running = None;
      return None;
    };
    let stop = Arc::new(AtomicBool::new(false));
    state.running = Some((job.id.clone(), job.request.key(), Arc::clone(&stop)));
    Some((job, stop))
  }
}
