// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use super::super::held_timelines::ready;
use super::super::pin_paths::{work_out, PinStatus, Work};
use super::super::platform::LumaReader;
use super::{Job, PinTracks, PROGRESS_STEP};
use crate::editor::annotations::pin::partial::patched;
use crate::editor::annotations::pin::{PinnedPath, TRACKING_SIDE};

impl PinTracks {
  pub(super) fn work(&self, first: (Job, Arc<AtomicBool>)) {
    let mut reader = match LumaReader::open(&self.recording, self.duration_ms, TRACKING_SIDE) {
      Ok(reader) => Some(reader),
      Err(error) => {
        eprintln!("Pinned annotations cannot read the recording: {error}");
        None
      }
    };
    let mut job = first;
    loop {
      let (
        Job {
          id,
          request,
          near_ms,
        },
        stop,
      ) = job;
      // The last path's stretches no longer say anything about this one.
      self.report(PinStatus {
        annotation_id: id.clone(),
        progress: Some(0.0),
        weak: Vec::new(),
        hidden: Vec::new(),
      });
      let reported = Mutex::new(0.0_f32);
      let progress = |share: f32| {
        let Ok(mut reported) = reported.lock() else {
          return;
        };
        if share - *reported >= PROGRESS_STEP {
          *reported = share;
          self.report(PinStatus {
            annotation_id: id.clone(),
            progress: Some(share),
            weak: Vec::new(),
            hidden: Vec::new(),
          });
        }
      };
      // Each leg that lands is shown at once, over the path the annotation
      // had where the rest is still to come. It is shown, not kept: its own
      // key keeps what is read along it apart from the finished path's.
      let mut partials = 0_u64;
      let mut partial = |landed: &PinnedPath| {
        partials += 1;
        let Ok(mut state) = self.state.lock() else {
          return;
        };
        let stale = state.latest.get(&id).cloned().unwrap_or_default();
        let shown = patched(
          landed,
          &stale,
          request.start_ms,
          request.end_ms,
          landed.key.wrapping_add(partials),
        );
        state.latest.insert(id.clone(), Arc::new(shown));
        drop(state);
        ready(&self.on_ready);
      };
      let work = Work {
        recording: &self.recording,
        duration_ms: self.duration_ms,
        frames_per_second: self.frames_per_second,
        stop: &stop,
        near_ms: Some(near_ms),
      };
      let path = reader.as_mut().and_then(|reader| {
        match work_out(reader, &work, &request, &progress, &mut partial) {
          Ok(path) => path,
          Err(error) => {
            eprintln!("A pinned annotation could not be tracked: {error}");
            None
          }
        }
      });
      if let Some(path) = &path {
        self.landed(&id, path);
      }
      let next = {
        let Ok(mut state) = self.state.lock() else {
          return;
        };
        Self::next(&mut state)
      };
      if path.is_some() {
        ready(&self.on_ready);
      }
      let Some(next) = next else {
        return;
      };
      job = next;
    }
  }
}
