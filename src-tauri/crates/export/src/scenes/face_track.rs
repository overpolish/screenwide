// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the face is over a camera recording, as the helper found it, kept
//! beside the camera file so whatever draws the camera can find it: the
//! preview, every export, and the app that has it found.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use screenwide_scenes::{FacePath, FaceSighting};
use screenwide_transcriber::FaceTrack;

/// Where the face track of the camera at `camera` is kept.
pub fn track_path(camera: &Path) -> PathBuf {
  let stem = camera
    .file_stem()
    .map_or_else(Default::default, |stem| stem.to_string_lossy());
  camera.with_file_name(format!("{stem}-faces.json"))
}

/// The smoothed path of the face over the camera at `camera`, where it has
/// been found and its track is one this version reads.
pub fn load(camera: &Path) -> Option<Arc<FacePath>> {
  let bytes = std::fs::read(track_path(camera)).ok()?;
  let track = serde_json::from_slice::<FaceTrack>(&bytes)
    .ok()
    .filter(|track| track.format == FaceTrack::FORMAT)?;
  let sightings = track
    .frames
    .iter()
    .filter_map(|&(ms, face)| {
      face.map(|[x, y, size]| FaceSighting {
        ms,
        x: f64::from(x),
        y: f64::from(y),
        size: f64::from(size),
      })
    })
    .collect::<Vec<_>>();
  FacePath::new(&sightings, track.aspect, track.duration_ms).map(Arc::new)
}
