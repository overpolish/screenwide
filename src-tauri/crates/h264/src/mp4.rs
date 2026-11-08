// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Encoded frames into a fragmented MP4 as they arrive.
//!
//! The Sink Writer is given H.264 in and H.264 out, so it inserts no encoder
//! and only muxes. Fragments are written as the file grows, so a recording
//! cut short by a crash keeps what was written before it.

use std::path::Path;

use windows::core::{Interface, PCWSTR};
use windows::Win32::Media::MediaFoundation::*;

use crate::mf::{attributes, win};
use crate::{annexb, EncodedFrame, Settings};

const NANOS_PER_100NS: i64 = 100;

struct Muxer {
  media_sink: IMFMediaSink,
  writer: IMFSinkWriter,
}

pub struct Mp4Writer {
  byte_stream: IMFByteStream,
  /// The shortest fragment, in 100 ns units; Media Foundation's own choice
  /// when `None`.
  fragment_100ns: Option<u64>,
  /// Opened at the first keyframe, whose parameter sets go in the header.
  muxer: Option<Muxer>,
  /// The newest frame, held until the next one says how long it showed.
  held: Option<EncodedFrame>,
  settings: Settings,
}

impl Mp4Writer {
  /// Creates the file at `path`, replacing one already there.
  pub fn create(
    path: &Path,
    settings: Settings,
    fragment_100ns: Option<u64>,
  ) -> Result<Self, String> {
    let location = path
      .to_str()
      .ok_or_else(|| "The video's location cannot be written as text".to_owned())?
      .encode_utf16()
      .chain(Some(0))
      .collect::<Vec<_>>();
    let byte_stream = win(unsafe {
      MFCreateFile(
        MF_ACCESSMODE_WRITE,
        MF_OPENMODE_DELETE_IF_EXIST,
        MF_FILEFLAGS_NONE,
        PCWSTR(location.as_ptr()),
      )
    })?;
    Ok(Self {
      byte_stream,
      fragment_100ns,
      muxer: None,
      held: None,
      settings,
    })
  }

  /// Takes `frame`, written once the next frame arrives, or at `finish`.
  ///
  /// The fragmented MP4 sink places each fragment's last frame by its
  /// duration, not by the next frame's time, and starts the next fragment
  /// where that duration ends. A duration other than the real gap therefore
  /// moves every later frame: a camera declaring 30 fps that delivers 32
  /// gained 0.88 s in three minutes. Each frame is given the real gap to the
  /// next instead.
  pub fn write(&mut self, frame: &EncodedFrame) -> Result<(), String> {
    if self.muxer.is_none() {
      // A file has to start on a keyframe; an encoder's first frame is one.
      if !frame.keyframe {
        return Ok(());
      }
      self.muxer = Some(self.open(&frame.sample)?);
    }
    if let Some(held) = self.held.replace(frame.clone()) {
      self.mux(&held, frame.pts_ns - held.pts_ns)?;
    }
    Ok(())
  }

  fn mux(&self, frame: &EncodedFrame, duration_ns: i64) -> Result<(), String> {
    let Some(muxer) = &self.muxer else {
      return Ok(());
    };
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
    win(unsafe { sample.SetSampleTime(frame.pts_ns / NANOS_PER_100NS) })?;
    win(unsafe { sample.SetSampleDuration((duration_ns / NANOS_PER_100NS).max(1)) })?;
    win(unsafe { sample.SetUINT32(&MFSampleExtension_CleanPoint, u32::from(frame.keyframe)) })?;
    win(unsafe { muxer.writer.WriteSample(0, &sample) })
  }

  /// Writes the file's last fragment and closes it.
  pub fn finish(&mut self) -> Result<(), String> {
    // The last frame has no next one; it keeps the duration it came with.
    if let Some(held) = self.held.take() {
      self.mux(&held, held.duration_ns)?;
    }
    let muxer = self
      .muxer
      .take()
      .ok_or_else(|| "The video has no frames".to_owned())?;
    let finalized = win(unsafe { muxer.writer.Finalize() });
    let shut = win(unsafe { muxer.media_sink.Shutdown() });
    // The media sink may already have closed the file as it shut down.
    let _ = unsafe { self.byte_stream.Close() };
    finalized.and(shut)
  }

  fn open(&self, keyframe: &[u8]) -> Result<Muxer, String> {
    let Settings {
      width, height, fps, ..
    } = self.settings;
    let media_type = win(unsafe { MFCreateMediaType() })?;
    win(unsafe { media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video) })?;
    win(unsafe { media_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264) })?;
    win(unsafe {
      media_type.SetUINT64(
        &MF_MT_FRAME_SIZE,
        (u64::from(width) << 32) | u64::from(height),
      )
    })?;
    win(unsafe { media_type.SetUINT64(&MF_MT_FRAME_RATE, (u64::from(fps.max(1)) << 32) | 1) })?;
    win(unsafe { media_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1_u64 << 32) | 1) })?;
    win(unsafe {
      media_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
    })?;
    win(unsafe { media_type.SetUINT32(&MF_MT_AVG_BITRATE, self.settings.bitrate) })?;
    if let Some(sets) = annexb::parameter_sets(keyframe) {
      win(unsafe { media_type.SetBlob(&MF_MT_MPEG_SEQUENCE_HEADER, &sets) })?;
    }
    let media_sink = win(unsafe { MFCreateFMPEG4MediaSink(&self.byte_stream, &media_type, None) })?;
    let opened = (|| {
      if let Some(fragment) = self.fragment_100ns {
        let sink_attributes = win(media_sink.cast::<IMFAttributes>())?;
        win(unsafe { sink_attributes.SetUINT64(&MF_MPEG4SINK_MIN_FRAGMENT_DURATION, fragment) })?;
      }
      let settings = attributes(1)?;
      win(unsafe { settings.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1) })?;
      let writer = win(unsafe { MFCreateSinkWriterFromMediaSink(&media_sink, &settings) })?;
      // The media sink made its one stream from `media_type`; the same type
      // in means the Sink Writer passes samples through untouched.
      win(unsafe { writer.SetInputMediaType(0, &media_type, None) })?;
      win(unsafe { writer.BeginWriting() })?;
      Ok(writer)
    })();
    match opened {
      Ok(writer) => Ok(Muxer { media_sink, writer }),
      Err(error) => {
        let _ = unsafe { media_sink.Shutdown() };
        Err(error)
      }
    }
  }
}

impl Drop for Mp4Writer {
  fn drop(&mut self) {
    if let Some(muxer) = self.muxer.take() {
      drop(muxer.writer);
      let _ = unsafe { muxer.media_sink.Shutdown() };
    }
    let _ = unsafe { self.byte_stream.Close() };
  }
}
