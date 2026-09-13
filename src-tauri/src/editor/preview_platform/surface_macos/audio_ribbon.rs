// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The audio-only preview's bars, handed their envelopes and their playhead.
//!
//! The shape and the colour are native: this is only the boundary that hands
//! over what they are drawn from.

use super::ffi::{
  screenwide_preview_surface_set_audio_ribbon, screenwide_preview_surface_set_audio_ribbon_clock,
  screenwide_preview_surface_set_audio_ribbon_playhead,
};
use super::RecordingPreviewSurface;
use crate::editor::recording_preview_player::audio_visualizer::AudioRibbonEnvelopes;

type ClockReader = Box<dyn Fn(f64) -> f64 + Send + Sync>;

unsafe extern "C" fn read_clock(context: *mut std::ffi::c_void, ahead: f64) -> f64 {
  (unsafe { &*context.cast::<ClockReader>() })(ahead)
}

unsafe extern "C" fn release_clock(context: *mut std::ffi::c_void) {
  drop(unsafe { Box::from_raw(context.cast::<ClockReader>()) });
}

impl RecordingPreviewSurface {
  pub(crate) fn set_audio_ribbon_clock(&self, read: impl Fn(f64) -> f64 + Send + Sync + 'static) {
    let clock: Box<ClockReader> = Box::new(Box::new(read));
    // Native ownership includes releasing the box when a seek replaces it or
    // the surface closes; queued main-thread reads cannot outlive the context.
    unsafe {
      screenwide_preview_surface_set_audio_ribbon_clock(
        self.handle,
        Box::into_raw(clock).cast(),
        read_clock,
        release_clock,
      );
    }
  }

  pub(crate) fn set_audio_ribbon(&self, envelopes: &AudioRibbonEnvelopes) {
    unsafe {
      screenwide_preview_surface_set_audio_ribbon(
        self.handle,
        envelopes.samples.as_ptr(),
        envelopes.tracks,
        envelopes.points,
        envelopes.gains.as_ptr(),
      );
    }
  }

  /// Presents the source position selected by the native audio playback clock.
  pub(crate) fn set_audio_ribbon_playhead(&self, ratio: f64) {
    unsafe {
      screenwide_preview_surface_set_audio_ribbon_playhead(self.handle, ratio);
    }
  }
}
