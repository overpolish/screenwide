// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a recording lost on its way to the file, saved beside the movie as
//! `<name>-capture.json`. The bundled app's standard error goes nowhere, so
//! this file is what is left to tell afterwards why a recording stutters or
//! where its sound had gaps.

use std::path::Path;
use std::sync::atomic::AtomicU64;

use serde::Serialize;

use super::*;

const REPORT_VERSION: u32 = 1;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
  version: u32,
  video: VideoReport,
  #[serde(skip_serializing_if = "Option::is_none")]
  system_audio: Option<AudioReport>,
  #[serde(skip_serializing_if = "Option::is_none")]
  microphone: Option<AudioReport>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoReport {
  appended: u64,
  dropped_by_device: u64,
  dropped_at_queue: u64,
  not_ready: u64,
  rejected: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioReport {
  /// Buffers lost because even the audio queue to the writer was full.
  dropped_at_queue: u64,
  /// Buffers let go because the encoder was busy for longer than a track
  /// holds audio back.
  dropped_at_encoder: u64,
  rejected: u64,
  /// Lost audio written back as silence, in milliseconds.
  silence_ms: u64,
}

fn load(counter: &AtomicU64) -> u64 {
  counter.load(Ordering::Relaxed)
}

fn audio_report(
  dropped: &AtomicU64,
  dropped_at_encoder: &AtomicU64,
  rejected: &AtomicU64,
  silence_ns: &AtomicU64,
) -> AudioReport {
  AudioReport {
    dropped_at_queue: load(dropped),
    dropped_at_encoder: load(dropped_at_encoder),
    rejected: load(rejected),
    silence_ms: load(silence_ns) / NANOS_PER_MS as u64,
  }
}

fn print_audio(name: &str, report: &AudioReport) {
  let lost = report.dropped_at_queue + report.dropped_at_encoder + report.rejected;
  if lost > 0 || report.silence_ms > 0 {
    eprintln!(
      "Recording dropped {} {name} buffers at the capture queue, {} at the encoder, rejected {}, and wrote {} ms of silence for lost audio",
      report.dropped_at_queue, report.dropped_at_encoder, report.rejected, report.silence_ms
    );
  }
}

/// Where the report for the movie at `movie` is kept.
fn report_path(movie: &Path) -> std::path::PathBuf {
  let stem = movie
    .file_stem()
    .map_or_else(Default::default, |stem| stem.to_string_lossy());
  movie.with_file_name(format!("{stem}-capture.json"))
}

impl Writer {
  pub(super) fn report_capture(&self) {
    let stats = &self.stats;
    let video = VideoReport {
      appended: load(&stats.appended),
      dropped_by_device: load(&stats.capture_dropped),
      dropped_at_queue: load(&stats.dropped),
      not_ready: load(&stats.not_ready),
      rejected: load(&stats.rejected),
    };
    if video.dropped_by_device + video.dropped_at_queue + video.not_ready > 0 {
      eprintln!(
        "Recording dropped {} frames at the capture device, {} at the capture queue, and {} at the encoder",
        video.dropped_by_device, video.dropped_at_queue, video.not_ready
      );
    }
    let system_audio = self.system_audio.is_some().then(|| {
      audio_report(
        &stats.audio_dropped,
        &stats.audio_not_ready,
        &stats.audio_rejected,
        &stats.audio_silence_ns,
      )
    });
    let microphone = self.microphone.is_some().then(|| {
      audio_report(
        &stats.microphone_dropped,
        &stats.microphone_not_ready,
        &stats.microphone_rejected,
        &stats.microphone_silence_ns,
      )
    });
    if let Some(report) = &system_audio {
      print_audio("system-audio", report);
    }
    if let Some(report) = &microphone {
      print_audio("microphone", report);
    }

    let report = Report {
      version: REPORT_VERSION,
      video,
      system_audio,
      microphone,
    };
    let saved = serde_json::to_vec(&report)
      .map_err(|error| error.to_string())
      .and_then(|json| {
        std::fs::write(report_path(&self.path), json).map_err(|error| error.to_string())
      });
    if let Err(error) = saved {
      eprintln!("Could not save what this recording lost: {error}");
    }
  }
}
