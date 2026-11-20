// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The second model, which adds the band above 8 kHz, run on the CPU while
//! the GPU restores the pieces after. Its recurrent layers step through
//! time one frame after another, so more threads on one piece gain little;
//! two pieces at a time on four threads each finish nearly twice as fast,
//! and keep within two thirds of a current Mac's processor.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{Scope, ScopedJoinHandle};

use ort::session::Session;
use ort::value::TensorRef;

use super::{UPSAMPLE, WHAT};
use crate::session::on_cpu_threads;

const WORKERS: usize = 2;
const THREADS: usize = 4;

/// A piece's restored voice to widen, with the context it is heard with.
pub(super) struct Restored {
  pub(super) index: usize,
  pub(super) voice: Vec<f32>,
  /// Where the piece starts in `voice`, and how long it is.
  pub(super) start: usize,
  pub(super) length: usize,
}

type Widened = (usize, Result<Vec<f32>, String>);

/// Pieces widened on worker threads, handed back in the order they went in.
pub(super) struct Widening<'scope> {
  pieces: Option<SyncSender<Restored>>,
  widened: Receiver<Widened>,
  waiting: BTreeMap<usize, Result<Vec<f32>, String>>,
  next: usize,
  workers: Vec<ScopedJoinHandle<'scope, ()>>,
}

impl<'scope> Widening<'scope> {
  pub(super) fn start<'env>(
    scope: &'scope Scope<'scope, 'env>,
    widener: &Path,
  ) -> Result<Self, String> {
    let sessions = (0..WORKERS)
      .map(|_| on_cpu_threads(widener, WHAT, THREADS))
      .collect::<Result<Vec<_>, _>>()?;
    // Room for one piece more than the workers hold, so the GPU runs ahead
    // without restored pieces piling up in memory.
    let (pieces, queue) = mpsc::sync_channel::<Restored>(1);
    let queue = Arc::new(Mutex::new(queue));
    let (done, widened) = mpsc::channel::<Widened>();
    let workers = sessions
      .into_iter()
      .map(|mut session| {
        let queue = Arc::clone(&queue);
        let done = done.clone();
        scope.spawn(move || loop {
          let next = queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .recv();
          let Ok(piece) = next else { return };
          let voice = widen(&mut session, &piece.voice).map(|mut widened| {
            let start = piece.start * UPSAMPLE;
            widened.resize(start + piece.length * UPSAMPLE, 0.0);
            widened.split_off(start)
          });
          if done.send((piece.index, voice)).is_err() {
            return;
          }
        })
      })
      .collect();
    Ok(Self {
      pieces: Some(pieces),
      widened,
      waiting: BTreeMap::new(),
      next: 0,
      workers,
    })
  }

  pub(super) fn send(&mut self, piece: Restored) -> Result<(), String> {
    self
      .pieces
      .as_ref()
      .and_then(|pieces| pieces.send(piece).ok())
      .ok_or_else(|| format!("{WHAT} stopped widening"))
  }

  /// The next piece in order once it is widened, if it is now, or, with
  /// `wait`, once it is.
  pub(super) fn next(&mut self, wait: bool) -> Option<Result<Vec<f32>, String>> {
    loop {
      if let Some(voice) = self.waiting.remove(&self.next) {
        self.next += 1;
        return Some(voice);
      }
      let received = if wait {
        self.widened.recv().ok()
      } else {
        self.widened.try_recv().ok()
      };
      let (index, voice) = received?;
      self.waiting.insert(index, voice);
    }
  }

  /// No more pieces will come; the workers finish what they hold.
  pub(super) fn close(&mut self) {
    self.pieces = None;
  }
}

impl Drop for Widening<'_> {
  fn drop(&mut self) {
    self.pieces = None;
    for worker in self.workers.drain(..) {
      let _ = worker.join();
    }
  }
}

fn widen(session: &mut Session, restored: &[f32]) -> Result<Vec<f32>, String> {
  let failed = |error: ort::Error| format!("{WHAT} could not run: {error}");
  let input = TensorRef::from_array_view(([1_usize, restored.len()], restored)).map_err(failed)?;
  let outputs = session
    .run(ort::inputs!["restored" => input])
    .map_err(failed)?;
  let (_, widened) = outputs["widened"]
    .try_extract_tensor::<f32>()
    .map_err(failed)?;
  Ok(widened.to_vec())
}
