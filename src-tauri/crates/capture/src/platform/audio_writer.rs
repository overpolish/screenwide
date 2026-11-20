// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Audio-only capture has its own audio-led writer. It never opens
//! a video encoder or creates a placeholder video track.

mod lifecycle;
mod samples;

use std::collections::VecDeque;
use std::path::PathBuf;

use cidre::av;

use super::audio_track::{AudioTrack, Track, TrackError, SYSTEM_AUDIO_FORMAT};
use super::media::{
  microphone_audio_settings, microphone_buffer_from_origin, nanos, system_audio_settings,
};
use super::*;
use crate::encoding::{FailureReport, Timeline};
use screenwide_project::PrimaryRecordingKind;

pub(super) struct AudioWriter {
  base: Instant,
  failed: Option<String>,
  microphone: Option<AudioTrack>,
  on_failure: FailureReport,
  origin: Option<Instant>,
  path: PathBuf,
  pending_microphone: VecDeque<MicrophoneBuffer>,
  pending_system_audio: VecDeque<AudioSample>,
  stats: Arc<CaptureStats>,
  system_audio: Option<AudioTrack>,
  timeline: Timeline,
  writer: arc::R<av::AssetWriter>,
}

impl AudioWriter {
  pub(super) fn new(
    path: PathBuf,
    system_audio: bool,
    microphone_format: Option<MicrophoneFormat>,
    stats: Arc<CaptureStats>,
    on_failure: FailureReport,
  ) -> Result<Self, String> {
    let location = path
      .to_str()
      .ok_or_else(|| "The recording's location cannot be written as text".to_owned())?;
    let url = ns::Url::with_fs_path_str(location, false);
    let container = Container::quicktime_fragmented();
    let mut writer = av::AssetWriter::with_url_and_file_type(&url, container.format.file_type())
      .map_err(|error| error.to_string())?;
    if let Some(interval) = container.fragment_interval {
      writer.set_movie_fragment_interval(interval);
    }

    let system_audio = system_audio
      .then(|| AudioTrack::add_to(&mut writer, &system_audio_settings(), SYSTEM_AUDIO_FORMAT))
      .transpose()?;
    let microphone = microphone_format
      .map(|format| AudioTrack::add_to(&mut writer, &microphone_audio_settings(format), format))
      .transpose()?;
    if !writer.start_writing() {
      return Err(asset_writer_error(
        &writer,
        "The audio recording could not be started",
      ));
    }
    writer.start_session_at_src_time(cm::Time::zero());

    Ok(Self {
      base: Instant::now(),
      failed: None,
      microphone,
      on_failure,
      origin: None,
      path,
      pending_microphone: VecDeque::new(),
      pending_system_audio: VecDeque::new(),
      stats,
      system_audio,
      timeline: Timeline::default(),
      writer,
    })
  }

  fn elapsed_ns(&self, at: Instant) -> i64 {
    i64::try_from(at.saturating_duration_since(self.base).as_nanos()).unwrap_or(i64::MAX)
  }

  fn begin(&mut self, at: Instant) -> Result<(), String> {
    self.origin = Some(at);
    let wall_ns = self.elapsed_ns(at);
    self.timeline.start_at(0, wall_ns);
    for sample in std::mem::take(&mut self.pending_system_audio) {
      self.append_system_audio(sample);
    }
    for buffer in std::mem::take(&mut self.pending_microphone) {
      self.append_pcm(buffer, Track::Microphone);
    }
    self.pad_tracks_to(0);
    self.failed.clone().map_or(Ok(()), Err)
  }

  fn fail(&mut self, error: String) {
    if self.failed.is_none() {
      (self.on_failure)(error.clone());
      self.failed = Some(error);
    }
  }
}
