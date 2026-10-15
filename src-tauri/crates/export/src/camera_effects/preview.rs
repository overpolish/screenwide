// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects as a preview holds them: the effects as they stand,
//! the matte read beside the camera frames it shows, and the pass that
//! draws them. A preview keeps its frames as decoded, so changing the
//! effects is a redraw.

use std::sync::Arc;

use super::gpu::EffectsPass;
use super::live::LiveCameraEffects;
use super::matte_frames::MatteFrames;
use screenwide_annotations::pin::LumaFrame;
use screenwide_compositor::compositor::{Compositor, SourceTexture};
use screenwide_gpu::Gpu;

/// The longest side the preview reads the matte at: the edge it draws is
/// soft, and a pane is rarely drawn larger.
const MATTE_SIDE: u32 = 960;

#[derive(Default)]
pub struct PreviewEffects {
  live: Option<Arc<LiveCameraEffects>>,
  matte: Option<MatteFrames>,
  pass: Option<EffectsPass>,
}

impl PreviewEffects {
  /// Draws with `live` from now on, opening its matte where it is not the
  /// one already open.
  pub fn set(
    &mut self,
    live: Option<Arc<LiveCameraEffects>>,
    gpu: &'static Gpu,
  ) -> Result<(), String> {
    let files = live.as_deref().map_or(&[][..], |live| &live.matte[..]);
    let same_matte = self
      .matte
      .as_ref()
      .is_some_and(|open| open.files() == files);
    if !same_matte {
      self.matte = (!files.is_empty()).then(|| MatteFrames::open(files, MATTE_SIDE));
    }
    if live.is_some() && self.pass.is_none() {
      self.pass = Some(EffectsPass::new(gpu));
    }
    self.live = live;
    Ok(())
  }

  /// The matte of the camera frame showing at `seconds`, where the effects
  /// read one.
  pub fn matte_at(&mut self, seconds: f64) -> Option<Arc<LumaFrame>> {
    let ms = ms_of(seconds);
    match self.matte.as_mut()?.at(ms) {
      Ok(frame) => frame,
      Err(error) => {
        eprintln!("Could not read the camera's matte: {error}");
        None
      }
    }
  }

  /// `camera`, the frame `seconds` into the recording, as the effects draw
  /// it, into `slot`, or nothing where there are no effects and the frame is
  /// drawn as decoded. `compositor` is the one drawing the frame, which
  /// blurs the room.
  pub fn drawn(
    &self,
    compositor: &Compositor,
    (camera, seconds): (&SourceTexture, f64),
    matte: Option<&LumaFrame>,
    slot: usize,
  ) -> Result<Option<SourceTexture>, String> {
    let (Some(live), Some(pass)) = (self.live.as_deref(), self.pass.as_ref()) else {
      return Ok(None);
    };
    pass
      .apply(compositor, (camera, ms_of(seconds)), matte, live, slot)
      .map(Some)
  }

  /// `camera` drawn with `still`'s effects into `slot`, apart from the
  /// effects the preview draws with: a still made of another moment.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  pub fn drawn_once(
    &mut self,
    compositor: &Compositor,
    still: &StillCameraEffects<'_>,
    camera: &SourceTexture,
    slot: usize,
  ) -> Result<SourceTexture, String> {
    self
      .pass
      .get_or_insert_with(|| EffectsPass::new(compositor.gpu()))
      .apply(
        compositor,
        (camera, still.source_ms),
        still.matte,
        still.live,
        slot,
      )
  }
}

/// `seconds` as whole milliseconds, as the matte is read at.
fn ms_of(seconds: f64) -> u64 {
  (seconds * 1_000.0).round().max(0.0) as u64
}

/// The camera's effects for one still drawn on the GPU, its matte read for
/// the moment it shows, that moment, and whether they are drawn on the
/// screen, which is the camera, or on the camera baked into it.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct StillCameraEffects<'a> {
  pub live: &'a LiveCameraEffects,
  pub matte: Option<&'a LumaFrame>,
  /// The moment the still shows, in milliseconds into the recording.
  pub source_ms: u64,
  pub on_screen: bool,
}
