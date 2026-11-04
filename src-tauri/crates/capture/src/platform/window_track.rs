// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a recorded window stands over time. A window is recorded from its
//! displays and cropped out of each frame (`desktop_capture::window_pieces`),
//! so the crop has to be where the window was when that frame was drawn, not
//! where it is by the time the frame arrives: read late, a window dragged
//! across the screen would jitter inside its own recording. Its bounds are
//! read from the window server every few milliseconds on the host clock the
//! frames carry, and each frame looks up the bounds it was drawn with.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use cidre::{cf, cg, cm};

use super::media::time_to_ns;
use crate::desktop_capture::DesktopRect;

/// Under a frame at 120 fps, so every frame finds bounds read before it.
const POLL: Duration = Duration::from_millis(4);
/// Changes kept, enough for the frames still on their way to look up.
const KEPT: usize = 64;

unsafe extern "C-unwind" {
  fn CGRectMakeWithDictionaryRepresentation(dict: &cf::Dictionary, rect: *mut cg::Rect) -> bool;
}

struct Shared {
  stopped: AtomicBool,
  /// Each change of bounds, with when it was read, oldest first.
  changes: Mutex<VecDeque<(i64, DesktopRect)>>,
}

pub(super) struct WindowTrack {
  shared: Arc<Shared>,
  thread: Option<JoinHandle<()>>,
}

impl WindowTrack {
  pub fn start(window_id: u32, start: DesktopRect) -> Result<Self, String> {
    let shared = Arc::new(Shared {
      stopped: AtomicBool::new(false),
      changes: Mutex::new(VecDeque::from([(i64::MIN, start)])),
    });
    let polling = Arc::clone(&shared);
    let thread = std::thread::Builder::new()
      .name("screenwide-window-track".to_owned())
      .spawn(move || {
        while !polling.stopped.load(Ordering::Acquire) {
          if let (Some(bounds), Some(now)) = (bounds(window_id), host_now_ns()) {
            let mut changes = polling
              .changes
              .lock()
              .unwrap_or_else(PoisonError::into_inner);
            if changes.back().is_none_or(|(_, last)| *last != bounds) {
              if changes.len() == KEPT {
                changes.pop_front();
              }
              changes.push_back((now, bounds));
            }
          }
          std::thread::sleep(POLL);
        }
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      shared,
      thread: Some(thread),
    })
  }

  /// Where the window stood at `ns` on the host clock: the bounds last read
  /// before then, or the oldest kept for a frame older than all of them. A
  /// window that has closed keeps the bounds it last had.
  pub fn at(&self, ns: i64) -> DesktopRect {
    let changes = self
      .shared
      .changes
      .lock()
      .unwrap_or_else(PoisonError::into_inner);
    changes
      .iter()
      .rev()
      .find(|(read, _)| *read <= ns)
      .or_else(|| changes.front())
      .map(|(_, bounds)| *bounds)
      .expect("the track starts with the window's bounds")
  }

  pub fn stop(&mut self) {
    self.shared.stopped.store(true, Ordering::Release);
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }
}

impl Drop for WindowTrack {
  fn drop(&mut self) {
    self.stop();
  }
}

fn host_now_ns() -> Option<i64> {
  time_to_ns(cm::Clock::host_time_clock().time())
}

/// The window's frame in global points, from the top start of the main
/// display, as the displays' own bounds are given; none once it has closed.
pub(crate) fn bounds(window_id: u32) -> Option<DesktopRect> {
  let list = cg::WindowList::info(cg::WindowListOpt::INCLUDING_WINDOW, window_id)?;
  let info = list.iter().next()?;
  let value = info.get(cg::window_keys::bounds())?;
  // The window server gives a window's bounds as a rect's dictionary.
  let dictionary = unsafe { &*std::ptr::from_ref(value).cast::<cf::Dictionary>() };
  let mut rect = cg::Rect::zero();
  if !unsafe { CGRectMakeWithDictionaryRepresentation(dictionary, &raw mut rect) } {
    return None;
  }
  Some(DesktopRect {
    x: rect.origin.x,
    y: rect.origin.y,
    width: rect.size.width,
    height: rect.size.height,
  })
}
