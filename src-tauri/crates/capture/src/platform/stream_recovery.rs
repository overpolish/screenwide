// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Capture streams that come back after macOS stops them.
//!
//! macOS stops a capture's ScreenCaptureKit streams when the Mac locks or
//! sleeps, and does not start them again. Nothing downstream can tell: the
//! writer holds the last frame, so a recording or replay that slept stays on
//! that frame for the rest of its length while the cursor, microphone and
//! camera carry on. Each stream is therefore kept with the recipe it was built
//! from, and one that stops with an error is rebuilt and restarted until it
//! runs again, which succeeds once the user is back at the desktop. The new
//! stream feeds the same outputs on the same host clock, so the timeline
//! carries straight on and the time on the lock screen holds the last frame.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use cidre::sc::stream::{Delegate, DelegateImpl};
use cidre::{arc, define_obj_type, dispatch, ns, objc, sc};

use super::desktop_stream::DesktopOutput;
use super::output::ScreenOutput;

/// How often a stopped stream is tried again. While the Mac is still locked
/// the start is refused; once it is unlocked the stream is back within this.
const RETRY_INTERVAL: Duration = Duration::from_secs(1);
const START_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) enum RecipeOutput {
  Desktop(arc::R<DesktopOutput>),
  Screen(arc::R<ScreenOutput>),
}

/// Everything a stream was built from, to build it again.
pub(super) struct StreamRecipe {
  cfg: arc::R<sc::StreamCfg>,
  filter: arc::R<sc::ContentFilter>,
  outputs: Vec<(RecipeOutput, sc::OutputType)>,
  queue: arc::R<dispatch::Queue>,
}

// SAFETY: the filter, configuration and queue are not changed after the
// recipe is made, and the outputs' state is only touched on their own serial
// queue. Other threads only retain, release and pass them to
// ScreenCaptureKit, and Objective-C reference counting is atomic.
unsafe impl Send for StreamRecipe {}
unsafe impl Sync for StreamRecipe {}

impl StreamRecipe {
  pub(super) fn new(
    filter: &sc::ContentFilter,
    cfg: arc::R<sc::StreamCfg>,
    queue: &dispatch::Queue,
  ) -> Self {
    Self {
      cfg,
      filter: filter.retained(),
      outputs: Vec::new(),
      queue: queue.retained(),
    }
  }

  pub(super) fn output(mut self, output: RecipeOutput, kind: sc::OutputType) -> Self {
    self.outputs.push((output, kind));
    self
  }

  /// Builds the stream, reporting a stop to `watch`.
  pub(super) fn build(self, watch: &StreamWatch) -> Result<WatchedStream, String> {
    let recipe = Arc::new(self);
    let stream = recipe.stream(watch)?;
    Ok(WatchedStream { recipe, stream })
  }

  fn stream(&self, watch: &StreamWatch) -> Result<arc::R<sc::Stream>, String> {
    let stream = sc::Stream::with_delegate(&self.filter, &self.cfg, watch);
    for (output, kind) in &self.outputs {
      match output {
        RecipeOutput::Desktop(output) => {
          stream.add_stream_output(output.as_ref(), *kind, Some(&self.queue))
        }
        RecipeOutput::Screen(output) => {
          stream.add_stream_output(output.as_ref(), *kind, Some(&self.queue))
        }
      }
      .map_err(|error| error.to_string())?;
    }
    Ok(stream)
  }
}

/// A stream and the recipe to rebuild it from.
pub(super) struct WatchedStream {
  recipe: Arc<StreamRecipe>,
  pub stream: arc::R<sc::Stream>,
}

pub(super) struct StreamWatchInner {
  stopped: Sender<arc::R<sc::Stream>>,
}

define_obj_type!(
  pub(super) StreamWatch + DelegateImpl,
  StreamWatchInner,
  STREAM_WATCH_CLS
);

impl Delegate for StreamWatch {}

#[objc::add_methods]
impl DelegateImpl for StreamWatch {
  extern "C" fn impl_stream_did_stop_with_err(
    &mut self,
    _command: Option<&objc::Sel>,
    stream: &sc::Stream,
    error: &ns::Error,
  ) {
    eprintln!(
      "ScreenCaptureKit stopped a capture stream ({}): {error}",
      error.code()
    );
    let _ = self.inner_mut().stopped.send(stream.retained());
  }
}

