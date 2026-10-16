// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One frame of an export as its encoder leaves it, for the export window to
//! set beside the same frame at Original. The frame is the last of a stretch
//! drawn and encoded exactly as the export would: an encoder spends far more
//! on the keyframe a file opens on than on the frames after it, so a frame
//! encoded on its own would flatter the compression.

use screenwide_sidecar::ffmpeg_command;
use screenwide_timeline::{within_output_windows, TimelineRange};
use screenwide_timeline_edit::TimelinePlan;

use super::*;

/// How much of the edited video is encoded ahead of the frame shown.
const LEAD_US: u64 = 1_000_000;

static PREVIEW_ATTEMPTS: AtomicU64 = AtomicU64::new(0);

/// How `timeline` lays out the source, or the whole `duration_ms` of it
/// where nothing was edited.
pub(crate) fn edited_ranges(
  timeline: Option<&TimelinePlan>,
  duration_ms: u64,
) -> Vec<TimelineRange> {
  timeline.map_or_else(
    || {
      vec![TimelineRange {
        output_start_us: 0,
        source_end_us: duration_ms.saturating_mul(1_000),
        source_start_us: 0,
        playback_rate: 1.0,
      }]
    },
    |timeline| timeline.ranges().to_vec(),
  )
}

/// What a preview of the frame at an edited-video moment encodes.
pub(crate) struct PreviewStretch {
  /// The stretch, as ranges laid end to end from zero.
  pub(crate) ranges: Vec<TimelineRange>,
  /// Where the stretch starts on the whole edited video.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub(crate) start_us: u64,
}

impl PreviewStretch {
  /// The stretch of the edited video `whole` lays out that ends on the
  /// frame at `output_us`. A microsecond past that moment keeps its frame
  /// and stops before the next one.
  pub(crate) fn leading_to(whole: &[TimelineRange], output_us: u64) -> Result<Self, String> {
    let start_us = output_us.saturating_sub(LEAD_US);
    let ranges = within_output_windows(whole, &[[start_us, output_us.saturating_add(1)]]);
    if ranges.is_empty() {
      return Err("The preview frame is outside the edited video".to_owned());
    }
    Ok(Self { ranges, start_us })
  }

  /// The stretch of the recording the stretch draws from, in milliseconds.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(crate) fn source_span_ms(&self) -> [u64; 2] {
    let start_us = self.ranges.iter().map(|range| range.source_start_us).min();
    let end_us = self.ranges.iter().map(|range| range.source_end_us).max();
    [
      start_us.unwrap_or_default() / 1_000,
      end_us.unwrap_or_default().div_ceil(1_000),
    ]
  }
}

/// A scratch file for an encoded stretch, named `extension`.
pub(crate) fn preview_temp_path(extension: &str) -> PathBuf {
  let attempt = PREVIEW_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
  std::env::temp_dir().join(format!(
    "{PREVIEW_PREFIX}frame-{}-{attempt}.{extension}",
    std::process::id()
  ))
}

fn run(mut command: std::process::Command) -> Result<(), String> {
  let output = command
    .output()
    .map_err(|error| format!("FFmpeg could not start the preview frame: {error}"))?;
  if output.status.success() {
    return Ok(());
  }
  let detail = String::from_utf8_lossy(&output.stderr);
  Err(format!(
    "FFmpeg could not make the preview frame: {}",
    detail.trim()
  ))
}

fn quiet_command() -> std::process::Command {
  let mut command = ffmpeg_command();
  command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"]);
  command
}

/// Writes the frame the decoder reaches last to `png`, each decoded frame
/// replacing the one before.
fn last_frame_args(command: &mut std::process::Command, png: &Path) {
  command
    .args(["-update", "1", "-c:v", "png", "-f", "image2"])
    .arg(png);
}

/// Writes the last frame of the video at `video` to `png`. `format` names a
/// raw stream FFmpeg cannot tell by its file name.
pub(crate) fn write_last_frame(
  video: &Path,
  format: Option<&str>,
  png: &Path,
) -> Result<(), String> {
  let mut command = quiet_command();
  if let Some(format) = format {
    command.args(["-f", format]);
  }
  command.arg("-i").arg(video).args(["-map", "0:v:0"]);
  last_frame_args(&mut command, png);
  run(command)
}

fn seconds(microseconds: u64) -> String {
  format!("{:.6}", microseconds as f64 / 1_000_000.0)
}

/// Each range's own input joined into one picture, at the export's size.
fn stretch_filter(ranges: &[TimelineRange], scale: Option<&str>) -> String {
  let mut filter = String::new();
  let mut joined = String::new();
  for (index, range) in ranges.iter().enumerate() {
    filter.push_str(&format!(
      "[{index}:v:0]setpts=(PTS-STARTPTS)/{}[piece{index}];",
      range.playback_rate
    ));
    joined.push_str(&format!("[piece{index}]"));
  }
  filter.push_str(&format!("{joined}concat=n={}:v=1:a=0", ranges.len()));
  if let Some(scale) = scale {
    filter.push(',');
    filter.push_str(scale);
  }
  filter.push_str("[preview]");
  filter
}

/// The frame at `output_us` of `source`'s edited video as FFmpeg's export
/// writes it, at `png`. `timeline` is the edit the export follows, which
/// always re-encodes, over a recording `duration_ms` long. Where the export
/// copies the recording as it is, the frame is the recording's own.
pub fn preview_frame(
  source: &Path,
  (timeline, duration_ms): (Option<&TimelinePlan>, u64),
  output_us: u64,
  video: VideoExportOptions,
  png: &Path,
) -> Result<(), String> {
  let stretch = PreviewStretch::leading_to(&edited_ranges(timeline, duration_ms), output_us)?;
  let scale = resolution_filter(video.source_scale_percent, video.resolution_scale_percent);
  let encodes = export_crf(video.compression, scale.is_some() || timeline.is_some()).is_some();
  let mut command = quiet_command();
  for range in &stretch.ranges {
    command
      .arg("-ss")
      .arg(seconds(range.source_start_us))
      .arg("-t")
      .arg(seconds(range.source_end_us - range.source_start_us))
      .arg("-i")
      .arg(source);
  }
  command
    .arg("-filter_complex")
    .arg(stretch_filter(&stretch.ranges, scale.as_deref()))
    .args(["-map", "[preview]"]);
  if !encodes {
    last_frame_args(&mut command, png);
    return run(command);
  }
  let encoded = preview_temp_path("mp4");
  command
    .args(encode::x264_args(video))
    .args(["-f", "mp4"])
    .arg(&encoded);
  let result = run(command).and_then(|()| write_last_frame(&encoded, None, png));
  let _ = std::fs::remove_file(&encoded);
  result
}
