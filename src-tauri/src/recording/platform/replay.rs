// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The replay buffer on macOS: the same capture streams a recording opens,
//! feeding writers that keep the last stretch of media in memory instead of
//! filling a movie.
//!
//! Video is encoded as it arrives by its own VideoToolbox session, and the
//! encoded frames are kept back to the keyframe a clip of the full length
//! would have to start from. Audio is kept as the PCM it arrived as: thirty
//! seconds of it is a few megabytes, and encoding it only for the clips that
//! are saved costs nothing while the buffer runs.
//!
//! Saving asks each writer for its stretch on its own thread, where the
//! encoder lives, and the movies are written from that copy on the caller's
//! thread so capture never waits on a save.

mod encoder;
mod mux;
mod ring;
mod session;
mod writer;

#[cfg(test)]
mod tests;

use std::sync::mpsc;
use std::time::Instant;

use cidre::{arc, cm};

pub use session::{begin_blocking, ReplayCapture, ReplaySession};
pub(super) use writer::{KeyframeLink, ReplayWriter, ReplayWriterConfig};

use crate::recording::microphone::Format as MicrophoneFormat;
use ring::{AudioChunk, EncodedFrame};

/// Where a writer's clip begins.
#[derive(Clone, Copy, Debug)]
pub(super) enum ClipFrom {
  /// The primary writer chooses: at most `length_ns` back from the save, and
  /// never before `since_ns`, the end of the previous save.
  Latest {
    length_ns: i64,
    since_ns: Option<i64>,
  },
  /// A secondary writer follows the start the primary chose.
  Follow { start_ns: i64 },
}

/// What a writer holds for one clip, in replay time.
pub(super) struct ClipTake {
  pub start_ns: i64,
  pub end_ns: i64,
  pub video: Option<VideoTake>,
  pub system_audio: Option<Vec<AudioChunk>>,
  pub microphone: Option<(MicrophoneFormat, Vec<AudioChunk>)>,
}

pub(super) struct VideoTake {
  pub frames: Vec<EncodedFrame<SharedSample>>,
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

/// An encoded frame, shared between the encoder's callback, the ring and a
/// clip being written.
#[derive(Clone)]
pub(super) struct SharedSample(pub arc::R<cm::SampleBuf>);

// SAFETY: a compressed `CMSampleBuffer` is immutable once VideoToolbox hands
// it over: the ring, the save and the muxer only read it and retain or release
// it, and CoreFoundation reference counting is atomic.
unsafe impl Send for SharedSample {}
unsafe impl Sync for SharedSample {}
