// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A window recorded from its displays rather than on its own. Capturing the
//! window alone makes macOS mark it as shared, with a badge on it the person
//! sees; capturing a display with every other app left out does not. So
//! each display gets a stream that draws only the window's app, and the
//! window is cut out of their frames where it stands
//! (`desktop_capture::window_pieces`). The window's other windows, menus and
//! sheets are drawn too, over it where they overlap it, as they are seen.

use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::thread::JoinHandle;

use super::*;
use crate::capture_kit::{app_only_filter, current_content_blocking};
use crate::desktop_capture::DesktopRect;
use crate::platform::desktop_compositor::WindowComposition;
use crate::platform::stream_recovery::StreamsHandle;
use crate::platform::window_track::WindowTrack;

/// How often apps opened since the recording started are looked for. One
/// opened in between can show over the window for up to this long.
const REFRESH: Duration = Duration::from_secs(1);
const STOP_CHECK: Duration = Duration::from_millis(100);

pub(in crate::platform) struct WindowStreamRequest<'a> {
  pub commands: SyncSender<Command>,
  pub content: &'a sc::ShareableContent,
  /// Every display, in the order their streams are built.
  pub displays: &'a [DesktopDisplay],
  pub fps: u32,
  pub height: u32,
  /// The window's app.
  pub pid: i32,
  pub queue: &'a dispatch::Queue,
  pub show_cursor: bool,
  /// The window's frame as the recording starts, in global points.
  pub start: DesktopRect,
  pub stats: Arc<CaptureStats>,
  pub watch: &'a StreamWatch,
  pub width: u32,
  pub window_id: u32,
}

pub(in crate::platform) fn create_window(
  request: WindowStreamRequest<'_>,
) -> Result<DesktopStreams, String> {
  let worker = DesktopCompositionWorker::spawn(
    Composition::Window(WindowComposition {
      displays: request.displays.to_vec(),
      height: request.height,
      track: WindowTrack::start(request.window_id, request.start)?,
      width: request.width,
    }),
    request.commands.clone(),
    Arc::clone(&request.stats),
  )?;
  let available = request.content.displays();
  let mut outputs = Vec::with_capacity(request.displays.len());
  let mut streams = Vec::with_capacity(request.displays.len());
  for (source_index, display) in request.displays.iter().enumerate() {
    let captured = available
      .iter()
      .find(|captured| captured.display_id().0 == display.id)
      .ok_or_else(|| "A ScreenCaptureKit display is no longer available".to_owned())?;
    let output = DesktopOutput::with(DesktopOutputInner {
      frames: worker.sender.as_ref().expect("worker is active").clone(),
      source_index,
      stats: Arc::clone(&request.stats),
    });
    let filter = app_only_filter(request.content, captured, request.pid);
    let cfg = whole_display(display, request.fps, request.show_cursor);
    streams.push(
      StreamRecipe::new(&filter, cfg, request.queue)
        .output(
          RecipeOutput::Desktop(output.clone()),
          sc::OutputType::Screen,
        )
        .build(request.watch)?,
    );
    outputs.push(output);
  }
  Ok(DesktopStreams {
    outputs,
    streams,
    worker,
  })
}

/// A display's whole frame at its own pixels, for the window to be cut out
/// of wherever it stands.
fn whole_display(display: &DesktopDisplay, fps: u32, show_cursor: bool) -> arc::R<sc::StreamCfg> {
  let mut cfg = sc::StreamCfg::new();
  cfg.set_width((display.width * display.scale).round() as usize);
  cfg.set_height((display.height * display.scale).round() as usize);
  cfg.set_scales_to_fit(false);
  cfg.set_pixel_format(cv::PixelFormat::_32_BGRA);
  cfg.set_minimum_frame_interval(cm::Time::new(1, fps as cm::TimeScale));
  cfg.set_queue_depth(STREAM_QUEUE_DEPTH);
  cfg.set_shows_cursor(show_cursor);
  cfg.set_color_space_name(cg::color_space::names::srgb());
  cfg
}

/// Leaves out apps opened after a window recording started, which its
/// displays' filters, made at the start, do not name. A stream rebuilt after
/// the Mac slept starts from that first filter too, so it is brought up to
/// date the same way.
pub(in crate::platform) struct WindowFilterRefresh {
  stopped: Arc<AtomicBool>,
  thread: Option<JoinHandle<()>>,
}

impl WindowFilterRefresh {
  /// `streams` holds the displays' streams first, in `display_ids` order.
  pub fn start(streams: StreamsHandle, display_ids: Vec<u32>, pid: i32) -> Result<Self, String> {
    let stopped = Arc::new(AtomicBool::new(false));
    let watching = Arc::clone(&stopped);
    let thread = std::thread::Builder::new()
      .name("screenwide-window-filter".to_owned())
      .spawn(move || {
        let mut applied: Option<(BTreeSet<i32>, Vec<usize>)> = None;
        while wait(&watching) {
          let Some(content) = current_content_blocking() else {
            continue;
          };
          let apps = content
            .apps()
            .iter()
            .map(|app| app.process_id())
            .collect::<BTreeSet<_>>();
          let current = streams
            .current()
            .into_iter()
            .take(display_ids.len())
            .collect::<Vec<_>>();
          let identities = current
            .iter()
            .map(|stream| std::ptr::from_ref::<sc::Stream>(stream).addr())
            .collect::<Vec<_>>();
          if applied.as_ref() == Some(&(apps.clone(), identities.clone())) {
            continue;
          }
          let displays = content.displays();
          for (stream, id) in current.iter().zip(&display_ids) {
            if let Some(display) = displays
              .iter()
              .find(|display| display.display_id().0 == *id)
            {
              stream.update_content_filter_ch(&app_only_filter(&content, display, pid), |_| {});
            }
          }
          applied = Some((apps, identities));
        }
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      stopped,
      thread: Some(thread),
    })
  }

  pub fn stop(&mut self) {
    self.stopped.store(true, Ordering::Release);
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }
}

impl Drop for WindowFilterRefresh {
  fn drop(&mut self) {
    self.stop();
  }
}

/// Waits out one refresh, reporting whether to go on.
fn wait(stopped: &AtomicBool) -> bool {
  let mut waited = Duration::ZERO;
  while waited < REFRESH {
    if stopped.load(Ordering::Acquire) {
      return false;
    }
    std::thread::sleep(STOP_CHECK);
    waited += STOP_CHECK;
  }
  !stopped.load(Ordering::Acquire)
}
