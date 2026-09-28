// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pixels a highlight's selection reads, held for the length of a gesture.
//!
//! A screenshot's picture is at hand whenever a gesture begins. A recording's
//! frame has to be decoded first, which is far too slow to wait for between
//! two pointer samples, so the press starts the decode and the samples that
//! follow read it once it has landed. Until then the highlight is drawn out as
//! a plain band, which the first sample after the frame arrives replaces.

use super::detect::HighlightPixels;
use crate::screenshots::CapturedImage;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// A picture a selection reads, and how it lines up with the source pixels
/// the annotation is placed in: a decoder may hand back a frame at another
/// size than the recording's own.
#[derive(Clone)]
pub(crate) struct HighlightPicture {
  image: Arc<CapturedImage>,
  /// Picture pixels per source pixel, across and down.
  scale: (f64, f64),
}

impl std::fmt::Debug for HighlightPicture {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("HighlightPicture")
      .field("width", &self.image.width)
      .field("height", &self.image.height)
      .field("scale", &self.scale)
      .finish()
  }
}

impl HighlightPicture {
  /// `image` read over a source `source` pixels in size.
  pub(crate) fn new(image: Arc<CapturedImage>, source: (u32, u32)) -> Option<Self> {
    let scale = (
      f64::from(image.width) / f64::from(source.0.max(1)),
      f64::from(image.height) / f64::from(source.1.max(1)),
    );
    Self::scaled(image, scale)
  }

  /// `image`, with `scale` of its pixels to each source pixel across and
  /// down.
  pub(crate) fn scaled(image: Arc<CapturedImage>, scale: (f64, f64)) -> Option<Self> {
    let expected = (image.width as usize)
      .checked_mul(image.height as usize)?
      .checked_mul(4)?;
    let usable = scale.0.is_finite() && scale.1.is_finite() && scale.0 > 0.0 && scale.1 > 0.0;
    if image.width == 0 || image.height == 0 || image.rgba.len() < expected || !usable {
      return None;
    }
    Some(Self { scale, image })
  }

  pub(crate) fn pixels(&self) -> HighlightPixels<'_> {
    HighlightPixels {
      rgba: &self.image.rgba,
      width: self.image.width,
      height: self.image.height,
    }
  }

  /// Picture pixels per source pixel, across and down.
  pub(crate) fn scale(&self) -> (f64, f64) {
    self.scale
  }
}

/// What a frame's picture is keyed by: whoever owns the cache decides, as
/// long as one frame is one key.
pub(crate) type PictureKey = (u64, u32, u64);

#[derive(Default)]
struct CacheState {
  key: Option<PictureKey>,
  picture: Option<Arc<HighlightPicture>>,
  decoding: Option<PictureKey>,
}

/// The latest frame's picture. A new key replaces the one before it: a
/// selection only ever reads the frame the pointer is over, and a 5K frame is
/// too large to keep a history of.
///
/// The mutex is a leaf, taken only for a clone or a store, so the pointer's
/// thread can read it while holding its manager.
#[derive(Default)]
pub(crate) struct PictureCache {
  state: Mutex<CacheState>,
  landed: Condvar,
}

impl PictureCache {
  /// The picture for `key`, if it has landed.
  pub(crate) fn picture(&self, key: PictureKey) -> Option<Arc<HighlightPicture>> {
    let state = self.state.lock().ok()?;
    (state.key == Some(key))
      .then(|| state.picture.clone())
      .flatten()
  }

  /// The picture for `key`, waiting up to `timeout` for a decode already on
  /// its way. A gesture that ends before its frame has landed waits this long
  /// rather than committing the plain band it has drawn so far.
  pub(crate) fn wait(&self, key: PictureKey, timeout: Duration) -> Option<Arc<HighlightPicture>> {
    let state = self.state.lock().ok()?;
    let (state, _) = self
      .landed
      .wait_timeout_while(state, timeout, |state| {
        state.decoding == Some(key) && state.key != Some(key)
      })
      .ok()?;
    (state.key == Some(key))
      .then(|| state.picture.clone())
      .flatten()
  }

  /// Whether this caller is the one that decodes `key`: there is no picture
  /// for it and none already on its way.
  fn claim(&self, key: PictureKey) -> bool {
    let Ok(mut state) = self.state.lock() else {
      return false;
    };
    if state.decoding == Some(key) || state.key == Some(key) {
      return false;
    }
    state.decoding = Some(key);
    true
  }

  fn store(&self, key: PictureKey, picture: Option<HighlightPicture>) {
    if let Ok(mut state) = self.state.lock() {
      // A decode that finished after a newer one was asked for is dropped.
      if state.decoding == Some(key) {
        state.key = Some(key);
        state.picture = picture.map(Arc::new);
        state.decoding = None;
      }
    }
    self.landed.notify_all();
  }
}

/// Decodes `key`'s picture off the pointer's thread unless it is already
/// there or on its way. A frame that cannot be decoded is remembered as having
/// no picture, so it is not decoded again on the next sample.
pub(crate) fn request_picture(
  cache: &Arc<PictureCache>,
  key: PictureKey,
  decode: impl FnOnce() -> Option<HighlightPicture> + Send + 'static,
) {
  if !cache.claim(key) {
    return;
  }
  let cache = Arc::clone(cache);
  tauri::async_runtime::spawn_blocking(move || cache.store(key, decode()));
}
