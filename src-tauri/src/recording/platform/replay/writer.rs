// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The replay buffer's writer thread: the recording writer's timing, with the
//! movie replaced by the rings in [`crate::recording::replay::ring`].
//!
//! The timeline rules are the recording writer's, so a clip lines up with its
//! cursor, keyboard and annotations exactly as a recording does: the primary
//! video's first frame stamps the shared origin, a camera beside it anchors to
//! that origin, and audio is mapped onto the same timeline before it is kept.
//! A replay is never paused.

mod audio;
mod take;
mod video;

use std::collections::VecDeque;
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use cidre::objc;

use super::super::media::{audio_sample_from_origin, microphone_buffer_from_origin, VideoEncoder};
use super::super::output::{AudioSample, CaptureStats, Command, Frame, FrameClock};
use super::super::{
  MICROPHONE_PREROLL_LIMIT, SYSTEM_AUDIO_CHANNELS, SYSTEM_AUDIO_PREROLL_LIMIT,
  SYSTEM_AUDIO_SAMPLE_RATE,
};
use super::encoder::Encoder;
use super::SharedSample;
use crate::recording::encoding::{FailureReport, Timeline};
use crate::recording::microphone::{Buffer as MicrophoneBuffer, Format as MicrophoneFormat};
use crate::recording::replay::ring::{AudioChunk, AudioRing, VideoRing};

/// How long a screen may sit unchanged before its last frame is encoded
/// again as a keyframe. A screen that does not change sends no frames, and a
/// clip can only start on a keyframe; without this a clip of a quiet screen
/// would reach back to whenever it last changed.
const KEYFRAME_REFRESH: Duration = Duration::from_secs(1);
/// What audio is kept beyond the buffer's length: a clip starts on a video
/// keyframe, which can sit up to a refresh before the full length.
const AUDIO_MARGIN: Duration = Duration::from_secs(5);
/// The least time between two encoded frames. A saved clip's video track is
/// timed in QuickTime's 600ths of a second, so two frames closer than one of
/// those would land on the same instant there. Only the frames this writer
/// encodes again - at a save and on a quiet screen - can come that close to
/// a captured one, and moving either by two milliseconds is invisible.
const MIN_FRAME_GAP_NS: i64 = 2_000_000;

/// The newest keyframe the primary video wrote, so a camera beside it can
/// put one of its own right after and a clip can start both together.
#[derive(Clone)]
pub(in crate::recording::platform) struct KeyframeLink(Arc<AtomicI64>);

impl KeyframeLink {
  pub(in crate::recording::platform) fn new() -> Self {
    Self(Arc::new(AtomicI64::new(i64::MIN)))
  }
}

pub(in crate::recording::platform) struct ReplayWriterConfig {
  /// The encoder for the video this writer keeps; `None` for audio only.
  pub encoder: Option<VideoEncoder>,
  pub fps: u32,
  pub height: u32,
  /// The primary keyframes: published by the primary video writer, followed
  /// by a camera writer beside it.
  pub keyframes: KeyframeLink,
  pub length: Duration,
  pub microphone_format: Option<MicrophoneFormat>,
  pub on_failure: FailureReport,
  /// Whether this writer's first frame stamps the shared origin.
  pub primary_video: bool,
  pub stats: Arc<CaptureStats>,
  pub system_audio: bool,
  pub timeline_origin: Arc<OnceLock<Instant>>,
  pub width: u32,
}

pub(in crate::recording::platform) struct ReplayWriter {
  base: Instant,
  encoder: Option<Encoder>,
  failed: Option<String>,
  /// The primary keyframe a follower last matched.
  followed_keyframe_ns: i64,
  /// A follower's next frame is a keyframe, so the next clip can start on it.
  force_keyframe: bool,
  frames: Arc<Mutex<VideoRing<SharedSample>>>,
  height: u32,
  keyframes: KeyframeLink,
  last_encoded_ns: Option<i64>,
  last_frame_at: Option<Instant>,
  last_refresh_at: Option<Instant>,
  microphone: Option<(MicrophoneFormat, AudioRing)>,
  on_failure: FailureReport,
  origin_source_ns: Option<i64>,
  origin_wall: Option<Instant>,
  pending_microphone: VecDeque<MicrophoneBuffer>,
  pending_system_audio: VecDeque<AudioSample>,
  primary_video: bool,
  stats: Arc<CaptureStats>,
  system_audio: Option<AudioRing>,
  tail: Option<Frame>,
  timeline: Timeline,
  timeline_origin: Arc<OnceLock<Instant>>,
  width: u32,
}

fn duration_ns(duration: Duration) -> i64 {
  i64::try_from(duration.as_nanos()).unwrap_or(i64::MAX)
}

