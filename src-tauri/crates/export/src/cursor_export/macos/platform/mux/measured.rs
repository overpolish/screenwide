// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The FFmpeg run an estimate measures: the drawn frames encoded at every
//! quality asked for at once.

use super::*;

/// `video` encoded as the export encodes it into each of `outputs`, raw
/// streams for an estimate to measure. The frames are drawn once, at the
/// largest output's size, and split to every encoder, each smaller one
/// scaled down on the way. The stretches the frames join would look like
/// scene cuts to x264, so scene detection is off and each stretch after the
/// first opens on a keyframe at `stretch_starts`, which the estimate counts
/// apart from the frames between.
pub(super) fn measured_args(
  video: &RawVideo,
  outputs: &[MeasuredOutput],
  stretch_starts: Option<&str>,
) -> Vec<OsString> {
  let mut args = raw_input_args(video);
  let filtered = outputs.len() > 1
    || outputs
      .iter()
      .any(|output| output.width != video.width || output.height != video.height);
  if filtered {
    args.push("-filter_complex".into());
    args.push(measured_graph(video, outputs).into());
  }
  for (index, output) in outputs.iter().enumerate() {
    args.push("-map".into());
    args.push(if filtered {
      format!("[measured{index}]").into()
    } else {
      "0:v:0".into()
    });
    args.extend(x264_codec_args(&RawVideo {
      width: output.width,
      height: output.height,
      frame_rate: video.frame_rate,
      compression: output.compression,
      bitrate: output.bitrate,
    }));
    args.extend(strings(&["-sc_threshold", "0"]));
    if let Some(starts) = stretch_starts {
      args.extend(strings(&["-force_key_frames", starts]));
    }
    args.extend(strings(&["-f", "h264"]));
    args.push(output.path.as_os_str().to_owned());
  }
  args
}

/// The graph splitting the drawn frames to every measured output, labelled
/// `measured<index>`, scaled where the output is smaller.
fn measured_graph(video: &RawVideo, outputs: &[MeasuredOutput]) -> String {
  let mut graph = format!("[0:v:0]split={}", outputs.len());
  for index in 0..outputs.len() {
    graph.push_str(&format!("[split{index}]"));
  }
  for (index, output) in outputs.iter().enumerate() {
    let filter = if output.width == video.width && output.height == video.height {
      "null".to_owned()
    } else {
      format!("scale={}:{}:flags=lanczos", output.width, output.height)
    };
    graph.push_str(&format!(";[split{index}]{filter}[measured{index}]"));
  }
  graph
}
