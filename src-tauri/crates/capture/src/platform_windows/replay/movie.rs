// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A clip's video, written to an MP4 exactly as it was encoded.
//!
//! The Sink Writer is given H.264 in and H.264 out, so it inserts no encoder
//! and only muxes. Unlike an elementary stream handed to FFmpeg, every frame
//! keeps its own time: a replay's frames are not evenly spaced, since a
//! quiet screen sends none and the frame at a save lands when it was made.

use std::path::Path;

use windows::core::PCWSTR;
use windows::Win32::Media::MediaFoundation::*;

use super::super::writer::{attributes, win, NANOS_PER_100NS};
use super::{annexb, VideoTake};

/// Writes `video`'s frames to `path`, timed from `start_ns`, the clip's
/// first instant, through `end_ns`, its last.
pub(super) fn write_video(
  path: &Path,
  video: &VideoTake,
  start_ns: i64,
  end_ns: i64,
) -> Result<(), String> {
  let written = write(path, video, start_ns, end_ns);
  if written.is_err() {
    let _ = std::fs::remove_file(path);
  }
  written
}

fn write(path: &Path, video: &VideoTake, start_ns: i64, end_ns: i64) -> Result<(), String> {
  let first = video
    .frames
    .first()
    .ok_or_else(|| "The clip has no video".to_owned())?;
  let settings = attributes(2)?;
  win(unsafe { settings.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1) })?;
  win(unsafe { settings.SetGUID(&MF_TRANSCODE_CONTAINERTYPE, &MFTranscodeContainerType_MPEG4) })?;
  let location = path
    .to_str()
    .ok_or_else(|| "The clip's location cannot be written as text".to_owned())?
    .encode_utf16()
    .chain(Some(0))
    .collect::<Vec<_>>();
  let writer =
    win(unsafe { MFCreateSinkWriterFromURL(PCWSTR(location.as_ptr()), None, &settings) })?;

  let media_type = win(unsafe { MFCreateMediaType() })?;
  let packed_size = (u64::from(video.width) << 32) | u64::from(video.height);
  win(unsafe { media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video) })?;
  win(unsafe { media_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264) })?;
  win(unsafe { media_type.SetUINT64(&MF_MT_FRAME_SIZE, packed_size) })?;
  win(unsafe { media_type.SetUINT64(&MF_MT_FRAME_RATE, (u64::from(video.fps) << 32) | 1) })?;
  win(unsafe { media_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1_u64 << 32) | 1) })?;
  win(unsafe {
    media_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
  })?;
  if let Some(sets) = annexb::parameter_sets(&first.sample) {
    win(unsafe { media_type.SetBlob(&MF_MT_MPEG_SEQUENCE_HEADER, &sets) })?;
  }
  let stream = win(unsafe { writer.AddStream(&media_type) })?;
  win(unsafe { writer.SetInputMediaType(stream, &media_type, None) })?;
  win(unsafe { writer.BeginWriting() })?;

  let frame_ns = 1_000_000_000 / i64::from(video.fps.max(1));
  for (index, frame) in video.frames.iter().enumerate() {
    let next_ns = video
      .frames
      .get(index + 1)
      .map_or(end_ns.max(frame.pts_ns + frame_ns), |next| next.pts_ns);
    let length =
      u32::try_from(frame.sample.len()).map_err(|_| "A frame is too large".to_owned())?;
    let buffer = win(unsafe { MFCreateMemoryBuffer(length) })?;
    let mut data = std::ptr::null_mut();
    win(unsafe { buffer.Lock(&mut data, None, None) })?;
    // SAFETY: the buffer was created to hold exactly `length` bytes.
    unsafe { std::ptr::copy_nonoverlapping(frame.sample.as_ptr(), data, frame.sample.len()) };
    win(unsafe { buffer.Unlock() })?;
    win(unsafe { buffer.SetCurrentLength(length) })?;
    let sample = win(unsafe { MFCreateSample() })?;
    win(unsafe { sample.AddBuffer(&buffer) })?;
    let time = frame.pts_ns.saturating_sub(start_ns).max(0) / NANOS_PER_100NS;
    let duration = (next_ns - frame.pts_ns).max(1) / NANOS_PER_100NS;
    win(unsafe { sample.SetSampleTime(time) })?;
    win(unsafe { sample.SetSampleDuration(duration.max(1)) })?;
    win(unsafe { sample.SetUINT32(&MFSampleExtension_CleanPoint, u32::from(frame.keyframe)) })?;
    win(unsafe { writer.WriteSample(stream, &sample) })?;
  }
  win(unsafe { writer.Finalize() })
}
