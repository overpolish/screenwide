// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_sidecar::ffmpeg_command;

use super::sample_frames::{stretch_starts, SampleFrames};

/// The H.264 quality represented by the compression control.
///
/// Zero is deliberately not a CRF: it means the original encoded video is
/// copied without a generation loss. The remaining values are named quality
/// steps in the UI, so each maps to a stable encoder setting.
pub(super) fn compression_crf(compression: u8) -> Option<u16> {
  match compression {
    0 => None,
    1 => Some(20),
    2 => Some(24),
    3 => Some(28),
    _ => Some(32),
  }
}

pub(super) fn resolution_filter(
  source_scale_percent: u16,
  resolution_scale_percent: u16,
) -> Option<String> {
  (resolution_scale_percent < source_scale_percent).then(|| {
    format!(
      "scale=trunc(iw*{resolution_scale_percent}/{source_scale_percent}/2)*2:trunc(ih*{resolution_scale_percent}/{source_scale_percent}/2)*2:flags=lanczos"
    )
  })
}

/// Resizing cannot stream-copy. High quality is deliberately used if a caller
/// requests a smaller resolution with Original compression.
pub(crate) fn export_crf(compression: u8, is_resizing: bool) -> Option<u16> {
  compression_crf(compression).or(is_resizing.then_some(20))
}

/// Whether this FFmpeg build carries the software H.264 encoder used to make
/// compression behave the same on macOS and Windows.
pub fn supports_compression() -> bool {
  static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

  *AVAILABLE.get_or_init(|| {
    ffmpeg_command()
      .args(["-hide_banner", "-encoders"])
      .stdin(Stdio::null())
      .output()
      .is_ok_and(|output| {
        output.status.success()
          && String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.split_whitespace().nth(1) == Some("libx264"))
      })
  })
}

static ESTIMATE_ATTEMPTS: AtomicU64 = AtomicU64::new(0);

/// How long each stretch an estimate encodes runs, and how many it spreads
/// across a longer video. A screen recording spends its bits in bursts, a
/// scroll or a zoom, between long still stretches, so many short stretches
/// see the bursts in proportion where a few longer ones over- or
/// under-count whichever they happen to land on.
const SAMPLE_US: u64 = 1_000_000;
const SAMPLES: u64 = 12;

/// How long a video an estimate encodes whole. Sampled, a short video's
/// stretches cover most of it, so one burst they land on weighs far more
/// than it does in the video; encoded whole it is exact, for little more
/// work than the samples.
const WHOLE_UP_TO_US: u64 = 20_000_000;

/// The stretches, `[start_us, end_us)`, an estimate encodes of a video
/// `duration_us` long: all of it when short, otherwise evenly spaced
/// stretches centred across it.
pub(crate) fn sample_windows(duration_us: u64) -> Vec<[u64; 2]> {
  if duration_us <= WHOLE_UP_TO_US {
    return vec![[0, duration_us]];
  }
  (0..SAMPLES)
    .map(|index| {
      let centre = duration_us * (2 * index + 1) / (2 * SAMPLES);
      [centre - SAMPLE_US / 2, centre + SAMPLE_US / 2]
    })
    .collect()
}

/// In the system's temporary folder, never beside the recording: that is a
/// project the user may be syncing or about to send.
pub(crate) fn estimate_temp_path() -> PathBuf {
  let attempt = ESTIMATE_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
  std::env::temp_dir().join(format!(
    "{PREVIEW_PREFIX}estimate-{}-{attempt}.h264.part",
    std::process::id()
  ))
}

pub(super) fn estimate_filter(sample_count: usize, scale_filter: Option<&str>) -> String {
  let mut filter = String::new();
  if sample_count == 1 {
    filter.push_str("[0:v:0]setpts=PTS-STARTPTS");
  } else {
    for index in 0..sample_count {
      filter.push_str(&format!("[{index}:v:0]setpts=PTS-STARTPTS[sample{index}];"));
    }
    for index in 0..sample_count {
      filter.push_str(&format!("[sample{index}]"));
    }
    filter.push_str(&format!("concat=n={sample_count}:v=1:a=0"));
  }
  if let Some(scale_filter) = scale_filter {
    filter.push(',');
    filter.push_str(scale_filter);
  }
  filter.push_str("[estimated]");

  filter
}

