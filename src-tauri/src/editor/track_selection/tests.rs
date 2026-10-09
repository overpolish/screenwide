// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::AudioTrackKind;

fn tracks(count: usize) -> Vec<RecordingAudioTrack> {
  (0..count)
    .map(|stream_index| RecordingAudioTrack {
      kind: AudioTrackKind::Unknown,
      label: format!("Audio {}", stream_index + 1),
      stream_index,
    })
    .collect()
}

#[test]
fn keeps_the_recordings_own_order_whatever_order_the_window_sent() {
  let selection = TrackSelection::new(&tracks(3), &[2, 0]);
  assert_eq!(selection.stream_indices, vec![0, 2]);
}

#[test]
fn drops_a_track_this_recording_does_not_have() {
  let selection = TrackSelection::new(&tracks(1), &[0, 7]);
  assert_eq!(selection.stream_indices, vec![0]);
  assert!(selection.covers(&tracks(1)));
}

#[test]
fn maps_the_empty_selection_to_no_audio() {
  let selection = TrackSelection::new(&tracks(2), &[]);
  assert!(selection.stream_indices.is_empty());
  assert_eq!(
    selection.audio_args(AudioLayout::Mixdown),
    vec!["-an".to_owned()]
  );
  assert_eq!(
    selection.audio_args(AudioLayout::SeparateTracks),
    vec!["-an".to_owned()]
  );
}

#[test]
fn volume_changes_require_processing_and_filter_the_selected_track() {
  let selection = TrackSelection::with_volumes(
    &tracks(2),
    &[0, 1],
    &[AudioTrackVolume {
      decibels: -6,
      stream_index: 1,
    }],
  )
  .unwrap();
  assert!(selection.needs_processing(&tracks(2), AudioLayout::SeparateTracks));
  let args = selection.audio_args(AudioLayout::SeparateTracks).join(" ");
  assert!(args.contains("[0:a:0]volume=0dB[track0]"));
  assert!(args.contains("[0:a:1]volume=-6dB[track1]"));
  assert!(args.contains("-map [track0] -map [track1]"));
}

#[test]
fn plays_a_cleaned_track_from_its_file_instead_of_copying_it() {
  let selection = TrackSelection::new(&tracks(2), &[0, 1])
    .with_cleaned(vec![(1, PathBuf::from("/tmp/noise-1.flac"))]);
  assert!(selection.needs_processing(&tracks(2), AudioLayout::SeparateTracks));
  let args = selection.audio_args(AudioLayout::SeparateTracks).join(" ");
  assert!(args.contains("amovie=/tmp/noise-1.flac[clean1];"));
  assert!(args.contains("[0:a:0]volume=0dB[track0]"));
  assert!(args.contains("[clean1]volume=0dB[track1]"));
}

#[test]
fn levels_a_track_after_its_cleaned_file_and_before_its_volume() {
  let selection = TrackSelection::with_volumes(
    &tracks(2),
    &[0, 1],
    &[AudioTrackVolume {
      stream_index: 1,
      decibels: -6,
    }],
  )
  .unwrap()
  .with_cleaned(vec![(1, PathBuf::from("/tmp/noise-1.flac"))])
  .with_filters(vec![(1, "acompressor,alimiter".to_owned())]);
  let args = selection.audio_args(AudioLayout::Mixdown).join(" ");
  assert!(args.contains("amovie=/tmp/noise-1.flac[clean1];[clean1]acompressor,alimiter[heard1];"));
  assert!(args.contains("[heard1]volume=-6dB[track1]"));
  // A track only leveled is read from the recording.
  let selection = TrackSelection::new(&tracks(2), &[0, 1])
    .with_filters(vec![(1, "acompressor,alimiter".to_owned())]);
  assert!(selection.needs_processing(&tracks(2), AudioLayout::SeparateTracks));
  let args = selection.audio_args(AudioLayout::SeparateTracks).join(" ");
  assert!(args.contains("[0:a:1]acompressor,alimiter[heard1];[0:a:0]volume=0dB[track0]"));
}