impl ReplayWriter {
  pub(in crate::recording::platform) fn new(config: ReplayWriterConfig) -> Result<Self, String> {
    let length_ns = duration_ns(config.length);
    let audio_horizon_ns = duration_ns(config.length + AUDIO_MARGIN);
    let frames = Arc::new(Mutex::new(VideoRing::new(length_ns)));
    // Only the primary publishes; a follower's own keyframes go nowhere.
    let published = if config.primary_video {
      config.keyframes.clone()
    } else {
      KeyframeLink::new()
    };
    let encoder = config
      .encoder
      .map(|codec| {
        Encoder::new(
          config.width,
          config.height,
          config.fps,
          codec,
          Arc::clone(&frames),
          published.0,
        )
      })
      .transpose()?;
    Ok(Self {
      base: Instant::now(),
      encoder,
      failed: None,
      followed_keyframe_ns: i64::MIN,
      force_keyframe: false,
      frames,
      height: config.height,
      keyframes: config.keyframes,
      last_encoded_ns: None,
      last_frame_at: None,
      last_refresh_at: None,
      microphone: config.microphone_format.map(|format| {
        (
          format,
          AudioRing::new(format.channels, format.sample_rate, audio_horizon_ns),
        )
      }),
      on_failure: config.on_failure,
      origin_source_ns: None,
      origin_wall: None,
      pending_microphone: VecDeque::new(),
      pending_system_audio: VecDeque::new(),
      primary_video: config.primary_video,
      stats: config.stats,
      system_audio: config.system_audio.then(|| {
        AudioRing::new(
          SYSTEM_AUDIO_CHANNELS as u16,
          SYSTEM_AUDIO_SAMPLE_RATE as u32,
          audio_horizon_ns,
        )
      }),
      tail: None,
      timeline: Timeline::default(),
      timeline_origin: config.timeline_origin,
      width: config.width,
    })
  }

  fn elapsed_ns(&self, at: Instant) -> i64 {
    i64::try_from(at.saturating_duration_since(self.base).as_nanos()).unwrap_or(i64::MAX)
  }

  pub(in crate::recording::platform) fn run(
    mut self,
    commands: &Receiver<Command>,
    first_frame: &mpsc::Sender<Result<(), String>>,
  ) {
    let mut announced = false;
    loop {
      let flow = match commands.recv_timeout(KEYFRAME_REFRESH) {
        // One autorelease pool per command, as the recording writer does:
        // this thread has none of its own, and every encode leaves scratch.
        Ok(command) => objc::ar_pool(|| self.handle(command, first_frame, &mut announced)),
        Err(RecvTimeoutError::Timeout) => ControlFlow::Continue(()),
        Err(RecvTimeoutError::Disconnected) => ControlFlow::Break(()),
      };
      if flow.is_break() {
        return;
      }
      objc::ar_pool(|| self.refresh_keyframe());
    }
  }

  fn handle(
    &mut self,
    command: Command,
    first_frame: &mpsc::Sender<Result<(), String>>,
    announced: &mut bool,
  ) -> ControlFlow<()> {
    match command {
      Command::Begin { at } => {
        // Audio-only: the buffer's own start is time zero.
        self.origin_wall = Some(at);
        self.timeline.start_at(0, self.elapsed_ns(at));
        self.flush_preroll();
        let _ = first_frame.send(Ok(()));
        *announced = true;
      }
      Command::Frame(frame) => {
        if self.frame(frame) && !*announced {
          *announced = true;
          let _ = first_frame.send(Ok(()));
        }
      }
      Command::SystemAudio(sample) => {
        if self.timeline.has_started() {
          self.system_audio(sample);
        } else {
          if self.pending_system_audio.len() == SYSTEM_AUDIO_PREROLL_LIMIT {
            self.pending_system_audio.pop_front();
          }
          self.pending_system_audio.push_back(sample);
        }
      }
      Command::Microphone(buffer) => {
        if self.timeline.has_started() {
          self.microphone(buffer);
        } else {
          if self.pending_microphone.len() == MICROPHONE_PREROLL_LIMIT {
            self.pending_microphone.pop_front();
          }
          self.pending_microphone.push_back(buffer);
        }
      }
      Command::MicrophoneFailed(error) => {
        self.fail(format!("The microphone stopped recording: {error}"));
      }
      Command::Replay(request) => {
        let result = self.take(request.at, request.from);
        let _ = request.reply.send(result);
      }
      // A replay is never paused; it is stopped by cancelling.
      Command::Pause { .. } | Command::Resume { .. } => {}
      Command::Stop { reply, .. } => {
        let _ = reply.send(Err("The replay buffer does not finish a movie".to_owned()));
        return ControlFlow::Break(());
      }
      Command::Cancel => return ControlFlow::Break(()),
    }
    ControlFlow::Continue(())
  }

  fn fail(&mut self, reason: String) {
    if self.failed.is_some() {
      return;
    }
    eprintln!("Replay buffer stopped keeping media: {reason}");
    (self.on_failure)(reason.clone());
    self.failed = Some(reason);
  }
}
