// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS timeline thumbnails and full-resolution source frames.
//!
//! Each track's thumbnails are one `AVAssetImageGenerator` request for all
//! their times, made on a thread of the track's own, so AVFoundation decodes
//! the screen and the camera side by side at the thumbnail size without ever
//! holding a full-resolution frame.

use std::sync::mpsc;

use cidre::{av, blocks, cg, cm, ns};
use screenwide_export::camera_effects::cpu::StillEffects;
use screenwide_imaging::CapturedImage;
use tauri::ipc::{Channel, InvokeResponseBody};

use super::image::{captured_image, frame_position, time};
use crate::editor::recording_preview_player::{
  timeline_thumbnails::{
    cell_centre_ms, cell_tolerance_ms, encoded_png, payload, thumbnail_tracks, ThumbnailTrack,
    THUMBNAIL_HEIGHT,
  },
  PlayerSources,
};

fn image_generator(
  path: &std::path::Path,
) -> Result<cidre::arc::R<cidre::av::AssetImageGenerator>, String> {
  use cidre::{av, cm, ns};

  let path_text = path
    .to_str()
    .ok_or_else(|| "The recording path is not valid UTF-8".to_owned())?;
  let url = ns::Url::with_fs_path_str(path_text, false);
  let asset = av::UrlAsset::with_url(&url, None)
    .ok_or_else(|| format!("AVFoundation could not open {}", path.display()))?;
  let mut generator = av::AssetImageGenerator::with_asset(&asset);
  generator.set_applies_preferred_track_transform(true);
  generator.set_requested_time_tolerance_before(cm::Time::new(100, 1_000));
  generator.set_requested_time_tolerance_after(cm::Time::new(1, 60));
  Ok(generator)
}

/// A request's outcome: the time asked for, and the frame with the time it
/// is really from, or nothing where AVFoundation could not make it.
type Generated = (u64, Option<(CapturedImage, u64)>);

pub(super) fn generate(sources: PlayerSources, count: u32, channel: Channel) {
  generate_tracks(
    thumbnail_tracks(&sources),
    sources.duration_ms,
    count,
    &channel,
  );
}

/// Each track on a thread of its own, so neither waits for the other's
/// frames, effects or encoding.
fn generate_tracks(
  tracks: Vec<ThumbnailTrack<'_>>,
  recording_ms: u64,
  count: u32,
  channel: &Channel,
) {
  std::thread::scope(|scope| {
    for track in tracks {
      scope.spawn(move || generate_track(track, recording_ms, count, channel));
    }
  });
}

fn generate_track(track: ThumbnailTrack<'_>, recording_ms: u64, count: u32, channel: &Channel) {
  let Ok(mut generator) = image_generator(track.path) else {
    return;
  };
  generator.set_max_size(cg::Size {
    width: f64::from(track.width),
    height: f64::from(THUMBNAIL_HEIGHT),
  });
  let tolerance = time(cell_tolerance_ms(recording_ms, count));
  generator.set_requested_time_tolerance_before(tolerance);
  generator.set_requested_time_tolerance_after(tolerance);
  let cells = cell_times(recording_ms, track.duration_ms, count);
  let times: Vec<_> = cells
    .iter()
    .map(|(at, _)| ns::Value::with_cm_time(&time(*at)))
    .collect();
  let (sender, receiver) = mpsc::channel::<Generated>();
  let mut handler = blocks::EscBlock::new5(
    move |requested: cm::Time,
          image: Option<&cg::Image>,
          actual: cm::Time,
          result: av::AssetImageGeneratorResult,
          _error: Option<&ns::Error>| {
      // Converted here, on AVFoundation's queue, so the next frame decodes
      // while the track's thread draws the effects on this one.
      let frame = image
        .filter(|_| result == av::AssetImageGeneratorResult::Succeeded)
        .and_then(|image| captured_image(image).ok())
        .map(|image| (image, milliseconds(actual)));
      let _ = sender.send((milliseconds(requested), frame));
    },
  );
  generator.cg_images_for_times_ch(&ns::Array::from_slice_retained(&times), &mut handler);

  let mut effects = StillEffects::new(track.effects, THUMBNAIL_HEIGHT * 4);
  let see_through = effects.as_ref().is_some_and(StillEffects::is_transparent);
  let mut sending = true;
  // Every time is answered, made, failed or cancelled, before the handler
  // is let go.
  for (requested_ms, frame) in receiver.iter().take(cells.len()) {
    let (Some((mut image, actual_ms)), true) = (frame, sending) else {
      continue;
    };
    let Ok(cell) = cells.binary_search_by_key(&requested_ms, |(at, _)| *at) else {
      continue;
    };
    if let Some(effects) = effects.as_mut() {
      // The matte is read for the moment the frame is from, which the
      // tolerance lets differ from the one asked for.
      effects.apply(&mut image, actual_ms);
    }
    let Ok(bytes) = encoded(&image, see_through) else {
      continue;
    };
    for &index in &cells[cell].1 {
      let body = InvokeResponseBody::Raw(payload(track.id, index, count, &bytes));
      if channel.send(body).is_err() {
        generator.cancel_all_cg_image_gen();
        sending = false;
        break;
      }
    }
  }
}

