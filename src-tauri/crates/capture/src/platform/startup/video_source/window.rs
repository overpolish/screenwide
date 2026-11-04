// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a window recording captures: the window's displays, drawing only its
//! app, with the window cut out of them where it stands
//! (`desktop_stream::create_window`).

use super::super::super::*;
use crate::capture_kit::desktop_layout;
use crate::desktop_capture::{corner_radius, window_output, DesktopDisplay, DesktopRect};
use screenwide_records::cursor::{CursorSource, CursorSourceKind};

pub(in crate::platform) struct WindowVideo {
  pub cursor_source: CursorSource,
  /// Every display, in the order of their streams.
  pub displays: Vec<DesktopDisplay>,
  pub fps: u32,
  pub height: u32,
  /// The window's app.
  pub pid: i32,
  pub scale: f32,
  pub show_cursor: bool,
  /// Where the window stands as the recording starts, in global points.
  pub start: DesktopRect,
  pub width: u32,
  pub window_id: u32,
}

pub(super) fn window_video(
  content: &sc::ShareableContent,
  fps: u32,
  show_cursor: bool,
  window_id: u32,
) -> Result<WindowVideo, String> {
  let windows = content.windows();
  let window = windows
    .iter()
    .find(|window| window.id() == window_id)
    .ok_or_else(|| "The selected window is no longer available".to_owned())?;
  let pid = window
    .owning_app()
    .map(|app| app.process_id())
    .ok_or_else(|| "The selected window has no app to record it from".to_owned())?;
  let frame = window.frame();
  let start = DesktopRect {
    x: frame.origin.x,
    y: frame.origin.y,
    width: frame.size.width,
    height: frame.size.height,
  };
  let displays = desktop_layout()?;
  // At the device pixels of the display it starts on, as a window captured
  // on its own is.
  let (centre_x, centre_y) = (start.x + start.width / 2.0, start.y + start.height / 2.0);
  let scale = displays
    .iter()
    .find(|display| {
      (display.x..display.x + display.width).contains(&centre_x)
        && (display.y..display.y + display.height).contains(&centre_y)
    })
    .or_else(|| displays.first())
    .map_or(1.0, |display| display.scale);
  let (width, height) = window_output(start, scale)
    .ok_or_else(|| "The selected window has no usable size".to_owned())?;

  Ok(WindowVideo {
    cursor_source: window_cursor_source(window_id, frame, width, height),
    displays,
    fps,
    height,
    pid,
    scale: scale as f32,
    show_cursor,
    start,
    width,
    window_id,
  })
}

/// How round the window's corners are, in the recording's pixels, read off
/// a still of the window alone: its corners are clear there, where its
/// recording, drawn from the display, has them black. A still carries no
/// sharing badge, unlike a capture of the window kept running.
pub(in crate::platform) async fn corner_radius_of(
  content: &sc::ShareableContent,
  video: &WindowVideo,
) -> Option<f32> {
  let windows = content.windows();
  let window = windows
    .iter()
    .find(|window| window.id() == video.window_id)?;
  let filter = sc::ContentFilter::with_desktop_independent_window(window);
  let mut cfg = sc::StreamCfg::new();
  cfg.set_width(video.width as usize);
  cfg.set_height(video.height as usize);
  cfg.set_pixel_format(cv::PixelFormat::_32_BGRA);
  cfg.set_shows_cursor(false);
  let sample = sc::ScreenshotManager::capture_sample_buf(&filter, &cfg)
    .await
    .ok()?;
  let mut buffer = sample.image_buf()?.retained();
  let flags = cv::pixel_buffer::LockFlags::READ_ONLY;
  let span = 96.min(video.width / 2).min(video.height / 2);
  // SAFETY: the buffer is retained for this call, locked read-only while its
  // bytes are read, and nothing read outlives the lock.
  unsafe {
    buffer.lock_base_addr(flags).result().ok()?;
    let (stride, rows) = (buffer.bytes_per_row(), buffer.height());
    let base = buffer.base_address().cast::<u8>();
    let radius = (!base.is_null() && buffer.width() as u32 >= span && rows as u32 >= span)
      .then(|| {
        let pixels = std::slice::from_raw_parts(base, stride * rows);
        corner_radius(
          |x, y| pixels[y as usize * stride + x as usize * 4 + 3] < 128,
          span,
        )
      })
      .flatten();
    let _ = buffer.unlock_lock_base_addr(flags);
    radius.map(|radius| radius as f32)
  }
}

pub(super) fn window_cursor_source(
  window_id: u32,
  window_frame: cg::Rect,
  video_width: u32,
  video_height: u32,
) -> CursorSource {
  CursorSource {
    height: window_frame.size.height,
    kind: CursorSourceKind::Window,
    platform_id: window_id.to_string(),
    video_height,
    video_width,
    width: window_frame.size.width,
    // CGEvent reports the pointer in global desktop coordinates, as the
    // window's frame is given.
    x: window_frame.origin.x,
    y: window_frame.origin.y,
  }
}
