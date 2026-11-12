// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a piece is made of: the sound file it plays, and how it fades.

use std::path::PathBuf;

/// A sound file a clip plays, as the mix needs it.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundFile {
  pub path: PathBuf,
  pub duration_ms: u64,
  /// How loud it is over its whole length, in LUFS; none where it is silent.
  pub loudness_lufs: Option<f64>,
  /// The singing in it, separated once by Studio ducking's model, lined up
  /// with it from its start; none until that is done, or without the model.
  pub singing: Option<PathBuf>,
}

/// How a fade's level moves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Curve {
  /// Straight, for a fade to or from silence.
  Linear,
  /// A quarter sine, so two sounds crossfading keep their power together.
  EqualPower,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fade {
  pub ms: u64,
  pub curve: Curve,
}

impl Fade {
  pub(super) const NONE: Self = Self {
    ms: 0,
    curve: Curve::Linear,
  };

  pub(super) const fn linear(ms: u64) -> Self {
    Self {
      ms,
      curve: Curve::Linear,
    }
  }

  pub(super) const fn equal_power(ms: u64) -> Self {
    Self {
      ms,
      curve: Curve::EqualPower,
    }
  }
}

/// A stretch of one sound file placed on the timeline: the file from
/// `file_start_ms` plays from `start_ms` to `end_ms` of the timeline.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
  pub path: PathBuf,
  pub file_start_ms: u64,
  pub start_ms: u64,
  pub end_ms: u64,
  pub fade_in: Fade,
  pub fade_out: Fade,
  pub gain_db: f64,
  /// Whether it comes down while the voice speaks.
  pub makes_way: bool,
  /// The singing in the file, which gets out of the voice's way entirely.
  pub singing: Option<PathBuf>,
}

impl Piece {
  pub(super) const fn length_ms(&self) -> u64 {
    self.end_ms.saturating_sub(self.start_ms)
  }

  /// Its fades shortened together where they would overlap.
  pub(super) fn fit_fades(&mut self) {
    let length = self.length_ms();
    let total = self.fade_in.ms + self.fade_out.ms;
    if total > length {
      self.fade_in.ms = self.fade_in.ms * length / total;
      self.fade_out.ms = length - self.fade_in.ms;
    }
  }
}