#[test]
fn ignores_a_cleaned_file_for_a_track_left_out() {
  let selection = TrackSelection::new(&tracks(2), &[0])
    .with_cleaned(vec![(1, PathBuf::from("/tmp/noise-1.flac"))]);
  assert_eq!(
    selection.audio_args(AudioLayout::SeparateTracks),
    ["-map", "0:a:0", "-c:a", "copy"]
  );
}

#[test]
fn escapes_a_path_for_both_readings_of_a_filter_graph() {
  assert_eq!(
    arguments::filter_path(std::path::Path::new("/a b/it's, [x];y:z.flac")),
    r"/a b/it\\\'s\, \[x\]\;y\\:z.flac"
  );
}

#[test]
fn rejects_volume_outside_the_inspector_range() {
  assert_eq!(
    TrackSelection::with_volumes(
      &tracks(1),
      &[0],
      &[AudioTrackVolume {
        decibels: 13,
        stream_index: 0
      }],
    ),
    Err("Audio volume must be between -60 dB and +12 dB".to_owned())
  );
}

#[test]
fn copies_a_lone_track_instead_of_re_encoding_it() {
  let selection = TrackSelection::new(&tracks(2), &[1]);
  assert_eq!(
    selection.audio_args(AudioLayout::Mixdown),
    ["-map", "0:a:1", "-c:a", "copy"]
  );
}

#[test]
fn sums_without_letting_amix_halve_either_track() {
  let selection = TrackSelection::new(&tracks(2), &[0, 1]);
  assert_eq!(
    selection.audio_args(AudioLayout::Mixdown),
    [
      "-filter_complex",
      "[0:a:0][0:a:1]amix=inputs=2:normalize=0[mix]",
      "-map",
      "[mix]",
      "-c:a",
      "aac",
      "-b:a",
      MIXDOWN_BITRATE
    ]
  );
}

#[test]
fn keeps_every_selected_track_separate_for_an_export() {
  let selection = TrackSelection::new(&tracks(3), &[0, 2]);
  assert_eq!(
    selection.audio_args(AudioLayout::SeparateTracks),
    ["-map", "0:a:0", "-map", "0:a:2", "-c:a", "copy"]
  );
}

#[test]
fn knows_when_something_was_left_out() {
  assert!(!TrackSelection::new(&tracks(2), &[0]).covers(&tracks(2)));
  assert!(TrackSelection::new(&tracks(2), &[0, 1]).covers(&tracks(2)));
}

#[test]
fn only_processes_an_export_when_its_contents_or_layout_change() {
  let all = TrackSelection::new(&tracks(2), &[0, 1]);
  assert!(!all.needs_processing(&tracks(2), AudioLayout::SeparateTracks));
  assert!(all.needs_processing(&tracks(2), AudioLayout::Mixdown));
  let one = TrackSelection::new(&tracks(2), &[0]);
  assert!(one.needs_processing(&tracks(2), AudioLayout::SeparateTracks));
  assert!(one.needs_processing(&tracks(2), AudioLayout::Mixdown));
  let only = TrackSelection::new(&tracks(1), &[0]);
  assert!(!only.needs_processing(&tracks(1), AudioLayout::Mixdown));
}

#[test]
fn estimates_selected_and_collapsed_audio_from_their_real_bitrates() {
  let typed = vec![
    RecordingAudioTrack {
      kind: AudioTrackKind::SystemAudio,
      label: "System audio".to_owned(),
      stream_index: 0,
    },
    RecordingAudioTrack {
      kind: AudioTrackKind::Microphone,
      label: "Microphone".to_owned(),
      stream_index: 1,
    },
  ];
  let both = TrackSelection::new(&typed, &[0, 1]);
  assert_eq!(
    both.estimated_audio_bytes(&typed, AudioLayout::SeparateTracks, 10_000),
    400_000
  );
  assert_eq!(
    both.estimated_audio_bytes(&typed, AudioLayout::Mixdown, 10_000),
    240_000
  );
  let microphone = TrackSelection::new(&typed, &[1]);
  assert_eq!(
    microphone.estimated_audio_bytes(&typed, AudioLayout::SeparateTracks, 10_000),
    160_000
  );
}
