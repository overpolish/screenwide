// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How big the drawn video of an export comes out, measured by drawing and
//! encoding a few stretches of it exactly as the export would. x264 spends
//! by what it sees, so only the export's own frames, with their background,
//! annotations, cursor and camera, tell it.

use screenwide_timeline::{output_duration_us, within_output_windows};

use super::*;

/// The bytes the video stream of `request`'s export takes, before its audio
/// and its MP4 tables. An error where it was cancelled, as a newer estimate
/// replacing it does.
pub(in super::super::super) fn estimate_video_bytes(
  request: CursorExportRequest<'_>,
) -> Result<u64, String> {
  let output = export_output::scaled(&request);
  let mut request = CursorExportRequest {
    output: &output,
    on_progress: &mut *request.on_progress,
    ..request
  };
  let whole = super::super::super::edited_ranges(&request);
  let duration_us = output_duration_us(&whole)
    .filter(|duration| *duration > 0)
    .ok_or_else(|| "The export has no video to estimate".to_owned())?;
  let windows = media_preview::sample_windows(duration_us);
  let sampled = within_output_windows(&whole, &windows);
  let sampled_us = output_duration_us(&sampled).unwrap_or_default();
  let piece_starts = sampled.iter().map(|piece| piece.output_start_us);
  let piece_lengths = piece_starts
    .clone()
    .skip(1)
    .chain([sampled_us])
    .zip(piece_starts)
    .map(|(end, start)| end - start);
  // Where FFmpeg cannot be started the export would encode with
  // AVFoundation, so the estimate does too, into a file it then measures.
  let fallback = gpu_video_path();
  let x264 = mux::X264Plan {
    output: mux::X264Output::Measured {
      path: media_preview::estimate_temp_path(),
      stretch_starts: media_preview::stretch_starts(piece_lengths),
    },
    width: request.output.width,
    height: request.output.height,
    compression: request.video.compression,
    bitrate: super::super::super::video_bitrate(
      request.output.width,
      request.output.height,
      request.video.compression,
    ),
  };
  let cancelled = request.cancelled;
  let rendered = render_gpu_video(&mut request, (&sampled, None), &fallback, x264);
  let measured = match rendered {
    Ok((ExportRunResult::Completed, Some(stream))) => {
      stream.measure(cancelled)?.map(Measured::Stream)
    }
    Ok((ExportRunResult::Completed, None)) => std::fs::metadata(&fallback)
      .ok()
      .map(|metadata| Measured::File(metadata.len())),
    Ok((ExportRunResult::Cancelled, _)) => None,
    Err(error) => {
      let _ = std::fs::remove_file(&fallback);
      return Err(error);
    }
  };
  let _ = std::fs::remove_file(&fallback);
  match measured.ok_or_else(|| "The size estimate was cancelled".to_owned())? {
    // The whole video, drawn: its size is the answer.
    Measured::Stream(stream) if windows.len() == 1 => Ok(stream.len() as u64),
    Measured::Stream(stream) => {
      let frames = media_preview::SampleFrames::of(&stream);
      let frame_rate = (frames.frames() as f64 * 1_000_000.0 / sampled_us.max(1) as f64).round();
      Ok(frames.whole_bytes(
        sampled_us,
        duration_us,
        mux::keyframe_interval(frame_rate as u32),
      ))
    }
    Measured::File(bytes) => Ok(media_preview::sampled_to_whole(
      bytes,
      sampled_us,
      duration_us,
    )),
  }
}

/// What the sampled stretches came to: x264's raw stream, whose frames can
/// be told apart, or the length of AVFoundation's file, which says nothing
/// of its frames.
enum Measured {
  Stream(Vec<u8>),
  File(u64),
}
