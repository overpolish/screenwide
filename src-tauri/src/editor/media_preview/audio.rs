// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// What a track is heard as: the file it was cleaned into, if any, played
/// through the filters Auto volume levels it with, if any.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeardAs<'a> {
  pub cleaned: Option<&'a Path>,
  pub filters: Option<&'a str>,
}

/// The track's waveform as it is heard.
pub fn waveform(
  source: &Path,
  track: &RecordingAudioTrack,
  duration_ms: u64,
  heard: HeardAs<'_>,
) -> Result<Vec<f32>, String> {
  let (source, stream_index) = heard
    .cleaned
    .map_or((source, track.stream_index), |file| (file, 0));
  read_peaks(
    source,
    stream_index,
    &track.label,
    (0, duration_ms),
    WAVEFORM_POINTS,
    heard.filters,
  )
}

/// The peak level of each of `points` equal stretches of audio stream
/// `stream_index`, over `window`: a start and a length in milliseconds. A
/// window from the start reads to the end, so a recording a little longer
/// than its stated length still lands in the last stretch.
pub(in crate::editor) fn peaks(
  source: &Path,
  stream_index: usize,
  label: &str,
  window: (u64, u64),
  points: usize,
) -> Result<Vec<f32>, String> {
  read_peaks(source, stream_index, label, window, points, None)
}

fn read_peaks(
  source: &Path,
  stream_index: usize,
  label: &str,
  (start_ms, length_ms): (u64, u64),
  points: usize,
  filters: Option<&str>,
) -> Result<Vec<f32>, String> {
  let mut command = ffmpeg_command();
  command.args(["-hide_banner", "-loglevel", "error", "-nostdin"]);
  if start_ms > 0 {
    command
      .arg("-ss")
      .arg(format!("{:.3}", start_ms as f64 / 1_000.0))
      .arg("-t")
      .arg(format!("{:.3}", length_ms as f64 / 1_000.0));
  }
  command
    .arg("-i")
    .arg(source)
    .args(["-map", &format!("0:a:{stream_index}"), "-vn"]);
  if let Some(filters) = filters {
    command.args(["-af", filters]);
  }
  let mut child = command
    .args([
      "-ac",
      "1",
      "-ar",
      &WAVEFORM_SAMPLE_RATE.to_string(),
      "-f",
      "f32le",
      "pipe:1",
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| format!("FFmpeg could not be started: {error}"))?;

  let stdout = child
    .stdout
    .take()
    .ok_or_else(|| "FFmpeg did not expose its waveform output".to_owned())?;
  let expected_samples = length_ms
    .saturating_mul(WAVEFORM_SAMPLE_RATE)
    .div_ceil(1_000)
    .max(1);
  let points = points.max(1);
  let mut peaks = vec![0.0_f32; points];
  let mut reader = BufReader::new(stdout);
  let mut bytes = [0_u8; 16 * 1024];
  let mut remainder = Vec::with_capacity(3);
  let mut sample_index = 0_u64;

  loop {
    let read = reader.read(&mut bytes).map_err(|error| error.to_string())?;
    if read == 0 {
      break;
    }
    remainder.extend_from_slice(&bytes[..read]);
    let complete = remainder.len() / 4 * 4;
    for sample in remainder[..complete].as_chunks::<4>().0.iter() {
      let value = f32::from_le_bytes([sample[0], sample[1], sample[2], sample[3]]);
      let bucket = ((sample_index.saturating_mul(points as u64)) / expected_samples)
        .min((points - 1) as u64) as usize;
      peaks[bucket] = peaks[bucket].max(value.abs().min(1.0));
      sample_index = sample_index.saturating_add(1);
    }
    remainder.drain(..complete);
  }

  let output = child
    .wait_with_output()
    .map_err(|error| error.to_string())?;
  if !output.status.success() {
    let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    return Err(if detail.is_empty() {
      format!("FFmpeg could not read the {label} waveform")
    } else {
      detail
    });
  }

  Ok(peaks)
}

/// The recording's tracks with their waveforms. `cleaned` lists the tracks
/// heard cleaned, with the files they are cleaned into, and `filters` the
/// tracks Auto volume levels, with its filters, so the waveforms and the
/// meter show what is heard.
pub fn prepare(
  artifact_id: u64,
  source: &Path,
  duration_ms: u64,
  tracks: &[RecordingAudioTrack],
  cleaned: &[(usize, PathBuf)],
  filters: &[(usize, String)],
) -> Result<RecordingPreview, String> {
  let mut prepared = Vec::with_capacity(tracks.len());
  for track in tracks {
    let heard = HeardAs {
      cleaned: cleaned
        .iter()
        .find(|(stream, _)| *stream == track.stream_index)
        .map(|(_, file)| file.as_path()),
      filters: filters
        .iter()
        .find(|(stream, _)| *stream == track.stream_index)
        .map(|(_, filters)| filters.as_str()),
    };
    // Nothing is written, so a failure part-way through leaves nothing behind
    // to tidy up - only a preview the window will not show.
    prepared.push(PreparedAudioTrack {
      kind: track.kind,
      label: track.label.clone(),
      stream_index: track.stream_index,
      waveform: waveform(source, track, duration_ms, heard)?,
    });
  }

  Ok(RecordingPreview {
    artifact_id,
    tracks: prepared,
  })
}
