// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The media the replay buffer holds, and where a clip of it may start.
//!
//! Generic over the sample type so the cutting rules can be tested without
//! an encoder.

use std::collections::VecDeque;

#[derive(Clone)]
pub(in crate::recording::platform) struct EncodedFrame<S> {
  pub keyframe: bool,
  pub pts_ns: i64,
  pub sample: S,
}

/// Encoded video, kept back to the keyframe that a clip starting
/// `horizon_ns` before the newest frame has to begin from.
pub(super) struct VideoRing<S> {
  frames: VecDeque<EncodedFrame<S>>,
  horizon_ns: i64,
}

impl<S: Clone> VideoRing<S> {
  pub(super) fn new(horizon_ns: i64) -> Self {
    Self {
      frames: VecDeque::new(),
      horizon_ns,
    }
  }

  pub(super) fn push(&mut self, frame: EncodedFrame<S>) {
    let cutoff = frame.pts_ns.saturating_sub(self.horizon_ns);
    self.frames.push_back(frame);
    // Everything before the newest keyframe at or before the cutoff can no
    // longer be the start of a clip, nor be needed to decode one.
    let keep_from = self
      .frames
      .iter()
      .take_while(|frame| frame.pts_ns <= cutoff)
      .enumerate()
      .filter(|(_, frame)| frame.keyframe)
      .map(|(index, _)| index)
      .last();
    if let Some(keep_from) = keep_from {
      self.frames.drain(..keep_from);
    }
  }

  /// Where a clip that wants to start at `at_ns` has to start: the newest
  /// keyframe at or before it, or failing that the oldest one held.
  pub(super) fn keyframe_at_or_before(&self, at_ns: i64) -> Option<i64> {
    let keyframes = || self.frames.iter().filter(|frame| frame.keyframe);
    keyframes()
      .take_while(|frame| frame.pts_ns <= at_ns)
      .last()
      .or_else(|| keyframes().next())
      .map(|frame| frame.pts_ns)
  }

  /// The first keyframe at or after `at_ns`, for a writer that has to follow
  /// a start another writer chose.
  pub(super) fn keyframe_at_or_after(&self, at_ns: i64) -> Option<i64> {
    self
      .frames
      .iter()
      .find(|frame| frame.keyframe && frame.pts_ns >= at_ns)
      .map(|frame| frame.pts_ns)
  }

  /// The frames from the keyframe at `start_ns` through `end_ns`.
  pub(super) fn frames(&self, start_ns: i64, end_ns: i64) -> Vec<EncodedFrame<S>> {
    self
      .frames
      .iter()
      .skip_while(|frame| frame.pts_ns < start_ns)
      .take_while(|frame| frame.pts_ns <= end_ns)
      .cloned()
      .collect()
  }
}

/// Interleaved PCM that starts at `pts_ns`.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::recording::platform) struct AudioChunk {
  pub pts_ns: i64,
  pub samples: Vec<f32>,
}

/// PCM held for the last `horizon_ns`.
pub(super) struct AudioRing {
  channels: usize,
  chunks: VecDeque<AudioChunk>,
  horizon_ns: i64,
  sample_rate: u32,
}

impl AudioRing {
  pub(super) fn new(channels: u16, sample_rate: u32, horizon_ns: i64) -> Self {
    Self {
      channels: usize::from(channels.max(1)),
      chunks: VecDeque::new(),
      horizon_ns,
      sample_rate: sample_rate.max(1),
    }
  }

  fn duration_ns(&self, chunk: &AudioChunk) -> i64 {
    let frames = (chunk.samples.len() / self.channels) as i64;
    frames.saturating_mul(1_000_000_000) / i64::from(self.sample_rate)
  }

  /// Where the newest chunk ends, which is where the next one belongs.
  pub(super) fn end_ns(&self) -> Option<i64> {
    self
      .chunks
      .back()
      .map(|chunk| chunk.pts_ns.saturating_add(self.duration_ns(chunk)))
  }

  pub(super) fn push(&mut self, chunk: AudioChunk) {
    let cutoff = chunk.pts_ns.saturating_sub(self.horizon_ns);
    self.chunks.push_back(chunk);
    while self
      .chunks
      .front()
      .is_some_and(|front| front.pts_ns.saturating_add(self.duration_ns(front)) < cutoff)
    {
      self.chunks.pop_front();
    }
  }

  /// The audio of `start_ns..end_ns`, the first chunk trimmed to start
  /// exactly there.
  pub(super) fn clip(&self, start_ns: i64, end_ns: i64) -> Vec<AudioChunk> {
    let mut clip = Vec::new();
    for chunk in &self.chunks {
      let chunk_end = chunk.pts_ns.saturating_add(self.duration_ns(chunk));
      if chunk_end <= start_ns || chunk.pts_ns >= end_ns {
        continue;
      }
      if chunk.pts_ns >= start_ns {
        clip.push(chunk.clone());
        continue;
      }
      let skip_frames = ((start_ns - chunk.pts_ns) as u128 * u128::from(self.sample_rate))
        .div_ceil(1_000_000_000) as usize;
      let skip = (skip_frames * self.channels).min(chunk.samples.len());
      if skip < chunk.samples.len() {
        clip.push(AudioChunk {
          pts_ns: start_ns,
          samples: chunk.samples[skip..].to_vec(),
        });
      }
    }
    clip
  }
}

#[cfg(test)]
mod tests;
