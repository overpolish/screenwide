// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The songs and sound effects a plan plays under the recording.

use screenwide_sound_track::{SoundFile, SoundTrack};

use super::TimelinePlan;

impl TimelinePlan {
  /// The songs and sound effects, and the files they play.
  pub fn sound_track(&self) -> &SoundTrack {
    &self.sound_track
  }

  /// Finds the files the audio clips play with `resolve`, and names the
  /// recorded stream of the voice they play under, if there is one.
  pub fn find_sounds(&mut self, voice: Option<usize>, resolve: impl Fn(&str) -> Option<SoundFile>) {
    self.sound_track = std::mem::take(&mut self.sound_track).with_files(resolve);
    self.sound_track.voice = voice;
  }
}