/// The delegate every stream of one capture reports to, and where it reports.
pub(super) fn watch() -> (arc::R<StreamWatch>, Receiver<arc::R<sc::Stream>>) {
  let (stopped, reports) = mpsc::channel();
  (StreamWatch::with(StreamWatchInner { stopped }), reports)
}

struct Shared {
  halted: AtomicBool,
  streams: Mutex<Vec<WatchedStream>>,
}

/// A capture's streams, kept running through sleep and lock.
pub(super) struct RecoveringStreams {
  shared: Arc<Shared>,
  worker: Option<JoinHandle<()>>,
}

impl RecoveringStreams {
  pub(super) fn new(
    watch: arc::R<StreamWatch>,
    reports: Receiver<arc::R<sc::Stream>>,
    streams: Vec<WatchedStream>,
  ) -> Self {
    let shared = Arc::new(Shared {
      halted: AtomicBool::new(false),
      streams: Mutex::new(streams),
    });
    let recovering = Arc::clone(&shared);
    let watch = SendWatch(watch);
    let worker = std::thread::Builder::new()
      .name("screenwide-capture-recovery".to_owned())
      .spawn(move || {
        let watch = watch;
        recover(&recovering, &watch.0, &reports);
      })
      .ok();
    Self { shared, worker }
  }

  /// The streams as they are now, for stopping them.
  pub(super) fn current(&self) -> Vec<arc::R<sc::Stream>> {
    self
      .shared
      .streams
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .iter()
      .map(|watched| watched.stream.clone())
      .collect()
  }

  /// Stops restarting streams, so the capture's own stop is final.
  pub(super) fn halt(&mut self) {
    self.shared.halted.store(true, Ordering::Release);
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
  }
}

impl Drop for RecoveringStreams {
  fn drop(&mut self) {
    self.halt();
  }
}

struct SendWatch(arc::R<StreamWatch>);

// SAFETY: the delegate's only state is a channel sender, which is `Send`;
// the object itself is only retained, released and handed to new streams.
unsafe impl Send for SendWatch {}

fn recover(shared: &Shared, watch: &StreamWatch, reports: &Receiver<arc::R<sc::Stream>>) {
  let mut stopped: Vec<Arc<StreamRecipe>> = Vec::new();
  while !shared.halted.load(Ordering::Acquire) {
    match reports.recv_timeout(RETRY_INTERVAL) {
      Ok(stream) => {
        let streams = shared
          .streams
          .lock()
          .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(watched) = streams
          .iter()
          .find(|watched| std::ptr::eq(watched.stream.as_ref(), stream.as_ref()))
        {
          if !stopped
            .iter()
            .any(|recipe| Arc::ptr_eq(recipe, &watched.recipe))
          {
            stopped.push(Arc::clone(&watched.recipe));
          }
        }
      }
      Err(RecvTimeoutError::Timeout) => {}
      Err(RecvTimeoutError::Disconnected) => return,
    }
    stopped.retain(|recipe| !restart(shared, watch, recipe));
  }
}

/// Builds and starts `recipe`'s stream again, reporting whether it is running.
fn restart(shared: &Shared, watch: &StreamWatch, recipe: &Arc<StreamRecipe>) -> bool {
  if shared.halted.load(Ordering::Acquire) {
    return true;
  }
  let Ok(stream) = recipe.stream(watch) else {
    return false;
  };
  let (started, start) = mpsc::channel();
  stream.start_with_ch(move |error| {
    let _ = started.send(error.is_none());
  });
  if start.recv_timeout(START_TIMEOUT) != Ok(true) {
    return false;
  }
  let mut streams = shared
    .streams
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if shared.halted.load(Ordering::Acquire) {
    stream.stop_with_ch(|_| {});
    return true;
  }
  if let Some(watched) = streams
    .iter_mut()
    .find(|watched| Arc::ptr_eq(&watched.recipe, recipe))
  {
    watched.stream = stream;
    eprintln!("Restarted a capture stream that ScreenCaptureKit had stopped");
  }
  true
}
