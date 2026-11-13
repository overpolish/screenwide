// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::warm_output::{Feed, Slot};
use super::*;
use crate::editor::recording_preview_player::PreviewProcessing;

/// What the output mixes FFmpeg's channels by: which tracks are on, how loud
/// each is, and which of each track's files is heard. All three change
/// while the preview plays. The songs and sound effects, where there are
/// any (`sound`), are one channel more after the tracks', always heard.
#[derive(Clone)]
pub(super) struct Mix {
  pub audio_volumes: Arc<RwLock<Vec<AudioTrackVolume>>>,
  pub channels: Vec<Channel>,
  pub processing: Arc<RwLock<PreviewProcessing>>,
  pub selected_audio: Arc<RwLock<Vec<usize>>>,
  pub sound: bool,
}

impl Mix {
  /// Whether `channel` is heard: its track is on, and it is the channel the
  /// track is heard from, as `processing` says, falling back to the recorded
  /// track, and to the unleveled one, where the preview has no channel for
  /// that. System audio makes way for a microphone only while it is on.
  pub(super) fn hears(
    &self,
    channel: &Channel,
    selected: &[usize],
    processing: &PreviewProcessing,
  ) -> bool {
    let stream = channel.stream_index;
    let has = |processing: Processing, leveling: bool| {
      self.channels.iter().any(|other| {
        other.stream_index == stream
          && other.processing() == processing
          && other.leveling.is_some() == leveling
      })
    };
    let microphone_off = |&(_, microphone): &(usize, usize)| !selected.contains(&microphone);
    let wanted = processing
      .heard
      .iter()
      .find(|(heard, _)| *heard == stream)
      .map(|(_, processing)| *processing)
      .filter(|wanted| {
        !(wanted.duck
          && processing
            .ducked_by
            .iter()
            .find(|(ducked, _)| *ducked == stream)
            .is_some_and(microphone_off))
      })
      .filter(|processing| has(*processing, false))
      .unwrap_or_default();
    let wanted_leveling = processing.leveled.contains(&stream) && has(wanted, true);
    selected.contains(&stream)
      && channel.processing() == wanted
      && channel.leveling.is_some() == wanted_leveling
  }
}

/// The stream the warm output sounds through (`warm_output.rs`): what the
/// play in `slot` hands it, mixed, or silence where no play does. A stream
/// error marks it `failed`, so the next play opens another.
pub(super) fn build_output<T>(
  device: &cpal::Device,
  config: &StreamConfig,
  slot: Slot,
  failed: Arc<AtomicBool>,
) -> Result<Stream, String>
where
  T: SizedSample + FromSample<f32>,
{
  let output_channels = usize::from(config.channels);
  // Reused by every callback: the audio thread is not the place to allocate.
  let mut gains: Vec<Option<f32>> = Vec::new();
  device
    .build_output_stream(
      *config,
      move |output: &mut [T], info| {
        let received_at = Instant::now();
        let feed = slot.lock().unwrap_or_else(|value| value.into_inner());
        let Some(Feed { clock, mix, queue }) = feed.as_ref() else {
          output.fill(T::from_sample(0.0_f32));
          return;
        };
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
          mix.hears(channel, &selected, &processing).then(|| {
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
          if mix.sound {
            mixed += queue.pop_front().unwrap_or(0.0);
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
      move |_| failed.store(true, Ordering::Release),
      None,
    )
    .map_err(|error| error.to_string())
}
