// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How big the drawn video of an export comes out, measured by drawing and
//! encoding a few stretches of it exactly as the export would. x264 spends
//! by what it sees, so only the export's own frames, with their background,
//! annotations, cursor and camera, tell it. Several qualities are measured
//! from one drawing: the frames are drawn once, at the largest size asked
//! for, and FFmpeg encodes them into a stream for each quality at once.

use screenwide_timeline::{output_duration_us, within_output_windows};

use super::*;

/// The bytes the video stream of `request`'s export takes at each of
/// `looks`, in order, before its audio and its MP4 tables. An error where it
/// was cancelled, as a newer estimate replacing it does.
pub(in super::super::super) fn estimate_video_bytes(
  request: CursorExportRequest<'_>,
  looks: &[VideoExportOptions],
) -> Result<Vec<u64>, String> {
  let drawn = *looks
    .iter()
    .max_by_key(|look| look.resolution_scale_percent)
    .ok_or_else(|| "The estimate was asked for no quality".to_owned())?;
  let unscaled = (request.output.width, request.output.height);
  let drawn_request = CursorExportRequest {
    video: drawn,
    ..request
  };
  let output = export_output::scaled(&drawn_request);
  let mut request = CursorExportRequest {
    output: &output,
    on_progress: &mut *drawn_request.on_progress,
    ..drawn_request
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
  let bitrate = |(width, height): (u32, u32), compression: u8| {
    super::super::super::video_bitrate(width, height, compression)
  };
  let outputs = looks
    .iter()
    .map(|look| {
      let fit = media_preview::FrameFit::of(unscaled.0, unscaled.1, *look);
      let (width, height) = (fit.width, fit.height);
      mux::MeasuredOutput {
        path: media_preview::estimate_temp_path(),
        width,
        height,
        compression: look.compression,
        bitrate: bitrate((width, height), look.compression),
      }
    })
    .collect::<Vec<_>>();
  let output_bitrates = outputs
    .iter()
    .map(|output| output.bitrate)
    .collect::<Vec<_>>();
  // Where FFmpeg cannot be started the export would encode with
  // AVFoundation, so the estimate does too, into a file it then measures.
  let fallback = gpu_video_path();
  let drawn_bitrate = bitrate(
    (request.output.width, request.output.height),
    drawn.compression,
  );
  let x264 = mux::X264Plan {
    output: mux::X264Output::Measured {
      outputs,
      stretch_starts: media_preview::stretch_starts(piece_lengths),
    },
    width: request.output.width,
    height: request.output.height,
    compression: drawn.compression,
    bitrate: drawn_bitrate,
  };
  let cancelled = request.cancelled;
  let rendered = render_gpu_video(&mut request, (&sampled, None), &fallback, x264);
  let measured = match rendered {
    Ok((ExportRunResult::Completed, Some(stream))) => {
      stream.measure(cancelled)?.map(Measured::Streams)
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
    Measured::Streams(streams) => Ok(
      streams
        .iter()
        .map(|stream| {
          if windows.len() == 1 {
            // The whole video, drawn: its size is the answer.
            return stream.len() as u64;
          }
          let frames = media_preview::SampleFrames::of(stream);
          let frame_rate =
            (frames.frames() as f64 * 1_000_000.0 / sampled_us.max(1) as f64).round();
          frames.whole_bytes(
            sampled_us,
            duration_us,
            mux::keyframe_interval(frame_rate as u32),
          )
        })
        .collect(),
    ),
    // AVFoundation encoded the drawn quality alone, held to its bitrate;
    // the others are carried from it by theirs.
    Measured::File(bytes) => {
      let drawn_bytes = media_preview::sampled_to_whole(bytes, sampled_us, duration_us);
      Ok(
        output_bitrates
          .iter()
          .map(|bitrate| {
            (drawn_bytes as f64 * *bitrate as f64 / drawn_bitrate.max(1) as f64).round() as u64
          })
          .collect(),
      )
    }
  }
}

/// What the sampled stretches came to: x264's raw streams, whose frames can
/// be told apart, or the length of AVFoundation's file, which says nothing
/// of its frames.
enum Measured {
  Streams(Vec<Vec<u8>>),
  File(u64),
}
