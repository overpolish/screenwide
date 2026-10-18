// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::worker::PlaybackMode;
use super::{PreviewCompositionSettings, PreviewPlayerManager};

impl PreviewPlayerManager {
  pub(super) fn selection_composition(&self) -> Option<PreviewCompositionSettings> {
    self
      .sources
      .as_ref()?
      .composition_settings
      .as_ref()?
      .read()
      .ok()
      .map(|settings| settings.clone())
  }

  /// Recomposes the paused stills from the frames the workspace holds against
  /// whatever composition is current, as the scenes arrange it, without
  /// asking the decoder for the same source pixels. `Ok(false)` means a frame
  /// is not resident yet and the decoder has to supply it.
  pub(super) fn redraw_still_now(&self) -> Result<bool, String> {
    let sources = self
      .sources
      .as_ref()
      .ok_or_else(|| "The recording preview player is not open".to_owned())?;
    let surface = sources
      .preview_surface
      .as_ref()
      .ok_or_else(|| "The recording preview surface is unavailable".to_owned())?;
    let composition = sources
      .composition_settings
      .as_ref()
      .ok_or_else(|| "The recording preview composition is unavailable".to_owned())?
      .read()
      .map_err(|_| "The recording preview composition is unavailable".to_owned())?
      .clone();
    let composition = self.arranged_composition(&composition);
    let baked = composition.bake_camera && sources.camera_path.is_some();
    let mut panes = vec![(0, &composition.recording_output.primary)];
    if !baked && sources.camera_path.is_some() {
      panes.push((1, &composition.recording_output.camera));
    }
    let recomposed = surface.recompose_recording_workspace(
      &panes,
      baked.then_some((
        composition.camera_overlay,
        composition.recording_output.camera.drop_shadow,
        screenwide_export::recording_model::CAMERA_IN_FRONT,
      )),
    )?;
    Ok(recomposed && surface.redraw_recording_workspace())
  }

  /// Redraws the paused still a selection gesture changed from the frames
  /// the workspace holds, or has the decoder supply them where it holds none.
  pub(super) fn refresh_selection_preview(&mut self, _layer_id: u32) -> Result<(), String> {
    if self.redraw_still_now()? {
      return Ok(());
    }
    self.restart(PlaybackMode::InteractiveStill)
  }

  /// Redraws the paused still a layout changed, from the cached sources. A
  /// bake toggle moves the camera in or out of the picture, which the
  /// annotations carried for it follow.
  pub(super) fn redraw_layout_still(&self, bake_changed: bool) -> bool {
    if bake_changed {
      self.recompose_paused_still()
    } else {
      self.redraw_still_now().unwrap_or(false)
    }
  }

  /// Recomposes the paused still from the cached sources after the camera's
  /// place in it changed: a scene moved it, or One video drew it in or took
  /// it out. The screen carries the camera's annotations where the camera
  /// was, so they are resolved again for where it is now, in the same
  /// present. `false` means a source is not cached yet and the decoder has to
  /// supply the frame.
  pub(super) fn recompose_paused_still(&self) -> bool {
    let Some(surface) = self
      .sources
      .as_ref()
      .and_then(|sources| sources.preview_surface.as_ref())
    else {
      return false;
    };
    let _batch = surface.present_batch();
    self.redraw_still_now().unwrap_or(false) && self.redraw_annotation_frame(0, self.position_ms)
  }
}
