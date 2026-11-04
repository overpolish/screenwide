// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recordings composed from whole displays' streams: a region crossing
//! displays, and a window followed across them.

use super::super::desktop_stream::{
  self, DesktopKeepalive, DesktopStreams, WindowFilterRefresh, WindowStreamRequest,
};
use super::super::stream_recovery::StreamsHandle;
use super::super::*;
use super::video_source::ResolvedVideo;

pub(super) struct DesktopVideoRequest<'a> {
  /// Where the whole Mac's sound goes, when a region's stream carries it.
  pub audio_output: Option<&'a arc::R<ScreenOutput>>,
  pub commands: &'a SyncSender<Command>,
  pub content: Option<&'a sc::ShareableContent>,
  pub include_own_windows: bool,
  pub queue: &'a dispatch::Queue,
  pub stats: &'a Arc<CaptureStats>,
  pub watch: &'a StreamWatch,
}

/// The streams for a composed recording, or none for any other.
pub(super) fn create(
  video: Option<&ResolvedVideo>,
  request: DesktopVideoRequest<'_>,
) -> Result<Option<DesktopStreams>, String> {
  match video {
    Some(ResolvedVideo::Desktop(video)) => {
      desktop_stream::create(desktop_stream::DesktopStreamRequest {
        audio_output: request.audio_output,
        commands: request.commands.clone(),
        content: request.content.expect("desktop video has content"),
        displays: &video.displays,
        fps: video.fps,
        include_own_windows: request.include_own_windows,
        plan: &video.plan,
        queue: request.queue,
        show_cursor: video.show_cursor,
        stats: Arc::clone(request.stats),
        watch: request.watch,
      })
      .map(Some)
    }
    Some(ResolvedVideo::Window(video)) => desktop_stream::create_window(WindowStreamRequest {
      commands: request.commands.clone(),
      content: request.content.expect("window video has content"),
      displays: &video.displays,
      fps: video.fps,
      height: video.height,
      pid: video.pid,
      queue: request.queue,
      show_cursor: video.show_cursor,
      start: video.start,
      stats: Arc::clone(request.stats),
      watch: request.watch,
      width: video.width,
      window_id: video.window_id,
    })
    .map(Some),
    _ => Ok(None),
  }
}

/// Keeps a window recording's displays leaving out apps opened once it has
/// started; its streams are the first of `streams`.
pub(super) fn follow_new_apps(
  video: Option<&ResolvedVideo>,
  desktop: Option<&mut DesktopKeepalive>,
  streams: StreamsHandle,
) -> Result<(), String> {
  if let (Some(ResolvedVideo::Window(video)), Some(desktop)) = (video, desktop) {
    let display_ids = video.displays.iter().map(|display| display.id).collect();
    desktop.follow_new_apps(WindowFilterRefresh::start(streams, display_ids, video.pid)?);
  }
  Ok(())
}
