// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The C entry points the native ribbon draws through. Its layer, display
//! link and appearance stay native; the shader and the frame are wgpu's.
//! AppKit calls every one of these on the main thread.

use std::ffi::c_void;
use std::ptr::{null_mut, NonNull};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::super::super::audio_ribbon::{AudioRibbonRenderer, RibbonLook};
use crate::editor::recording_preview_player::audio_visualizer::AudioRibbonEnvelopes;
use crate::gpu::surface::Surface;
use crate::gpu::Gpu;

pub struct NativeAudioRibbon {
  gpu: &'static Gpu,
  renderer: AudioRibbonRenderer,
  /// Set from a frame's submission until the GPU finishes it. wgpu turns off
  /// the layer's drawable timeout, so acquiring while every drawable is
  /// queued would stall the main thread instead of skipping a frame.
  in_flight: Arc<AtomicBool>,
}

/// A renderer presenting into `layer`, a `CAMetalLayer` the caller owns and
/// places. Null when no graphics device or surface is available.
///
/// # Safety
/// `layer` must be a live `CAMetalLayer`.
#[no_mangle]
pub unsafe extern "C" fn screenwide_audio_ribbon_renderer_create(
  layer: *mut c_void,
) -> *mut NativeAudioRibbon {
  let Some(layer) = NonNull::new(layer) else {
    return null_mut();
  };
  let created = crate::gpu::shared().and_then(|gpu| {
    let surface = unsafe { Surface::on_metal_layer(gpu, layer) }?;
    Ok(NativeAudioRibbon {
      gpu,
      renderer: AudioRibbonRenderer::new(gpu, surface),
      in_flight: Arc::default(),
    })
  });
  match created {
    Ok(ribbon) => Box::into_raw(Box::new(ribbon)),
    Err(error) => {
      eprintln!("The audio ribbon could not open its surface: {error}");
      null_mut()
    }
  }
}

/// # Safety
/// `ribbon` must come from [`screenwide_audio_ribbon_renderer_create`] and
/// not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn screenwide_audio_ribbon_renderer_destroy(ribbon: *mut NativeAudioRibbon) {
  if !ribbon.is_null() {
    drop(unsafe { Box::from_raw(ribbon) });
  }
}

/// `tracks` rows of `points` samples, row-major, and one gain per track.
/// Zero tracks or points clears the bars.
///
/// # Safety
/// `ribbon` must be live, `samples` must hold `tracks * points` values and
/// `gains` `tracks` values, unless either count is zero.
#[no_mangle]
pub unsafe extern "C" fn screenwide_audio_ribbon_renderer_set_envelopes(
  ribbon: *mut NativeAudioRibbon,
  samples: *const f32,
  tracks: u32,
  points: u32,
  gains: *const f32,
) {
  let Some(ribbon) = (unsafe { ribbon.as_mut() }) else {
    return;
  };
  let envelopes = if samples.is_null() || gains.is_null() || tracks == 0 || points == 0 {
    AudioRibbonEnvelopes::default()
  } else {
    let count = tracks as usize * points as usize;
    AudioRibbonEnvelopes {
      gains: unsafe { std::slice::from_raw_parts(gains, tracks as usize) }.to_vec(),
      points,
      samples: unsafe { std::slice::from_raw_parts(samples, count) }.to_vec(),
      tracks,
    }
  };
  ribbon.renderer.set_envelopes(&envelopes);
}

/// Draws one frame into a `width` by `height` drawable, unless the last one
/// is still on the GPU or nothing changed. `colors` is the recording's RGBA
/// then the outside's. Returns 1 when a frame was submitted.
///
/// # Safety
/// `ribbon` must be live and `colors` must hold eight values.
#[no_mangle]
pub unsafe extern "C" fn screenwide_audio_ribbon_renderer_draw(
  ribbon: *mut NativeAudioRibbon,
  width: u32,
  height: u32,
  scale: f32,
  playhead: f32,
  colors: *const f32,
) -> i32 {
  let Some(ribbon) = (unsafe { ribbon.as_mut() }) else {
    return 0;
  };
  if colors.is_null() {
    return 0;
  }
  // Completion callbacks only run while the device is maintained.
  let _ = ribbon.gpu.device.poll(wgpu::PollType::Poll);
  if ribbon.in_flight.load(Ordering::Acquire) {
    return 0;
  }
  let colors = unsafe { std::slice::from_raw_parts(colors, 8) };
  ribbon.renderer.set_viewport((width, height));
  let look = RibbonLook {
    scale,
    playhead,
    color: [colors[0], colors[1], colors[2], colors[3]],
    flat: [colors[4], colors[5], colors[6], colors[7]],
  };
  match ribbon.renderer.draw(look) {
    Ok(true) => {
      ribbon.in_flight.store(true, Ordering::Release);
      let in_flight = Arc::clone(&ribbon.in_flight);
      ribbon
        .gpu
        .queue
        .on_submitted_work_done(move || in_flight.store(false, Ordering::Release));
      1
    }
    Ok(false) => 0,
    Err(error) => {
      eprintln!("The audio ribbon could not draw: {error}");
      0
    }
  }
}
