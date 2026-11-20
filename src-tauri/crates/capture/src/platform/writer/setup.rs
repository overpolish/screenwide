// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Writer {
  pub(crate) fn new(config: WriterConfig) -> Result<Self, String> {
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

    let system_audio = system_audio
      .then(|| AudioTrack::add_to(&mut writer, &system_audio_settings(), SYSTEM_AUDIO_FORMAT))
      .transpose()?;
    let microphone = microphone_format
      .map(|format| AudioTrack::add_to(&mut writer, &microphone_audio_settings(format), format))
      .transpose()?;

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
      microphone,
      microphone_failure_reported: false,
      on_failure,
      origin_source_ns: None,
      origin_wall: None,
      picture: Self::picture_tap(&path),
      path,
      pending_microphone: VecDeque::new(),
      pending_system_audio: VecDeque::new(),
      primary_video,
      rejection_streak: 0,
      source,
      stats,
      system_audio,
      tail: None,
      timeline: Timeline::default(),
      timeline_origin,
      width,
      writer,
    })
  }
}
