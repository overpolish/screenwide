// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The legs and paths pinned annotations have been worked out to, by
//! recording, shared by the preview and the export.
//!
//! A leg is kept by where it starts and how far it ran. A leg is followed a
//! frame at a time, so a shorter one from the same start is the start of a
//! longer one: adding a correction inside a stretch, or trimming a clip
//! shorter, takes its legs from what was already followed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

use crate::editor::annotations::pin::leg::{Leg, LegSample};
use crate::editor::annotations::pin::PinnedPath;

/// How many legs and paths are kept before they are all let go.
const KEPT_LEGS: usize = 512;
const KEPT_PATHS: usize = 128;

pub(super) type Samples = Arc<Vec<LegSample>>;

#[derive(Default)]
struct Cache {
  /// Legs by recording and where they start, each with how far it ran.
  legs: HashMap<(PathBuf, u64), Vec<(u64, Samples)>>,
  leg_count: usize,
  paths: HashMap<(PathBuf, u64), Arc<PinnedPath>>,
}

static CACHE: LazyLock<Mutex<Cache>> = LazyLock::new(Default::default);

pub(super) fn cached_path(recording: &Path, key: u64) -> Option<Arc<PinnedPath>> {
  CACHE
    .lock()
    .ok()?
    .paths
    .get(&(recording.to_path_buf(), key))
    .cloned()
}

/// `leg`'s frames, from a leg already followed as far or further from the
/// same start.
pub(super) fn cached_leg(recording: &Path, leg: &Leg) -> Option<Samples> {
  let cache = CACHE.lock().ok()?;
  let followed = cache.legs.get(&(recording.to_path_buf(), leg.origin()))?;
  let (until_ms, samples) = followed
    .iter()
    .find(|(until_ms, _)| leg.within(*until_ms))?;
  Some(if *until_ms == leg.until_ms {
    Arc::clone(samples)
  } else {
    Arc::new(leg.cut_from(samples))
  })
}

pub(super) fn store_leg(recording: &Path, leg: &Leg, samples: &Samples) {
  let Ok(mut cache) = CACHE.lock() else {
    return;
  };
  if cache.leg_count >= KEPT_LEGS {
    cache.legs.clear();
    cache.leg_count = 0;
  }
  cache.leg_count += 1;
  cache
    .legs
    .entry((recording.to_path_buf(), leg.origin()))
    .or_default()
    .push((leg.until_ms, Arc::clone(samples)));
}

/// Keeps `path` by its key.
pub(super) fn store_path(recording: &Path, path: &Arc<PinnedPath>) {
  let Ok(mut cache) = CACHE.lock() else {
    return;
  };
  if cache.paths.len() >= KEPT_PATHS {
    cache.paths.clear();
  }
  cache
    .paths
    .insert((recording.to_path_buf(), path.key), Arc::clone(path));
}
