// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whole FFmpeg commands for an export that follows a timeline: the inputs,
//! the filter graph `filter_args` builds, and the file they are written to.

use super::*;

fn base_args(source: &Path) -> Vec<OsString> {
  let mut args: Vec<OsString> = ["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"]
    .map(OsString::from)
    .into();
  args.push(source.into());
  args
}

fn finish(args: Vec<OsString>, destination: &Path) -> Vec<OsString> {
  finish_as(args, destination, &EXPORT_MP4_OUTPUT)
}

fn finish_as(mut args: Vec<OsString>, destination: &Path, output: &[&str]) -> Vec<OsString> {
  args.extend(output.iter().map(OsString::from));
  args.push(destination.into());
  args
}

pub(crate) fn timeline_audio_mapping_args(
  timeline: &TimelinePlan,
  audio_input: usize,
  selection: &TrackSelection,
  layout: AudioLayout,
) -> Vec<OsString> {
  filter_args(
    timeline,
    None,
    audio_input,
    selection,
    layout,
    AudioEncoding::Aac,
    VideoExportOptions {
      compression: 0,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
  )
}

pub(crate) fn timeline_selected_export_args(
  source: &Path,
  destination: &Path,
  selection: &TrackSelection,
  layout: AudioLayout,
  video: VideoExportOptions,
  timeline: &TimelinePlan,
) -> Vec<OsString> {
  let mut args = base_args(source);
  args.extend(["-progress", "pipe:1", "-nostats"].map(OsString::from));
  args.extend(filter_args(
    timeline,
    Some(0),
    0,
    selection,
    layout,
    AudioEncoding::Aac,
    video,
  ));
  finish(args, destination)
}

pub(crate) fn timeline_camera_export_args(
  audio_source: &Path,
  camera_source: &Path,
  destination: &Path,
  selection: &TrackSelection,
  layout: AudioLayout,
  video: VideoExportOptions,
  timeline: &TimelinePlan,
) -> Vec<OsString> {
  let mut args = base_args(audio_source);
  args.push(OsString::from("-i"));
  args.push(camera_source.into());
  args.extend(["-progress", "pipe:1", "-nostats"].map(OsString::from));
  args.extend(filter_args(
    timeline,
    Some(1),
    0,
    selection,
    layout,
    AudioEncoding::Aac,
    video,
  ));
  finish(args, destination)
}

pub(crate) fn timeline_audio_export_args(
  source: &Path,
  destination: &Path,
  selection: &TrackSelection,
  layout: AudioLayout,
  timeline: &TimelinePlan,
  file: AudioFile,
) -> Vec<OsString> {
  let mut args = base_args(source);
  args.extend(["-progress", "pipe:1", "-nostats"].map(OsString::from));
  args.extend(filter_args(
    timeline,
    None,
    0,
    selection,
    layout,
    file.encoding(),
    VideoExportOptions {
      compression: 0,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
  ));
  finish_as(args, destination, file.output_args())
}
