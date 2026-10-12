// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Sink {
  pub(super) fn new(config: &WriterConfig) -> Result<Self, String> {
    let mut reset_token = 0;
    let mut manager = None;
    unsafe { MFCreateDXGIDeviceManager(&mut reset_token, &mut manager) }
      .map_err(|error| error.to_string())?;
    let manager = manager.ok_or_else(|| "Media Foundation created no D3D manager".to_owned())?;
    unsafe { manager.ResetDevice(&config.device, reset_token) }
      .map_err(|error| error.to_string())?;

    let attributes = attributes(5)?;
    win(unsafe {
      attributes.SetGUID(
        &MF_TRANSCODE_CONTAINERTYPE,
        &MFTranscodeContainerType_FMPEG4,
      )
    })?;
    win(unsafe { attributes.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1) })?;
    win(unsafe { attributes.SetUINT32(&MF_LOW_LATENCY, 1) })?;
    win(unsafe { attributes.SetUnknown(&MF_SINK_WRITER_D3D_MANAGER, &manager) })?;
    let path = config
      .path
      .to_str()
      .ok_or_else(|| "The recording path is not valid UTF-8".to_owned())?;
    let wide = path.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let output = video_type(MFVideoFormat_H264, config.width, config.height, config.fps)?;
    win(unsafe {
      output.SetUINT32(
        &MF_MT_AVG_BITRATE,
        u32::try_from(bitrate_bps(config.width, config.height, config.fps)).unwrap_or(u32::MAX),
      )
    })?;
    win(unsafe { output.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_High.0 as u32) })?;
    win(unsafe { output.SetUINT32(&MF_MT_MAX_KEYFRAME_SPACING, (config.fps / 2).max(1)) })?;
    let byte_stream = win(unsafe {
      MFCreateFile(
        MF_ACCESSMODE_WRITE,
        MF_OPENMODE_DELETE_IF_EXIST,
        MF_FILEFLAGS_NONE,
        PCWSTR(wide.as_ptr()),
      )
    })?;
    let media_sink = win(unsafe { MFCreateFMPEG4MediaSink(&byte_stream, &output, None) })?;
    let sink_attributes = media_sink
      .cast::<IMFAttributes>()
      .map_err(|error| error.to_string())?;
    // Half-second fragments align with the working encoder's half-second GOP,
    // bounding crash loss while keeping seeks cheap in the export scrubber.
    win(unsafe { sink_attributes.SetUINT64(&MF_MPEG4SINK_MIN_FRAGMENT_DURATION, 5_000_000) })?;
    // MF_MT_MAX_KEYFRAME_SPACING above is advisory and the Microsoft AVC DX12
    // Encoder HMFT (Windows 11 D3D12 shim) ignores it, as it does ICodecAPI
    // calls made after SetInputMediaType and per-sample IDR requests. Codec
    // API values handed over as MF_SINK_WRITER_ENCODER_CONFIG reach the
    // encoder before its media types are negotiated and are honoured, giving
    // working recordings the half-second GOP the export scrubber relies on.
    match encoder_config((config.fps / 2).max(1)) {
      Ok(store) => win(unsafe { attributes.SetUnknown(&MF_SINK_WRITER_ENCODER_CONFIG, &store) })?,
      Err(message) => eprintln!("windows recorder: encoder config not applied: {message}"),
    }
    let sink = win(unsafe { MFCreateSinkWriterFromMediaSink(&media_sink, &attributes) })?;
    // MFCreateFMPEG4MediaSink has already created the fixed video stream. The
    // Sink Writer must feed that stream so its encoder can update the sink's
    // media type with the generated H.264 sequence header.
    let stream = 0;

    let input = video_type(
      MFVideoFormat_ARGB32,
      config.width,
      config.height,
      config.fps,
    )?;
    win(unsafe { sink.SetInputMediaType(stream, &input, None) })?;
    win(unsafe { sink.BeginWriting() })?;

    Ok(Self {
      _byte_stream: byte_stream,
      _device_manager: manager,
      media_sink,
      sink: Some(sink),
      stream,
    })
  }

  pub(super) fn write(
    &self,
    frame: &Frame,
    pts_100ns: i64,
    duration_100ns: i64,
  ) -> Result<(), String> {
    let buffer =
      unsafe { MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, &frame.texture, 0, false) }
        .map_err(|error| error.to_string())?;
    // A DXGI surface buffer starts with a current length of zero. Sink Writer
    // treats that as an empty video sample even though the texture itself is
    // valid, and the H.264 transform rejects it with E_INVALIDARG. Preserve
    // the GPU surface and describe its payload through IMF2DBuffer; this does
    // not copy or map the frame back to the CPU.
    let two_dimensional = buffer
      .cast::<IMF2DBuffer>()
      .map_err(|error| error.to_string())?;
    let length =
      unsafe { two_dimensional.GetContiguousLength() }.map_err(|error| error.to_string())?;
    win(unsafe { buffer.SetCurrentLength(length) })?;
    let sample = unsafe { MFCreateSample() }.map_err(|error| error.to_string())?;
    win(unsafe { sample.AddBuffer(&buffer) })?;
    win(unsafe { sample.SetSampleTime(pts_100ns) })?;
    win(unsafe { sample.SetSampleDuration(duration_100ns.max(1)) })?;
    let sink = self
      .sink
      .as_ref()
      .ok_or_else(|| "The recording has already been finalized".to_owned())?;
    unsafe { sink.WriteSample(self.stream, &sample) }.map_err(|error| {
      let mut description = D3D11_TEXTURE2D_DESC::default();
      unsafe { frame.texture.GetDesc(&mut description) };
      format!(
        "{error} (sample time {pts_100ns} / duration {duration_100ns} x100ns; texture {}x{} format {:?} bind 0x{:x} misc 0x{:x} usage {:?})",
        description.Width,
        description.Height,
        description.Format,
        description.BindFlags,
        description.MiscFlags,
        description.Usage,
      )
    })?;
    Ok(())
  }

  pub(super) fn finish(&mut self) -> Result<(), String> {
    let sink = self
      .sink
      .take()
      .ok_or_else(|| "The recording has already been finalized".to_owned())?;
    win(unsafe { sink.Finalize() })?;
    drop(sink);
    win(unsafe { self.media_sink.Shutdown() })
  }
}

impl Drop for Sink {
  fn drop(&mut self) {
    self.sink.take();
    let _ = unsafe { self.media_sink.Shutdown() };
  }
}
