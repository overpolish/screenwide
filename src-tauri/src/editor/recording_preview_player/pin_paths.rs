// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pinned paths as they are worked out and kept, shared by the preview and
//! the export: the cache of legs and paths by recording, the path of one
//! request, and the export's own pass over its clips.
//!
//! The legs a path is missing are followed nearest the playhead first, on two
//! decoders at once: past two the hardware decoder is the limit. Each leg the
//! first of them finishes is handed over as a path straight away, so the
//! preview shows it while the rest is followed.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use serde::Serialize;

use super::pin_cache::{cached_leg, cached_path, store_leg, store_path, Samples};
use super::platform::LumaReader;
use crate::editor::annotations::pin::leg::track_leg;
use crate::editor::annotations::pin::{
  assemble, FrameSource, PinRequest, PinnedPath, TRACKING_SIDE,
};
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};

/// What the timeline shows of a pin: how far its path is worked out, and
/// where it was followed poorly or went off the frame.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PinStatus {
  pub(crate) annotation_id: String,
  /// From 0 to 1 while the path is worked out; absent once it has landed.
  pub(crate) progress: Option<f32>,
  pub(crate) weak: Vec<[u64; 2]>,
  pub(crate) hidden: Vec<[u64; 2]>,
}

impl PinStatus {
  pub(super) fn landed(annotation_id: &str, path: &PinnedPath) -> Self {
    Self {
      annotation_id: annotation_id.to_owned(),
      progress: None,
      weak: path.weak.clone(),
      hidden: path.hidden.clone(),
    }
  }
}

/// A pin's status as the editor window hears it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PinStatusEvent {
  pub(crate) session_id: u64,
  #[serde(flatten)]
  pub(crate) status: PinStatus,
}

/// What working out a path needs besides its request.
pub(super) struct Work<'a> {
  pub(super) recording: &'a Path,
  pub(super) duration_ms: u64,
  pub(super) frames_per_second: f64,
  pub(super) stop: &'a AtomicBool,
  /// The legs nearest this moment are followed first: where the playhead is.
  pub(super) near_ms: Option<u64>,
}

