// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One frame of an export for the export window to set beside the same frame
//! at Original. The exported frame is the last of a stretch drawn and
//! encoded exactly as the export would: an encoder spends far more on the
//! keyframe a file opens on than on the frames after it, so a frame encoded
//! on its own would flatter the compression. The Original frame is drawn and
//! never encoded.
//!
//! FFmpeg hands the frame over as a PPM, which `image` reads at once and
//! writes as the PNG the window shows: the FFmpeg built for macOS leaves out
//! zlib, and with it the PNG encoder. Only the frame shown is converted from
//! the video's own pixels; converting every frame of the stretch cost more
//! than drawing and encoding it.

use screenwide_sidecar::ffmpeg_command;
use screenwide_timeline::{within_output_windows, TimelineRange};
use screenwide_timeline_edit::TimelinePlan;

use super::sample_frames::SampleFrames;
use super::*;

/// How much of the edited video is encoded ahead of the frame shown.
const LEAD_US: u64 = 1_000_000;

static PREVIEW_ATTEMPTS: AtomicU64 = AtomicU64::new(0);

/// Which of the two compared pictures a preview frame is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewPicture {
  /// As the export's encoder leaves it.
  Encoded,
  /// As drawn, before any encoder.
  Drawn,
}

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

/// What a preview of the frame at an edited-video moment draws.
pub(crate) struct PreviewStretch {
  /// The stretch, as ranges laid end to end from zero.
  pub(crate) ranges: Vec<TimelineRange>,
  /// Where the stretch starts on the whole edited video.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub(crate) start_us: u64,
}

impl PreviewStretch {
  /// The stretch of the edited video `whole` lays out that ends on the
  /// frame at `output_us`, encoded ahead of it.
  pub(crate) fn leading_to(whole: &[TimelineRange], output_us: u64) -> Result<Self, String> {
    Self::spanning(whole, output_us.saturating_sub(LEAD_US), output_us)
  }

  /// The frame at `output_us` of the edited video `whole` lays out, alone.
  #[cfg(target_os = "macos")]
  pub(crate) fn single(whole: &[TimelineRange], output_us: u64) -> Result<Self, String> {
    Self::spanning(whole, output_us, output_us)
  }

