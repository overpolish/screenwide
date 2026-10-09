// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::recording_preview_player::PreviewProcessing;

/// What the output mixes FFmpeg's channels by: which tracks are on, how loud
/// each is, and which of each track's files is heard. All three change
/// while the preview plays.
#[derive(Clone)]
pub(super) struct Mix {
  pub audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  pub channels: Vec<Channel>,
  pub processing: Arc<RwLock<PreviewProcessing>>,
  pub selected_audio: Arc<RwLock<Vec<usize>>>,
}

impl Mix {
  /// Whether `channel` is heard: its track is on, and it is the channel the
  /// track is heard from, as `heard` and `leveled` say, falling back to the
  /// recorded track, and to the unleveled one, where the preview has no
  /// channel for that.
  pub(super) fn hears(
    &self,
    channel: &Channel,
    selected: &[usize],
    (heard, leveled): (&[(usize, Processing)], &[usize]),
  ) -> bool {
    let stream = channel.stream_index;
    let has = |processing: Processing, leveling: bool| {
      self.channels.iter().any(|other| {
        other.stream_index == stream
          && other.processing() == processing
          && other.leveling.is_some() == leveling
      })
    };
    let wanted = heard
      .iter()
      .find(|(heard, _)| *heard == stream)
      .map(|(_, processing)| *processing)
      .filter(|processing| has(*processing, false))
      .unwrap_or_default();
    let wanted_leveling = leveled.contains(&stream) && has(wanted, true);
    selected.contains(&stream)
      && channel.processing() == wanted
      && channel.leveling.is_some() == wanted_leveling
  }
}

pub(super) fn build_output<T>(
  device: &cpal::Device,
  config: &StreamConfig,
  queue: Arc<Mutex<VecDeque<f32>>>,
  clock: Arc<AudioClock>,
  mix: Mix,
) -> Result<Stream, String>
where
  T: SizedSample + FromSample<f32>,
{
  let output_channels = usize::from(config.channels);
  // Reused by every callback: the audio thread is not the place to allocate.
  let mut gains: Vec<Option<f32>> = Vec::with_capacity(mix.channels.len());
  device
    .build_output_stream(
      *config,
      move |output: &mut [T], info| {
        let received_at = Instant::now();
        let mut queue = queue.lock().unwrap_or_else(|value| value.into_inner());
        let selected = mix
          .selected_audio
          .read()
          .unwrap_or_else(|value| value.into_inner());
        let volumes = mix
          .audio_volumes
          .read()
          .unwrap_or_else(|value| value.into_inner());
        let processing = mix
          .processing
          .read()
          .unwrap_or_else(|value| value.into_inner());
        gains.clear();
        gains.extend(mix.channels.iter().map(|channel| {
          let heard = (processing.heard.as_slice(), processing.leveled.as_slice());
          mix.hears(channel, &selected, heard).then(|| {
            let decibels = volumes
              .iter()
              .find_map(|volume| {
                (volume.stream_index == channel.stream_index).then_some(volume.decibels)
              })
              .unwrap_or(0);
            10_f32.powf(f32::from(decibels) / 20.0)
          })
        }));
        drop(processing);
        for frame in output.chunks_mut(output_channels) {
          let mut mixed = 0.0_f32;
          for gain in &gains {
            let sample = queue.pop_front().unwrap_or(0.0);
            if let Some(gain) = gain {
              mixed += sample * gain;
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
  mix: Mix,
) -> Result<(Stream, Arc<AudioClock>, StreamConfig), String> {
  let device = cpal::default_host()
    .default_output_device()
    .ok_or_else(|| "No audio output device is available".to_owned())?;
  let supported = device
    .default_output_config()
    .map_err(|error| error.to_string())?;
  let config: StreamConfig = supported.config();
  let clock = Arc::new(AudioClock::new(config.sample_rate));
  let queue_for = || Arc::clone(&queue);
  let clock_for = || Arc::clone(&clock);
  let stream = match supported.sample_format() {
    SampleFormat::F32 => build_output::<f32>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::F64 => build_output::<f64>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::I8 => build_output::<i8>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::I16 => build_output::<i16>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::I24 => build_output::<cpal::I24>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::I32 => build_output::<i32>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::I64 => build_output::<i64>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::U8 => build_output::<u8>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::U16 => build_output::<u16>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::U24 => build_output::<cpal::U24>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::U32 => build_output::<u32>(&device, &config, queue_for(), clock_for(), mix),
    SampleFormat::U64 => build_output::<u64>(&device, &config, queue_for(), clock_for(), mix),
    format => Err(format!("Unsupported audio output format: {format}")),
  }?;
  Ok((stream, clock, config))
}