/// Works out `request`'s path, taking what legs the cache has and following
/// the rest on `reader` and a second decoder of its own. `progress` hears the
/// share of frames followed so far; `partial` is handed the path joined from
/// the legs to hand each time one lands while more are still to come. `None`
/// once `stop` is set.
pub(super) fn work_out(
  reader: &mut LumaReader,
  work: &Work<'_>,
  request: &PinRequest,
  progress: &(dyn Fn(f32) + Sync),
  partial: &mut dyn FnMut(&PinnedPath),
) -> Result<Option<Arc<PinnedPath>>, String> {
  let legs = request.legs();
  let fps = work.frames_per_second;
  let total = legs.iter().map(|leg| leg.frames(fps)).sum::<u64>().max(1);
  let mut resolved: Vec<Option<Samples>> = legs
    .iter()
    .map(|leg| cached_leg(work.recording, leg))
    .collect();
  let done = AtomicU64::new(
    legs
      .iter()
      .zip(&resolved)
      .filter(|(_, samples)| samples.is_some())
      .map(|(leg, _)| leg.frames(fps))
      .sum(),
  );
  let step = || {
    let done = done.fetch_add(1, Ordering::Relaxed) + 1;
    progress(done as f32 / total as f32);
  };
  let mut missing: Vec<usize> = (0..legs.len())
    .filter(|&index| resolved[index].is_none())
    .collect();
  if let Some(near_ms) = work.near_ms {
    missing.sort_by_key(|&index| legs[index].distance_from(near_ms));
  }
  // The nearest leg, and every other one after it, on this decoder; the rest
  // on the second.
  let (mine, theirs): (Vec<_>, Vec<_>) = missing
    .iter()
    .enumerate()
    .partition(|(place, _)| place % 2 == 0);
  let mine: Vec<usize> = mine.into_iter().map(|(_, &index)| index).collect();
  let theirs: Vec<usize> = theirs.into_iter().map(|(_, &index)| index).collect();

  let helped = std::thread::scope(|scope| -> Result<Option<Vec<(usize, Samples)>>, String> {
    let helper = (!theirs.is_empty()).then(|| {
      let (legs, theirs, step) = (&legs, &theirs, &step);
      scope.spawn(move || -> Result<Option<Vec<(usize, Samples)>>, String> {
        let mut second = LumaReader::open(work.recording, work.duration_ms, TRACKING_SIDE)?;
        let mut followed = Vec::with_capacity(theirs.len());
        for &index in theirs {
          let Some(samples) = track_leg(&mut second, &legs[index], work.stop, &mut || step())?
          else {
            return Ok(None);
          };
          let samples = Arc::new(samples);
          store_leg(work.recording, &legs[index], &samples);
          followed.push((index, samples));
        }
        Ok(Some(followed))
      })
    });
    for (place, &index) in mine.iter().enumerate() {
      let Some(samples) = track_leg(reader, &legs[index], work.stop, &mut || step())? else {
        return Ok(None);
      };
      let samples = Arc::new(samples);
      store_leg(work.recording, &legs[index], &samples);
      resolved[index] = Some(samples);
      if place + 1 < mine.len() || !theirs.is_empty() {
        // Whatever the second decoder has finished is in the cache by now.
        for (slot, leg) in resolved.iter_mut().zip(&legs) {
          if slot.is_none() {
            *slot = cached_leg(work.recording, leg);
          }
        }
        let landed: Vec<_> = legs
          .iter()
          .zip(&resolved)
          .filter_map(|(leg, samples)| Some((*leg, samples.as_deref()?.as_slice())))
          .collect();
        partial(&assemble(request, &landed, reader.source_size()));
      }
    }
    match helper {
      Some(helper) => helper
        .join()
        .map_err(|_| "A pinned annotation's second decoder failed".to_owned())?,
      None => Ok(Some(Vec::new())),
    }
  })?;
  let Some(helped) = helped else {
    return Ok(None);
  };
  for (index, samples) in helped {
    resolved[index] = Some(samples);
  }
  let joined: Vec<_> = legs
    .iter()
    .zip(&resolved)
    .filter_map(|(leg, samples)| Some((*leg, samples.as_deref()?.as_slice())))
    .collect();
  if joined.len() < legs.len() {
    return Ok(None);
  }
  let path = Arc::new(assemble(request, &joined, reader.source_size()));
  store_path(work.recording, &path);
  Ok(Some(path))
}

/// Hands each pinned clip among an export's `clips` its path, working out
/// any the preview has not already, from the recording at `recording`. A
/// path that cannot be worked out leaves its annotation where it was drawn.
pub(crate) fn attach_for_export(
  recording: &Path,
  duration_ms: u64,
  clips: &mut [RecordingAnnotationClip],
  cancelled: &AtomicBool,
) {
  let mut reader: Option<LumaReader> = None;
  let work = Work {
    recording,
    duration_ms,
    frames_per_second: 60.0,
    stop: cancelled,
    near_ms: None,
  };
  for clip in clips.iter_mut() {
    if clip.track_id != AnnotationTrack::Primary {
      continue;
    }
    let Some(pin) = clip.pin.as_ref() else {
      continue;
    };
    if reader.is_none() {
      reader = LumaReader::open(recording, duration_ms, TRACKING_SIDE).ok();
    }
    let Some(source) = reader.as_mut() else {
      return;
    };
    let request = PinRequest::of(clip, pin, source.source_size());
    let path = match cached_path(recording, request.key()) {
      Some(path) => Some(path),
      None => work_out(source, &work, &request, &|_| {}, &mut |_| {})
        .ok()
        .flatten(),
    };
    if let Some(pin) = clip.pin.as_mut() {
      pin.path = path;
    }
  }
}
