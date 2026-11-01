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

/// The scale FFmpeg gives a recording it writes smaller: each side to the
/// nearest even, as `FrameFit::of` sizes the compositor's frame. FFmpeg can
/// only stretch to whole pixels, so the picture fills the frame, at most
/// half a pixel taller or wider at each edge than one scale would make it;
/// the compare is told so (see `FrameFit::canvas`'s callers).
pub(super) fn resolution_filter(
  source_scale_percent: u16,
  resolution_scale_percent: u16,
) -> Option<String> {
  (resolution_scale_percent < source_scale_percent).then(|| {
    let side = |input: &str| {
      format!("ceil({input}*{resolution_scale_percent}/{source_scale_percent}/2-0.5)*2")
    };
    format!("scale={}:{}:flags=lanczos", side("iw"), side("ih"))
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
/// under-count whichever they happen to land on. Measured against whole
/// encodes, fewer or shorter stretches missed a recording's bursts and
/// under-counted it by half, so the Export island gains its speed from
/// measuring every look in one pass instead.
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

/// The graph joining the `sample_count` seeked stretches and handing them to
/// one encoder per entry of `scale_filters`, labelled `estimated<index>`,
/// each scaled by its filter where it has one.
pub(super) fn estimate_filter(sample_count: usize, scale_filters: &[Option<String>]) -> String {
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
  if let [scale_filter] = scale_filters {
    if let Some(scale_filter) = scale_filter {
      filter.push(',');
      filter.push_str(scale_filter);
    }
    filter.push_str("[estimated0]");
    return filter;
  }
  filter.push_str(&format!(",split={}", scale_filters.len()));
  for index in 0..scale_filters.len() {
    filter.push_str(&format!("[joined{index}]"));
  }
  for (index, scale_filter) in scale_filters.iter().enumerate() {
    let scale_filter = scale_filter.as_deref().unwrap_or("null");
    filter.push_str(&format!(";[joined{index}]{scale_filter}[estimated{index}]"));
  }
  filter
}

/// Estimates compressed video size at each of `looks`, in order, from
/// stretches spread across the source rather than assuming a screen is
/// equally busy throughout. The seeked pieces are decoded once and joined
/// before the encoders, one per look, so they pay the mandatory opening
/// I-frame once rather than once per sample. The outputs are raw H.264 so
/// MP4 headers cannot be multiplied into the estimate. An export that
/// follows a timeline always re-encodes, as `follows_timeline` says it
/// will; every look must re-encode.
pub fn estimate_compressed_video_bytes(
  source: &Path,
  duration_ms: u64,
  looks: &[VideoExportOptions],
  follows_timeline: bool,
) -> Result<Vec<u64>, String> {
  let encodes = looks
    .iter()
    .map(|look| {
      let scale_filter =
        resolution_filter(look.source_scale_percent, look.resolution_scale_percent);
      let crf = export_crf(look.compression, scale_filter.is_some() || follows_timeline)
        .ok_or_else(|| "Original video does not need a compression estimate".to_owned())?;
      Ok((scale_filter, crf))
    })
    .collect::<Result<Vec<_>, String>>()?;
  if encodes.is_empty() {
    return Ok(Vec::new());
  }
  if !supports_compression() {
    return Err("This FFmpeg build does not include the H.264 encoder".to_owned());
  }
  if duration_ms == 0 {
    return Err("The recording duration is not available".to_owned());
  }

  let windows = sample_windows(duration_ms.saturating_mul(1_000));
  let temporaries = encodes
    .iter()
    .map(|_| estimate_temp_path())
    .collect::<Vec<_>>();
  let remove_temporaries = || {
    for temporary in &temporaries {
      let _ = std::fs::remove_file(temporary);
    }
  };
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
  let scale_filters = encodes
    .iter()
    .map(|(scale_filter, _)| scale_filter.clone())
    .collect::<Vec<_>>();
  command.args([
    "-filter_complex",
    &estimate_filter(windows.len(), &scale_filters),
  ]);
  let lengths = windows.iter().map(|[start, end]| end - start);
  let starts = stretch_starts(lengths.clone());
  for (index, ((_, crf), temporary)) in encodes.iter().zip(&temporaries).enumerate() {
    command
      .arg("-map")
      .arg(format!("[estimated{index}]"))
      .args(["-an", "-c:v", "libx264", "-preset", "medium", "-crf"])
      .arg(crf.to_string())
      // Joining distant screen moments can look like a scene cut that does
      // not exist in the real timeline. Each joined stretch opens on a
      // keyframe instead, which the frame count below sets apart from the
      // rest.
      .args([
        "-sc_threshold",
        "0",
        "-pix_fmt",
        "yuv420p",
        "-profile:v",
        "high",
      ]);
    if let Some(starts) = &starts {
      command.arg("-force_key_frames").arg(starts);
    }
    command.args(["-f", "h264"]).arg(temporary);
  }
  let output = command.output().map_err(|error| {
    remove_temporaries();
    format!("FFmpeg could not start the size estimate: {error}")
  })?;

  if !output.status.success() {
    remove_temporaries();
    let detail = String::from_utf8_lossy(&output.stderr);
    let detail = detail.trim();
    return Err(if detail.is_empty() {
      "FFmpeg could not estimate the compressed video".to_owned()
    } else {
      format!("FFmpeg could not estimate the compressed video: {detail}")
    });
  }
  let streams = temporaries
    .iter()
    .map(|temporary| std::fs::read(temporary))
    .collect::<Result<Vec<_>, _>>();
  remove_temporaries();
  let streams = streams.map_err(|error| error.to_string())?;
  let whole_us = duration_ms * 1_000;
  let sampled_us = lengths.sum();
  Ok(
    streams
      .iter()
      .map(|stream| {
        if windows.len() == 1 {
          stream.len() as u64
        } else {
          SampleFrames::of(stream).whole_bytes(sampled_us, whole_us, X264_KEYFRAME_INTERVAL)
        }
      })
      .collect(),
  )
}

/// x264's own keyframe spacing, which the FFmpeg export leaves as it is.
const X264_KEYFRAME_INTERVAL: u64 = 250;

/// `sample_bytes`, encoded of `sampled_us` of a video, carried across all
/// `duration_us` of it.
pub(crate) fn sampled_to_whole(sample_bytes: u64, sampled_us: u64, duration_us: u64) -> u64 {
  (sample_bytes as f64 * duration_us as f64 / sampled_us.max(1) as f64).round() as u64
}
