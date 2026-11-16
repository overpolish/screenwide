// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Listening on Windows: the microphone and the WASAPI loopback captures a
//! recording starts, each run of samples handed to a listener with how long
//! after listening began it was heard; nothing is written.

use std::sync::{Arc, OnceLock};
use std::time::Instant;

use super::audio::{AudioCaptures, AudioDestination};
use crate::listening::{Listener, Listening};

/// Listening under way. Stopped, or dropped, it lets go of the microphone
/// and the computer's audio.
pub struct ListeningSession {
  _audio: AudioCaptures,
}

impl ListeningSession {
  pub fn stop(self) {}
}

/// Starts listening as `listening` says, handing what is heard to
/// `listener`.
pub fn begin_listening_blocking(
  listening: Listening,
  listener: Listener,
) -> Result<ListeningSession, String> {
  let origin = Arc::new(OnceLock::new());
  let _ = origin.set(Instant::now());
  let audio = AudioCaptures::start(
    listening.microphone_id.as_deref(),
    &listening.system_audio,
    origin,
    listening.monitor,
    listening.on_failure,
    AudioDestination::Listened(&listener),
  )?;
  Ok(ListeningSession { _audio: audio })
}
