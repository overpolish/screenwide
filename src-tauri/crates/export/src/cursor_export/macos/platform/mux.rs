// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The FFmpeg run that writes the exported file: the drawn video, either
//! encoded already or as raw frames for x264, beside the recording's audio.

use super::*;
use std::ffi::OsString;

/// How many threads x264 encodes with. Four more than doubled the media
/// engine at 4K (143 against 60 frames a second on an M4 Pro) while keeping
/// about a quarter of its cores busy; every core was twice as fast again but
/// ran a laptop hot, and at 1080p four already outran decoding the recording.
const X264_THREADS: &str = "4";

/// The x264 quality for each compression level, the media engine's equal or
/// better. Measured against a lossless export of a 4K screen recording and a
/// 1080p mesh background, the media engine barely told its levels apart on
/// either, undershooting every bitrate it was given, so these sit close
/// together: from 3 dB better at the same size for the best level to the
/// same quality in a fifth smaller file for the smallest.
fn x264_crf(compression: u8) -> u8 {
  match compression {
    0 => 6,
    1 => 8,
    2 => 10,
    3 => 11,
    _ => 12,
  }
}

/// The export's audio: the recording it is read from and how its tracks are
/// mapped into the file, the drawn video being the first input.
pub(super) struct Audio {
  input: OsString,
  mapping: Vec<OsString>,
}

/// Raw frames written to FFmpeg's standard input: NV12, each plane's rows
/// packed, at a constant rate.
struct RawVideo {
  width: u32,
  height: u32,
  frame_rate: u32,
  compression: u8,
  bitrate: u64,
}

/// An export encoded with x264, waiting for the loop to know its frame rate.
pub(super) struct X264Plan {
  pub(super) audio: Audio,
  pub(super) width: u32,
  pub(super) height: u32,
  pub(super) compression: u8,
  /// The bitrate the media engine would have been given, which caps x264's.
  pub(super) bitrate: u64,
  /// Where FFmpeg writes until the file is whole; see `remux_temp_path`.
  pub(super) temporary: PathBuf,
}

impl X264Plan {
  /// FFmpeg started on the export at `frame_rate`, waiting for its frames.
  pub(super) fn start(self, frame_rate: u32) -> Result<media_preview::StreamedExport, String> {
    let video = RawVideo {
      width: self.width,
      height: self.height,
      frame_rate,
      compression: self.compression,
      bitrate: self.bitrate,
    };
    media_preview::StreamedExport::start(
      self.audio.x264_args(&video, &self.temporary),
      &self.temporary,
    )
  }
}

impl Audio {
  pub(super) fn of(request: &CursorExportRequest<'_>) -> Self {
    Self {
      input: request.audio_source.unwrap_or(request.screen).into(),
      mapping: request.timeline.map_or_else(
        || {
          request
            .selection
            .audio_args_from(request.audio_layout, 1)
            .into_iter()
            .map(OsString::from)
            .collect()
        },
        |timeline| {
          media_preview::timeline_audio_mapping_args(
            timeline,
            1,
            request.selection,
            request.audio_layout,
          )
        },
      ),
    }
  }

  /// The encoded `video` copied beside the audio into `temporary`, with
  /// FFmpeg's progress on its standard output.
  pub(super) fn remux_args(&self, video: &Path, temporary: &Path) -> Vec<OsString> {
    let mut args = strings(&["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"]);
    args.push(video.into());
    let output = strings(&[
      "-progress",
      "pipe:1",
      "-nostats",
      "-map",
      "0:v:0",
      "-c:v",
      "copy",
    ]);
    self.finish(args, output, temporary)
  }

  /// `video` read from standard input and encoded with x264 beside the audio
  /// into `temporary`. The colour stays untagged, as the media engine's
  /// files were, so players read the BT.709 frames the same way.
  fn x264_args(&self, video: &RawVideo, temporary: &Path) -> Vec<OsString> {
    let mut args = strings(&["-hide_banner", "-loglevel", "error", "-y"]);
    args.extend(strings(&["-f", "rawvideo", "-pix_fmt", "nv12", "-s"]));
    args.push(format!("{}x{}", video.width, video.height).into());
    args.push("-framerate".into());
    args.push(video.frame_rate.to_string().into());
    args.extend(strings(&["-i", "pipe:0"]));
    let mut codec = strings(&[
      "-map",
      "0:v:0",
      "-c:v",
      "libx264",
      "-preset",
      "veryfast",
      "-threads",
      X264_THREADS,
      "-profile:v",
      "high",
    ]);
    // Constant quality spends next to nothing on a still screen; the cap, half
    // again the media engine's bitrate over two seconds, holds a busy stretch
    // to the size its level promises.
    for (flag, value) in [
      ("-crf", u64::from(x264_crf(video.compression))),
      ("-maxrate", video.bitrate * 3 / 2),
      ("-bufsize", video.bitrate * 2),
      // A keyframe at least every four seconds, as the media engine wrote.
      ("-g", u64::from(video.frame_rate) * 4),
    ] {
      codec.push(flag.into());
      codec.push(value.to_string().into());
    }
    self.finish(args, codec, temporary)
  }

  /// `inputs` with the audio after them, then `video`, which maps and
  /// encodes the first input's picture, the audio's mapping, and the MP4
  /// written to `temporary`.
  fn finish(
    &self,
    mut args: Vec<OsString>,
    video: Vec<OsString>,
    temporary: &Path,
  ) -> Vec<OsString> {
    args.push("-i".into());
    args.push(self.input.clone());
    args.extend(video);
    args.extend(self.mapping.iter().cloned());
    args.extend(strings(&[
      "-tag:v",
      "avc1",
      "-movflags",
      "+faststart",
      "-map_metadata",
      "-1",
      "-f",
      "mp4",
    ]));
    args.push(temporary.into());
    args
  }
}

fn strings(values: &[&str]) -> Vec<OsString> {
  values.iter().map(OsString::from).collect()
}
