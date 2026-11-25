// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The C boundary of the video export: AVFoundation reads, paces and writes
//! the frames, and calls back here to compose each one and to wait for it.

use std::ffi::{c_char, c_void};
use std::sync::atomic::Ordering;

use super::beside::Beside;
use super::frames::{ExportFrame, FrameComposer};
use super::mux::X264Plan;
use crate::media_preview::StreamedExport;

/// The share of the progress bar the GPU pass owns; muxing takes the rest.
pub(super) const GPU_PROGRESS_PERCENT: u64 = 95;

pub(super) struct ExportContext<'a> {
  pub(super) cancelled: &'a std::sync::atomic::AtomicBool,
  pub(super) duration_ms: u64,
  pub(super) on_progress: &'a mut dyn FnMut(u64),
  pub(super) progress_pace: super::super::super::progress_pace::ProgressPace,
  pub(super) composer: FrameComposer<'a>,
  /// The formats drawn from each decoded frame into buffers of their own.
  pub(super) beside: Vec<Beside>,
  /// Why a frame could not be composed, which outranks the loop's own
  /// report of the frame it then stopped on. Set, it stops the loop.
  pub(super) error: Option<String>,
  /// What x264 is started with once the loop knows the frame rate, and then
  /// the run the frames are written to. With neither, AVFoundation encodes.
  pub(super) x264: Option<X264Plan>,
  pub(super) stream: Option<StreamedExport>,
  /// Where the loop writes its own frames when every format is drawn
  /// beside it, which keeps nothing.
  pub(super) null: Option<std::fs::File>,
}

pub(super) unsafe extern "C" fn should_cancel(context: *mut c_void) -> bool {
  let context = unsafe { &*(context.cast::<ExportContext<'_>>()) };
  context.cancelled.load(Ordering::Acquire) || context.error.is_some()
}

pub(super) unsafe extern "C" fn progress(context: *mut c_void, position_ms: u64) {
  let context = unsafe { &mut *(context.cast::<ExportContext<'_>>()) };
  let position_ms = position_ms.min(context.duration_ms);
  let finishes = position_ms == context.duration_ms;
  if context
    .progress_pace
    .due(std::time::Instant::now(), finishes)
  {
    (context.on_progress)(position_ms.saturating_mul(GPU_PROGRESS_PERCENT) / 100);
  }
}

/// Starts x264 at `frame_rate`, answering the descriptor each frame is
/// written to, or -1 to have AVFoundation encode the frames instead. The
/// formats beside the loop's own start their x264 runs with it; they have
/// no AVFoundation to fall back on.
pub(super) unsafe extern "C" fn open_stream(context: *mut c_void, frame_rate: u32) -> i32 {
  let context = unsafe { &mut *(context.cast::<ExportContext<'_>>()) };
  for beside in &mut context.beside {
    if let Err(error) = beside.start(frame_rate) {
      context.error = Some(error);
      return -1;
    }
  }
  if let Some(null) = &context.null {
    return std::os::fd::AsRawFd::as_raw_fd(null);
  }
  let started = context.x264.take().map(|plan| plan.start(frame_rate));
  let stream = match started {
    Some(Ok(stream)) => stream,
    Some(Err(error)) if !context.beside.is_empty() => {
      context.error = Some(error);
      return -1;
    }
    _ => return -1,
  };
  let Some(input) = stream.input() else {
    return -1;
  };
  context.stream = Some(stream);
  input
}

pub(super) unsafe extern "C" fn compose_frame(
  context: *mut c_void,
  frame: *const ExportFrame,
  token: *mut u64,
) -> bool {
  let context = unsafe { &mut *(context.cast::<ExportContext<'_>>()) };
  let composed = (|| {
    let buffers = context
      .beside
      .iter_mut()
      .map(Beside::next_buffer)
      .collect::<Result<Vec<_>, _>>()?;
    let destinations = context
      .beside
      .iter()
      .zip(&buffers)
      .map(|(beside, &index)| beside.buffer(index))
      .collect::<Vec<_>>();
    let submitted = context
      .composer
      .compose(unsafe { &*frame }, &destinations)?;
    for (beside, index) in context.beside.iter_mut().zip(buffers) {
      beside.composed(submitted, index);
    }
    Ok::<u64, String>(submitted)
  })();
  match composed {
    Ok(submitted) => {
      unsafe { *token = submitted };
      true
    }
    Err(error) => {
      context.error.get_or_insert(error);
      false
    }
  }
}

pub(super) unsafe extern "C" fn wait_frame(context: *mut c_void, token: u64) -> bool {
  let context = unsafe { &mut *(context.cast::<ExportContext<'_>>()) };
  let drawn = context.composer.wait(token).and_then(|()| {
    context
      .beside
      .iter_mut()
      .try_for_each(|beside| beside.drawn(token))
  });
  match drawn {
    Ok(()) => true,
    Err(error) => {
      context.error.get_or_insert(error);
      false
    }
  }
}

unsafe extern "C" {
  pub(super) fn screenwide_video_export(
    screen_path: *const c_char,
    camera_path: *const c_char,
    output_path: *const c_char,
    timeline_ranges: *const screenwide_timeline::TimelineRange,
    timeline_range_count: u32,
    width: u32,
    height: u32,
    bitrate: u64,
    context: *mut c_void,
    should_cancel: unsafe extern "C" fn(*mut c_void) -> bool,
    progress: unsafe extern "C" fn(*mut c_void, u64),
    compose: unsafe extern "C" fn(*mut c_void, *const ExportFrame, *mut u64) -> bool,
    wait: unsafe extern "C" fn(*mut c_void, u64) -> bool,
    open_stream: unsafe extern "C" fn(*mut c_void, u32) -> i32,
    error_text: *mut c_char,
    error_capacity: usize,
  ) -> i32;
}
