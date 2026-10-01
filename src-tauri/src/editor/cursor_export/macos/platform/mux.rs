// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use std::ffi::OsString;

pub(super) fn args(
  request: &CursorExportRequest<'_>,
  video: &Path,
  temporary: &Path,
) -> Vec<OsString> {
  let mut args = ["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"]
    .map(OsString::from)
    .to_vec();
  args.push(video.into());
  args.extend([
    OsString::from("-i"),
    request.audio_source.unwrap_or(request.screen).into(),
  ]);
  args.extend(
    [
      "-progress",
      "pipe:1",
      "-nostats",
      "-map",
      "0:v:0",
      "-c:v",
      "copy",
    ]
    .map(OsString::from),
  );
  args.extend(request.timeline.map_or_else(
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
  ));
  args.extend(
    [
      "-tag:v",
      "avc1",
      "-movflags",
      "+faststart",
      "-map_metadata",
      "-1",
      "-f",
      "mp4",
    ]
    .map(OsString::from),
  );
  args.push(temporary.into());
  args
}
