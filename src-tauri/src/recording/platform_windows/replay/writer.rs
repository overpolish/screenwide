// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The replay buffer's writer thread: the recording writer's timing, with the
//! movie replaced by a ring of encoded frames.
//!
//! The timeline rules are the recording writer's, so a clip lines up with its
//! cursor, keyboard, annotations and audio exactly as a recording does: the
//! track that sets the shared origin stamps it with its first frame, and
//! every other track anchors to it. A replay is never paused.
//!
//! Frames are encoded at no more than the capture's frame rate. A frame that
//! arrives sooner than that after the last one waits, converted, until it is
//! due, and is replaced if a newer one arrives first, so the newest picture
//! is always the one encoded and a fast monitor costs no extra encoding.

mod take;
mod video;

use std::ops::ControlFlow;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, TryRecvError};
use std::time::{Duration, Instant};

use super::super::writer::{Command, Frame, MediaFoundation, WriterConfig, NANOS_PER_100NS};
use super::convert::Converter;
use super::encoder::{Encoder, Size};
use super::{EncodedFrame, KeyframeLink};
use crate::recording::encoding::Timeline;
use crate::recording::replay::ring::VideoRing;

/// How long a screen may sit unchanged before its last frame is encoded
/// again as a keyframe. A screen that does not change sends no frames, and a
/// clip can only start on a keyframe; without this a clip of a quiet screen
/// would reach back to whenever it last changed.
const KEYFRAME_REFRESH: Duration = Duration::from_secs(1);
/// The least time between two encoded frames, so a frame encoded again at a
/// save or on a quiet screen never shares an instant with a captured one.
const MIN_FRAME_GAP_NS: i64 = 2_000_000;
/// How often the writer looks for frames the encoder has finished while it
/// still holds some.
const OUTPUT_POLL: Duration = Duration::from_millis(5);

pub(in crate::recording::platform_windows) struct ReplayWriterConfig {
  pub keyframes: KeyframeLink,
  /// Whether this writer chooses where clips start, and publishes its
  /// keyframes for the others to follow.
  pub leads: bool,
  pub length: Duration,
  pub writer: WriterConfig,
}

/// A converted frame waiting until it is due to be encoded.
struct Staged {
  pts_ns: i64,
  slot: usize,
}

struct ReplayWriter {
  base: Instant,
  config: WriterConfig,
  converter: Converter,
  /// What the encoder handed back and the ring has not yet taken.
  encoded: Vec<EncodedFrame>,
  encoder: Encoder,
  failed: Option<String>,
  /// The primary keyframe a follower last matched.
  followed_keyframe_ns: i64,
  /// A follower's next frame is a keyframe, so the next clip can start on it.
  force_keyframe: bool,
  frame_duration_100ns: i64,
  frames: VideoRing<std::sync::Arc<[u8]>>,
  keyframes: KeyframeLink,
  last_encoded_at: Option<Instant>,
  last_encoded_ns: Option<i64>,
  last_frame_at: Option<Instant>,
  last_refresh_at: Option<Instant>,
  leads: bool,
  min_gap: Duration,
  next_slot: usize,
  staged: Option<Staged>,
  /// The slot holding the last picture encoded, kept for a refresh or a save.
  tail: Option<usize>,
  timeline: Timeline,
}

fn duration_ns(duration: Duration) -> i64 {
  i64::try_from(duration.as_nanos()).unwrap_or(i64::MAX)
}

