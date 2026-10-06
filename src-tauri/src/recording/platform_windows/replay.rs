// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The replay buffer on Windows: the capture a recording opens, feeding
//! writers that keep the last stretch of media in memory instead of filling
//! a movie.
//!
//! Each frame is converted to NV12 on the GPU and encoded as it arrives by
//! an H.264 encoder transform the writer drives itself, and the encoded
//! frames are kept back to the keyframe a clip of the full length would have
//! to start from. Audio is kept as the PCM it arrived as, beside the capture
//! that produced it, and encoded only for the clips that are saved.
//!
//! Saving asks each writer for its stretch on its own thread, where the
//! encoder lives, and the clip is written from that copy on the caller's
//! thread so capture never waits on a save. A clip's video is written to an
//! MP4 as it was encoded, and its audio is added the way a recording's is.

mod annexb;
mod convert;
mod encoder;
mod movie;
mod session;
mod writer;

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

pub(super) use crate::recording::replay::ring::ClipFrom;
pub use session::{begin_replay_blocking, ReplayCapture, ReplaySession};
pub(super) use writer::{run, ReplayWriterConfig};

/// What audio is kept beyond the buffer's length: a clip starts on a video
/// keyframe, which can sit a little before the full length.
const AUDIO_MARGIN: Duration = Duration::from_secs(5);

/// How far back a replay of `length` keeps its audio.
pub(super) fn horizon_ns(length: Duration) -> i64 {
  i64::try_from((length + AUDIO_MARGIN).as_nanos()).unwrap_or(i64::MAX)
}

/// An encoded H.264 access unit, Annex B, shared between the ring and a clip
/// being written.
type EncodedFrame = crate::recording::replay::ring::EncodedFrame<Arc<[u8]>>;

/// The newest keyframe the primary video wrote, so a camera beside it can
/// put one of its own right after and a clip can start both together.
#[derive(Clone)]
pub(super) struct KeyframeLink(Arc<AtomicI64>);

impl KeyframeLink {
  pub(super) fn new() -> Self {
    Self(Arc::new(AtomicI64::new(i64::MIN)))
  }

  fn publish(&self, pts_ns: i64) {
    self.0.fetch_max(pts_ns, Ordering::AcqRel);
  }

  fn latest(&self) -> i64 {
    self.0.load(Ordering::Acquire)
  }
}

/// What a writer holds for one clip, in replay time.
pub(super) struct ClipTake {
  pub start_ns: i64,
  pub end_ns: i64,
  pub video: Option<VideoTake>,
}

pub(super) struct VideoTake {
  pub frames: Vec<EncodedFrame>,
  pub fps: u32,
  pub height: u32,
  pub width: u32,
}

pub(super) struct ClipRequest {
  pub at: Instant,
  pub from: ClipFrom,
  pub reply: mpsc::Sender<Result<ClipTake, String>>,
}

impl ClipRequest {
  /// The answer from a writer that is filling a movie rather than a buffer.
  pub(super) fn refuse(self) {
    let _ = self
      .reply
      .send(Err("This capture is not a replay buffer".to_owned()));
  }
}
