// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Listening without recording: the microphone and the computer's audio
//! handed over as they arrive, and nothing written anywhere. Meeting notes
//! listen this way, keeping no audio of the call. Each platform's capture
//! starts it (`capture::begin_listening_blocking`).

use std::sync::Arc;
use std::time::Duration;

use crate::encoding::FailureReport;
use crate::monitor::RecordingMonitor;
use crate::SystemAudioSelection;

/// What to listen to.
pub struct Listening {
  /// The microphone's id, or none for no microphone.
  pub microphone_id: Option<String>,
  pub system_audio: SystemAudioSelection,
  /// Where the island reads the levels from.
  pub monitor: Arc<RecordingMonitor>,
  /// Told when listening fails part-way.
  pub on_failure: FailureReport,
}

/// Where a run of samples was heard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeardSource {
  Microphone,
  System,
}

/// A run of samples, interleaved, as the source gave them, and how long
/// after listening began the first was heard.
#[derive(Debug)]
pub struct Heard {
  pub source: HeardSource,
  pub samples: Vec<f32>,
  pub channels: u16,
  pub sample_rate: u32,
  pub at: Duration,
}

/// What is handed each run of samples, on the capture's own threads.
pub type Listener = Arc<dyn Fn(Heard) + Send + Sync>;
