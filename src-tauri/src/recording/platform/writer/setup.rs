// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Writer {
  pub(in crate::recording::platform) fn new(config: WriterConfig) -> Result<Self, String> {
    let WriterConfig {
      path,
      width,
      height,
      fps,
      encoder,
      system_audio,
      microphone_format,
      stats,
      on_failure,
      container,
      primary_video,
      source,
      timeline_origin,
    } = config;
    let location = path
      .to_str()
      .ok_or_else(|| "The recording's location cannot be written as text".to_owned())?;
    let url = ns::Url::with_fs_path_str(location, false);
    let mut writer = av::AssetWriter::with_url_and_file_type(&url, container.format.file_type())
      .map_err(|error| error.to_string())?;
    // Set before any input is added, and only for a container that can take
    // it: fragmenting an .mp4 through this pipeline fails the writer outright.
    // `Container::quicktime_fragmented` carries the whole argument.
    if let Some(interval) = container.fragment_interval {
      writer.set_movie_fragment_interval(interval);
    }

    let settings = video_settings(width, height, fps, encoder);
    let mut input = av::AssetWriterInput::with_media_type_and_output_settings(
      av::MediaType::video(),
      Some(&settings),
    )
    .map_err(|error| error.to_string())?;
    // Frames arrive as fast as the screen changes and no faster, so the input
    // must not wait for a backlog it will never get.
    input.set_expects_media_data_in_real_time(true);

    let adaptor = av::asset::WriterInputPixelBufAdaptor::with_input_writer(&input, None)
      .map_err(|error| error.to_string())?;
    writer
      .add_input(&input)
      .map_err(|error| error.to_string())?;

    let (system_audio_input, system_audio_format_description) = if system_audio {
      let settings = system_audio_settings();
      let mut audio_input = av::AssetWriterInput::with_media_type_and_output_settings(
        av::MediaType::audio(),
        Some(&settings),
      )
      .map_err(|error| error.to_string())?;
      audio_input.set_expects_media_data_in_real_time(true);
      writer
        .add_input(&audio_input)
        .map_err(|error| error.to_string())?;
      (
        Some(audio_input),
        Some(microphone_format_description(MicrophoneFormat {
          channels: SYSTEM_AUDIO_CHANNELS as u16,
          sample_rate: SYSTEM_AUDIO_SAMPLE_RATE as u32,
        })?),
      )
    } else {
      (None, None)
    };

    let (microphone_input, microphone_format_description) = if let Some(format) = microphone_format
    {
      let settings = microphone_audio_settings(format);
      let mut audio_input = av::AssetWriterInput::with_media_type_and_output_settings(
        av::MediaType::audio(),
        Some(&settings),
      )
      .map_err(|error| error.to_string())?;
      audio_input.set_expects_media_data_in_real_time(true);
      writer
        .add_input(&audio_input)
        .map_err(|error| error.to_string())?;
      (
        Some(audio_input),
        Some(microphone_format_description(format)?),
      )
    } else {
      (None, None)
    };

    if !writer.start_writing() {
      return Err(asset_writer_error(
        &writer,
        "The recording could not be started",
      ));
    }
    writer.start_session_at_src_time(cm::Time::zero());

    Ok(Self {
      adaptor,
      base: Instant::now(),
      failed: None,
      last_appended_ns: None,
      height,
      input,
      last_microphone_pts_ns: None,
      last_system_audio_pts_ns: None,
      microphone_end_ns: None,
      microphone_failure_reported: false,
      microphone_format,
      microphone_format_description,
      microphone_input,
      on_failure,
      origin_source_ns: None,
      origin_wall: None,
      path,
      pending_microphone: VecDeque::new(),
      pending_system_audio: VecDeque::new(),
      primary_video,
      rejection_streak: 0,
      source,
      stats,
      system_audio_end_ns: None,
      system_audio_format_description,
      system_audio_input,
      tail: None,
      timeline: Timeline::default(),
      timeline_origin,
      width,
      writer,
    })
  }
}
