// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native Media Foundation timeline thumbnails.

use tauri::ipc::{Channel, InvokeResponseBody};

use crate::editor::recording_preview_player::{
  timeline_thumbnails::{
    cell_centre_ms, cell_tolerance_ms, encoded_png, payload, thumbnail_tracks, ThumbnailTrack,
    THUMBNAIL_HEIGHT,
  },
  PlayerSources,
};
use screenwide_export::camera_effects::cpu::StillEffects;
use screenwide_export::decode::windows::decoder::{encoded_jpeg, NativeVideoReader};

pub(super) fn generate(sources: PlayerSources, count: u32, channel: Channel) {
  let recording_ms = sources.duration_ms;
  // Each track on a thread of its own, so the camera's strip fills alongside
  // the screen's rather than after it.
  std::thread::scope(|scope| {
    for track in thumbnail_tracks(&sources) {
      let channel = &channel;
      scope.spawn(move || generate_track(track, recording_ms, count, channel));
    }
  });
}

fn generate_track(track: ThumbnailTrack<'_>, recording_ms: u64, count: u32, channel: &Channel) {
  let Ok(mut reader) = NativeVideoReader::open(track.path, track.width, THUMBNAIL_HEIGHT, 0) else {
    return;
  };
  let mut effects = StillEffects::new(track.effects, THUMBNAIL_HEIGHT * 4);
  let see_through = effects.as_ref().is_some_and(StillEffects::is_transparent);
  let tolerance_ms = cell_tolerance_ms(recording_ms, count);
  let mut last_thumbnail = None;
  for index in 0..count {
    let target_ms =
      cell_centre_ms(recording_ms, index, count).min(track.duration_ms.saturating_sub(1));
    // Any frame from half a cell before the centre will do, so the seek does
    // not rewind: most thumbnails are the keyframe it lands on, or a few
    // frames after it.
    let decoded = reader
      .seek_near(target_ms)
      .and_then(|()| reader.timed_frame_at(target_ms.saturating_sub(tolerance_ms)))
      .ok()
      .flatten()
      .and_then(|(mut frame, frame_ms)| {
        if let Some(effects) = effects.as_mut() {
          // The matte is read for the moment the frame is from, which the
          // tolerance lets differ from the cell's centre.
          effects.apply(&mut frame, frame_ms);
        }
        let encoded = if see_through {
          encoded_png(&frame)
        } else {
          encoded_jpeg(&frame, 80)
        };
        encoded.ok()
      });
    // Sidecars can end a fraction before the shared recording timeline.
    // Keep the strip complete with the most recent real camera frame rather
    // than exposing the frontend's grey missing-thumbnail placeholder.
    let Some(bytes) = thumbnail_or_previous(decoded, &last_thumbnail) else {
      continue;
    };
    let body = InvokeResponseBody::Raw(payload(track.id, index, count, &bytes));
    last_thumbnail = Some(bytes);
    if channel.send(body).is_err() {
      return;
    }
  }
}

fn thumbnail_or_previous(decoded: Option<Vec<u8>>, previous: &Option<Vec<u8>>) -> Option<Vec<u8>> {
  decoded.or_else(|| previous.clone())
}

#[cfg(test)]
mod tests {
  use super::thumbnail_or_previous;

  #[test]
  fn a_short_sidecar_reuses_its_last_real_thumbnail() {
    let previous = Some(vec![1, 2, 3]);
    assert_eq!(thumbnail_or_previous(None, &previous), previous);
    assert_eq!(
      thumbnail_or_previous(Some(vec![4]), &previous),
      Some(vec![4])
    );
  }
}