  /// From `start_us` to the frame at `output_us`: a microsecond past that
  /// moment keeps its frame and stops before the next one.
  fn spanning(whole: &[TimelineRange], start_us: u64, output_us: u64) -> Result<Self, String> {
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

/// What `command` wrote to its standard output.
fn run(mut command: std::process::Command) -> Result<Vec<u8>, String> {
  let output = command
    .output()
    .map_err(|error| format!("FFmpeg could not start the preview frame: {error}"))?;
  if output.status.success() {
    return Ok(output.stdout);
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

/// The filter keeping only the frame at `index`, counted from zero.
fn only_frame(index: u64) -> String {
  format!("select=eq(n\\,{index})")
}

/// Writes `ppm`, a picture FFmpeg wrote, to `png`.
pub(crate) fn write_picture(ppm: &[u8], png: &Path) -> Result<(), String> {
  if ppm.is_empty() {
    return Err("FFmpeg wrote no preview frame".to_owned());
  }
  ::image::load_from_memory_with_format(ppm, ::image::ImageFormat::Pnm)
    .and_then(|frame| frame.save_with_format(png, ::image::ImageFormat::Png))
    .map_err(|error| format!("The preview frame could not be written: {error}"))
}

/// Runs `command`, which maps the one frame to show, and writes it to `png`.
fn write_frame(mut command: std::process::Command, png: &Path) -> Result<(), String> {
  command.args([
    "-frames:v",
    "1",
    "-c:v",
    "ppm",
    "-f",
    "image2pipe",
    "pipe:1",
  ]);
  write_picture(&run(command)?, png)
}

/// Writes the last of the `frames` frames of the video at `video` to `png`.
/// `format` names a raw stream FFmpeg cannot tell by its file name.
pub(crate) fn write_last_frame(
  video: &Path,
  format: Option<&str>,
  frames: u64,
  png: &Path,
) -> Result<(), String> {
  let last = frames
    .checked_sub(1)
    .ok_or_else(|| "The preview stretch has no frames".to_owned())?;
  let mut command = quiet_command();
  if let Some(format) = format {
    command.args(["-f", format]);
  }
  command
    .arg("-i")
    .arg(video)
    .args(["-map", "0:v:0", "-vf"])
    .arg(only_frame(last));
  write_frame(command, png)
}

fn seconds(microseconds: u64) -> String {
  format!("{:.6}", microseconds as f64 / 1_000_000.0)
}

/// FFmpeg reading `stretch` of `source`, each range its own input, joined
/// into one picture mapped as `[preview]` and passed through `filters`.
fn stretch_command(
  source: &Path,
  stretch: &PreviewStretch,
  filters: &[String],
) -> std::process::Command {
  let mut command = quiet_command();
  let mut graph = String::new();
  let mut joined = String::new();
  for (index, range) in stretch.ranges.iter().enumerate() {
    command
      .arg("-ss")
      .arg(seconds(range.source_start_us))
      .arg("-t")
      .arg(seconds(range.source_end_us - range.source_start_us))
      .arg("-i")
      .arg(source);
    graph.push_str(&format!(
      "[{index}:v:0]setpts=(PTS-STARTPTS)/{}[piece{index}];",
      range.playback_rate
    ));
    joined.push_str(&format!("[piece{index}]"));
  }
  graph.push_str(&format!(
    "{joined}concat=n={}:v=1:a=0",
    stretch.ranges.len()
  ));
  for filter in filters {
    graph.push(',');
    graph.push_str(filter);
  }
  graph.push_str("[preview]");
  command
    .arg("-filter_complex")
    .arg(graph)
    .args(["-map", "[preview]"]);
  command
}

/// How many frames `command`, mapping a picture, decodes.
fn count_frames(mut command: std::process::Command) -> Result<u64, String> {
  command.args(["-progress", "pipe:1", "-nostats", "-f", "null", "-"]);
  let progress = run(command)?;
  String::from_utf8_lossy(&progress)
    .lines()
    .rev()
    .find_map(|line| line.strip_prefix("frame=")?.trim().parse::<u64>().ok())
    .ok_or_else(|| "FFmpeg did not count the preview stretch".to_owned())
}

/// The frame at `output_us` of `source`'s edited video as `picture` asks,
/// at `png`. `timeline` is the edit the export follows, which always
/// re-encodes, over a recording `duration_ms` long. Where the export copies
/// the recording as it is, its frame is the recording's own, as drawn.
pub fn preview_frame(
  source: &Path,
  (timeline, duration_ms): (Option<&TimelinePlan>, u64),
  output_us: u64,
  video: VideoExportOptions,
  picture: PreviewPicture,
  png: &Path,
) -> Result<(), String> {
  let stretch = PreviewStretch::leading_to(&edited_ranges(timeline, duration_ms), output_us)?;
  let scale = resolution_filter(video.source_scale_percent, video.resolution_scale_percent);
  let encodes = picture == PreviewPicture::Encoded
    && export_crf(video.compression, scale.is_some() || timeline.is_some()).is_some();
  let mut filters: Vec<String> = scale.into_iter().collect();
  if !encodes {
    // The same stretch as the encoded frame's, so both sides show the frame
    // its last one is, decoded once to count it and again to keep it.
    let frames = count_frames(stretch_command(source, &stretch, &filters))?;
    let last = frames
      .checked_sub(1)
      .ok_or_else(|| "The preview stretch has no frames".to_owned())?;
    filters.push(only_frame(last));
    return write_frame(stretch_command(source, &stretch, &filters), png);
  }
  // A raw stream, whose frames `SampleFrames` counts; passed through so
  // every decoded frame is encoded once, as the drawn side counts them.
  let encoded = preview_temp_path("h264");
  let mut command = stretch_command(source, &stretch, &filters);
  command
    .args(encode::x264_args(video))
    .args(["-fps_mode", "passthrough", "-f", "h264"])
    .arg(&encoded);
  let result = run(command)
    .and_then(|_| std::fs::read(&encoded).map_err(|error| error.to_string()))
    .and_then(|stream| {
      write_last_frame(
        &encoded,
        Some("h264"),
        SampleFrames::of(&stream).frames(),
        png,
      )
    });
  let _ = std::fs::remove_file(&encoded);
  result
}
