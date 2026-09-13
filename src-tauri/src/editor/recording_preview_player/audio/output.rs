// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn build_output<T>(
  device: &cpal::Device,
  config: &StreamConfig,
  queue: Arc<Mutex<VecDeque<f32>>>,
  clock: Arc<AudioClock>,
  selected_audio: Arc<RwLock<Vec<usize>>>,
  audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  stream_indices: Vec<usize>,
) -> Result<Stream, String>
where
  T: SizedSample + FromSample<f32>,
{
  let output_channels = usize::from(config.channels);
  let track_count = stream_indices.len();
  device
    .build_output_stream(
      *config,
      move |output: &mut [T], info| {
        let received_at = Instant::now();
        let mut queue = queue.lock().unwrap_or_else(|value| value.into_inner());
        let selected = selected_audio
          .read()
          .unwrap_or_else(|value| value.into_inner());
        let volumes = audio_volumes
          .read()
          .unwrap_or_else(|value| value.into_inner());
        for frame in output.chunks_mut(output_channels) {
          let mut mixed = 0.0_f32;
          for stream_index in stream_indices.iter().take(track_count) {
            let sample = queue.pop_front().unwrap_or(0.0);
            if selected.contains(stream_index) {
              let decibels = volumes
                .iter()
                .find_map(|volume| {
                  (volume.stream_index == *stream_index).then_some(volume.decibels)
                })
                .unwrap_or(0);
              mixed += sample * 10_f32.powf(f32::from(decibels) / 20.0);
            }
          }
          let mixed = mixed.clamp(-1.0, 1.0);
          for sample in frame {
            *sample = T::from_sample(mixed);
          }
        }
        clock.submit(
          info.timestamp(),
          received_at,
          output.len() / output_channels,
        );
      },
      |_| {},
      None,
    )
    .map_err(|error| error.to_string())
}

pub(super) fn output_stream(
  queue: Arc<Mutex<VecDeque<f32>>>,
  selected_audio: Arc<RwLock<Vec<usize>>>,
  audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  stream_indices: Vec<usize>,
) -> Result<(Stream, Arc<AudioClock>, StreamConfig), String> {
  let device = cpal::default_host()
    .default_output_device()
    .ok_or_else(|| "No audio output device is available".to_owned())?;
  let supported = device
    .default_output_config()
    .map_err(|error| error.to_string())?;
  let config: StreamConfig = supported.config();
  let clock = Arc::new(AudioClock::new(config.sample_rate));
  let build = |format| match format {
    SampleFormat::F32 => build_output::<f32>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::F64 => build_output::<f64>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::I8 => build_output::<i8>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::I16 => build_output::<i16>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::I24 => build_output::<cpal::I24>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::I32 => build_output::<i32>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::I64 => build_output::<i64>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::U8 => build_output::<u8>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::U16 => build_output::<u16>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::U24 => build_output::<cpal::U24>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::U32 => build_output::<u32>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    SampleFormat::U64 => build_output::<u64>(
      &device,
      &config,
      Arc::clone(&queue),
      Arc::clone(&clock),
      Arc::clone(&selected_audio),
      Arc::clone(&audio_volumes),
      stream_indices.clone(),
    ),
    format => Err(format!("Unsupported audio output format: {format}")),
  };
  let stream = build(supported.sample_format())?;
  Ok((stream, clock, config))
}