pub(in crate::recording::platform_windows) fn run(
  config: ReplayWriterConfig,
  commands: Receiver<Command>,
  initialized: mpsc::Sender<Result<(), String>>,
  first_frame: mpsc::Sender<Result<(), String>>,
) {
  let started =
    MediaFoundation::start().and_then(|runtime| Ok((runtime, ReplayWriter::new(config)?)));
  let (_media_foundation, mut writer) = match started {
    Ok(started) => started,
    Err(error) => {
      let _ = initialized.send(Err(error.clone()));
      let _ = first_frame.send(Err(error));
      return;
    }
  };
  let _ = initialized.send(Ok(()));
  let mut announced = false;
  let mut pending = None;
  loop {
    let command = match pending.take() {
      Some(command) => Some(command),
      None => match commands.recv_timeout(writer.next_wake()) {
        Ok(command) => Some(command),
        Err(RecvTimeoutError::Timeout) => None,
        Err(RecvTimeoutError::Disconnected) => return,
      },
    };
    if let Some(command) = command {
      let command = match command {
        // Only the newest of the frames queued up matters: an older one
        // would be replaced before it was due anyway.
        Command::Frame(mut frame) => loop {
          match commands.try_recv() {
            Ok(Command::Frame(newer)) => frame = newer,
            Ok(other) => {
              pending = Some(other);
              break Command::Frame(frame);
            }
            Err(TryRecvError::Empty) => break Command::Frame(frame),
            Err(TryRecvError::Disconnected) => return,
          }
        },
        command => command,
      };
      if writer
        .handle(command, &first_frame, &mut announced)
        .is_break()
      {
        return;
      }
    }
    writer.service();
  }
}

impl ReplayWriter {
  fn new(config: ReplayWriterConfig) -> Result<Self, String> {
    let ReplayWriterConfig {
      keyframes,
      leads,
      length,
      writer,
    } = config;
    let size = Size {
      width: writer.width,
      height: writer.height,
      fps: writer.fps.max(1),
    };
    let encoder = Encoder::new(&writer.device, size)?;
    let converter = Converter::new(
      &writer.device,
      size.width,
      size.height,
      size.fps,
      writer.source_crop,
      encoder.input_bind_flags(),
    )?;
    let cadence = Duration::from_nanos(1_000_000_000 / u64::from(size.fps));
    Ok(Self {
      base: Instant::now(),
      converter,
      encoded: Vec::new(),
      encoder,
      failed: None,
      followed_keyframe_ns: i64::MIN,
      force_keyframe: false,
      frame_duration_100ns: 10_000_000 / i64::from(size.fps),
      frames: VideoRing::new(duration_ns(length)),
      keyframes,
      last_encoded_at: None,
      last_encoded_ns: None,
      last_frame_at: None,
      last_refresh_at: None,
      leads,
      // A little under one frame, so capture jitter never drops a frame
      // that arrives on time.
      min_gap: cadence * 3 / 4,
      next_slot: 0,
      staged: None,
      tail: None,
      timeline: Timeline::default(),
      config: writer,
    })
  }

  fn elapsed_ns(&self, at: Instant) -> i64 {
    i64::try_from(at.saturating_duration_since(self.base).as_nanos()).unwrap_or(i64::MAX)
  }

  fn handle(
    &mut self,
    command: Command,
    first_frame: &mpsc::Sender<Result<(), String>>,
    announced: &mut bool,
  ) -> ControlFlow<()> {
    match command {
      Command::Frame(frame) => {
        if self.frame(frame) && !*announced {
          *announced = true;
          let _ = first_frame.send(Ok(()));
        } else if !*announced {
          if let Some(error) = self.failed.clone() {
            let _ = first_frame.send(Err(error));
            return ControlFlow::Break(());
          }
        }
      }
      Command::Replay(request) => {
        let result = self.take(request.at, request.from);
        let _ = request.reply.send(result);
      }
      // A replay is never paused; it is stopped by cancelling.
      Command::Pause(_) | Command::Resume(_) => {}
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
    eprintln!("Replay buffer stopped keeping video: {reason}");
    (self.config.on_failure)(reason.clone());
    self.failed = Some(reason);
  }
}

impl Frame {
  fn source_ns(&self) -> i64 {
    self.source_100ns.saturating_mul(NANOS_PER_100NS)
  }
}
