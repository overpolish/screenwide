// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;
use super::video_source::PrimaryVideo;

pub(super) struct VideoStreamRequest<'a> {
  pub captures_audio: bool,
  pub output: &'a arc::R<ScreenOutput>,
  pub queue: &'a dispatch::Queue,
  pub video: &'a PrimaryVideo,
  pub watch: &'a StreamWatch,
}

pub(super) fn create_video(request: VideoStreamRequest<'_>) -> Result<WatchedStream, String> {
  let VideoStreamRequest {
    captures_audio,
    output,
    queue,
    video,
    watch,
  } = request;
  let mut cfg = sc::StreamCfg::new();
  cfg.set_width(video.width as usize);
  cfg.set_height(video.height as usize);
  cfg.set_scales_to_fit(true);
  if video.is_window {
    // A desktop-independent filter follows the window between displays. This
    // keeps pixels outside the current display edge instead of clipping them.
    cfg.set_ignore_global_clip_single_window(true);
  }
  if let Some(rect) = video.source_rect {
    cfg.set_src_rect(rect);
  }
  cfg.set_pixel_format(cv::PixelFormat::_420V);
  cfg.set_minimum_frame_interval(cm::Time::new(1, video.fps as cm::TimeScale));
  cfg.set_queue_depth(STREAM_QUEUE_DEPTH);
  cfg.set_shows_cursor(video.show_cursor);
  cfg.set_captures_audio(captures_audio);
  if captures_audio {
    configure_system_audio(&mut cfg);
  }
  cfg.set_color_space_name(cg::color_space::names::srgb());

  let mut recipe = StreamRecipe::new(&video.filter, cfg, queue)
    .output(RecipeOutput::Screen(output.clone()), sc::OutputType::Screen);
  if captures_audio {
    recipe = recipe.output(RecipeOutput::Screen(output.clone()), sc::OutputType::Audio);
  }
  recipe.build(watch)
}

pub(super) struct AllAudioStreamRequest<'a> {
  pub content: &'a sc::ShareableContent,
  pub display: &'a sc::Display,
  pub output: &'a arc::R<ScreenOutput>,
  pub queue: &'a dispatch::Queue,
  pub watch: &'a StreamWatch,
}

pub(super) fn create_all_audio(
  request: AllAudioStreamRequest<'_>,
) -> Result<WatchedStream, String> {
  let AllAudioStreamRequest {
    content,
    display,
    output,
    queue,
    watch,
  } = request;
  let filter = sc::ContentFilter::with_display_excluding_windows(display, &our_windows(content));
  let mut cfg = sc::StreamCfg::new();
  cfg.set_captures_audio(true);
  configure_system_audio(&mut cfg);
  StreamRecipe::new(&filter, cfg, queue)
    .output(RecipeOutput::Screen(output.clone()), sc::OutputType::Audio)
    .build(watch)
}