/// Estimates compressed video size from stretches spread across the source
/// rather than assuming a screen is equally busy throughout. The seeked
/// pieces are joined before one encoder, so they pay the mandatory opening
/// I-frame once rather than once per sample. The output is raw H.264 so MP4
/// headers cannot be multiplied into the estimate. An export that follows a
/// timeline always re-encodes, as `follows_timeline` says it will.
pub fn estimate_compressed_video_bytes(
  source: &Path,
  duration_ms: u64,
  compression: u8,
  source_scale_percent: u16,
  resolution_scale_percent: u16,
  follows_timeline: bool,
) -> Result<u64, String> {
  let scale_filter = resolution_filter(source_scale_percent, resolution_scale_percent);
  let crf = export_crf(compression, scale_filter.is_some() || follows_timeline)
    .ok_or_else(|| "Original video does not need a compression estimate".to_owned())?;
  if !supports_compression() {
    return Err("This FFmpeg build does not include the H.264 encoder".to_owned());
  }
  if duration_ms == 0 {
    return Err("The recording duration is not available".to_owned());
  }

  let windows = sample_windows(duration_ms.saturating_mul(1_000));
  let temporary = estimate_temp_path();
  let mut command = ffmpeg_command();
  command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"]);
  for [start, end] in &windows {
    command
      .arg("-ss")
      .arg(format!("{:.3}", *start as f64 / 1_000_000.0))
      .arg("-t")
      .arg(format!("{:.3}", (end - start) as f64 / 1_000_000.0))
      .arg("-i")
      .arg(source);
  }
  command
    .args([
      "-filter_complex",
      &estimate_filter(windows.len(), scale_filter.as_deref()),
    ])
    .args([
      "-map",
      "[estimated]",
      "-an",
      "-c:v",
      "libx264",
      "-preset",
      "medium",
      "-crf",
    ])
    .arg(crf.to_string())
    // Joining distant screen moments can look like a scene cut that does not
    // exist in the real timeline. Each joined stretch opens on a keyframe
    // instead, which the frame count below sets apart from the rest.
    .args([
      "-sc_threshold",
      "0",
      "-pix_fmt",
      "yuv420p",
      "-profile:v",
      "high",
    ]);
  let lengths = windows.iter().map(|[start, end]| end - start);
  if let Some(starts) = stretch_starts(lengths.clone()) {
    command.arg("-force_key_frames").arg(starts);
  }
  command.args(["-f", "h264"]).arg(&temporary);
  let output = command.output().map_err(|error| {
    let _ = std::fs::remove_file(&temporary);
    format!("FFmpeg could not start the size estimate: {error}")
  })?;

  if !output.status.success() {
    let _ = std::fs::remove_file(&temporary);
    let detail = String::from_utf8_lossy(&output.stderr);
    let detail = detail.trim();
    return Err(if detail.is_empty() {
      "FFmpeg could not estimate the compressed video".to_owned()
    } else {
      format!("FFmpeg could not estimate the compressed video: {detail}")
    });
  }
  let stream = std::fs::read(&temporary);
  let _ = std::fs::remove_file(&temporary);
  let stream = stream.map_err(|error| error.to_string())?;
  let whole_us = duration_ms * 1_000;
  if windows.len() == 1 {
    return Ok(stream.len() as u64);
  }
  Ok(SampleFrames::of(&stream).whole_bytes(lengths.sum(), whole_us, X264_KEYFRAME_INTERVAL))
}

/// x264's own keyframe spacing, which the FFmpeg export leaves as it is.
const X264_KEYFRAME_INTERVAL: u64 = 250;

/// `sample_bytes`, encoded of `sampled_us` of a video, carried across all
/// `duration_us` of it.
pub(crate) fn sampled_to_whole(sample_bytes: u64, sampled_us: u64, duration_us: u64) -> u64 {
  (sample_bytes as f64 * duration_us as f64 / sampled_us.max(1) as f64).round() as u64
}
