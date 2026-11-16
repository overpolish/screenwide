// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The voices in a call's computer audio told apart, over the whole call at
//! once: NVIDIA's Nemotron 3 Diarization at its most accurate setting, the
//! one its authors publish for whole recordings, which reads twelve minutes
//! in under two seconds on an M4 Pro. It tells eight voices apart; a call
//! that fills all eight is told apart again in windows, halved while they
//! still fill them, joined by the voices' fingerprints (`voices`).

use std::path::Path;

use screenwide_transcriber::SpeakerTurn;
use transcribe_cpp::{
  DiarizeExtension, DiarizeOptions, DiarizeSession, Model, SortformerDiarizeOptions,
  SortformerPreset,
};

use super::voices::{self, Fingerprints};

pub struct Speakers {
  session: DiarizeSession,
  slots: u32,
  voices: Fingerprints,
}

impl Speakers {
  /// The speaker model at `path` and the voice model at `voices`, loaded.
  pub fn load(path: &Path, voices: &Path) -> Result<Self, String> {
    let failed =
      |error: transcribe_cpp::Error| format!("Could not load the speaker model: {error}");
    let model = Model::load(path).map_err(failed)?;
    let slots = model.diarize_info().map_err(failed)?.max_speakers;
    let session = model.diarize_session().map_err(failed)?;
    Ok(Self {
      session,
      slots: u32::try_from(slots).unwrap_or(0),
      voices: Fingerprints::load(voices)?,
    })
  }

  /// When each voice in `audio`, 16-bit samples, spoke; none where nothing
  /// was heard.
  pub fn turns(&mut self, audio: &[i16]) -> Result<Vec<SpeakerTurn>, String> {
    if audio.is_empty() {
      return Ok(Vec::new());
    }
    let pcm: Vec<f32> = audio
      .iter()
      .map(|&sample| f32::from(sample) / 32_768.0)
      .collect();
    let whole = self.diarize(&pcm, 0)?;
    let call_ms = pcm.len() as u64 / 16;
    if !self.fills(&whole) || call_ms <= voices::WINDOW_MS {
      return Ok(whole);
    }
    let mut windows = Vec::new();
    let mut per_window = Vec::new();
    for window in voices::windows(call_ms) {
      self.windowed(&pcm, window, &mut windows, &mut per_window)?;
    }
    voices::joined(&self.voices, &pcm, &windows, &per_window)
  }

  /// Tells `window` of the call apart, in halves while it fills every slot,
  /// adding each window told apart to `windows` and its turns to
  /// `per_window`, in order.
  fn windowed(
    &mut self,
    pcm: &[f32],
    (start, end): (u64, u64),
    windows: &mut Vec<(u64, u64)>,
    per_window: &mut Vec<Vec<SpeakerTurn>>,
  ) -> Result<(), String> {
    let turns = self.diarize(&pcm[(start * 16) as usize..(end * 16) as usize], start)?;
    let halves = if self.fills(&turns) {
      voices::halves((start, end))
    } else {
      None
    };
    if let Some(halves) = halves {
      for half in halves {
        self.windowed(pcm, half, windows, per_window)?;
      }
      return Ok(());
    }
    windows.push((start, end));
    per_window.push(turns);
    Ok(())
  }

  /// Whether `turns` hold as many voices as the model has room for.
  fn fills(&self, turns: &[SpeakerTurn]) -> bool {
    let mut heard: Vec<u32> = turns.iter().map(|turn| turn.speaker).collect();
    heard.sort_unstable();
    heard.dedup();
    heard.len() as u32 >= self.slots
  }

  /// The turns in `pcm`, which starts `offset_ms` into the call.
  fn diarize(&mut self, pcm: &[f32], offset_ms: u64) -> Result<Vec<SpeakerTurn>, String> {
    let options = DiarizeOptions {
      family: Some(DiarizeExtension::Sortformer(SortformerDiarizeOptions {
        preset: Some(SortformerPreset::VeryHighLatency),
      })),
    };
    let segments = self
      .session
      .run(pcm, &options)
      .map_err(|error| format!("Could not tell the speakers apart: {error}"))?;
    Ok(
      segments
        .into_iter()
        .map(|segment| SpeakerTurn {
          start_ms: offset_ms + u64::try_from(segment.t0_ms).unwrap_or(0),
          end_ms: offset_ms + u64::try_from(segment.t1_ms).unwrap_or(0),
          speaker: u32::try_from(segment.speaker_id).unwrap_or(0),
        })
        .collect(),
    )
  }
}