/// The time each cell's thumbnail is asked for in a video `duration_ms`
/// long, with the cells that show it: a camera that stops a little before
/// the screen gives its last frame, decoded once, to every cell past its end.
fn cell_times(recording_ms: u64, duration_ms: u64, count: u32) -> Vec<(u64, Vec<u32>)> {
  let mut cells: Vec<(u64, Vec<u32>)> = Vec::new();
  for index in 0..count {
    let at = frame_position(cell_centre_ms(recording_ms, index, count), duration_ms);
    match cells.last_mut() {
      Some((last, indices)) if *last == at => indices.push(index),
      _ => cells.push((at, vec![index])),
    }
  }
  cells
}

fn milliseconds(time: cm::Time) -> u64 {
  u64::try_from(time.value.saturating_mul(1_000) / i64::from(time.scale.max(1))).unwrap_or_default()
}

/// A thumbnail as the timeline takes it: a PNG where it is see-through, and
/// otherwise a JPEG, which is smaller.
fn encoded(
  image: &screenwide_imaging::CapturedImage,
  see_through: bool,
) -> Result<Vec<u8>, String> {
  if see_through {
    encoded_png(image)
  } else {
    super::composition::encoded_jpeg(image)
  }
}

pub(super) fn source_frame_jpeg(
  path: &std::path::Path,
  position_ms: u64,
  duration_ms: u64,
) -> Result<Vec<u8>, String> {
  let generator = image_generator(path)?;
  let position = frame_position(position_ms.min(duration_ms), duration_ms);
  let (image, _) =
    tauri::async_runtime::block_on(generator.cg_image_for_time(super::image::time(position)))
      .map_err(|error| error.to_string())?;
  let captured = super::image::captured_image(&image)?;
  super::composition::encoded_jpeg(&captured)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cells_past_a_short_cameras_end_share_its_last_frame() {
    let cells = cell_times(10_000, 8_000, 10);
    assert_eq!(cells.len(), 9);
    assert_eq!(cells[7], (7_500, vec![7]));
    assert_eq!(cells[8], (7_999, vec![8, 9]));
  }

  #[test]
  #[ignore = "uses the video path in SCREENWIDE_THUMBNAIL_BENCHMARK"]
  fn benchmarks_thirty_minute_native_thumbnail_extraction() {
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    let path = std::env::var("SCREENWIDE_THUMBNAIL_BENCHMARK")
      .expect("set SCREENWIDE_THUMBNAIL_BENCHMARK to a 30-minute video");
    let made = Arc::new(AtomicUsize::new(0));
    let channel = Channel::new({
      let made = made.clone();
      move |_| {
        made.fetch_add(1, Ordering::Relaxed);
        Ok(())
      }
    });
    let track = |id| ThumbnailTrack {
      id,
      path: Path::new(&path),
      duration_ms: 1_800_000,
      width: 114,
      effects: None,
    };
    let started = Instant::now();
    generate_tracks(vec![track(0), track(1)], 1_800_000, 24, &channel);
    let elapsed = started.elapsed();
    let made = made.load(Ordering::Relaxed);
    eprintln!("made {made} thumbnails from two 30-minute tracks in {elapsed:?}");
    assert_eq!(made, 48);
  }
}
