// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  /// Redraws the paused stills from their cached full-resolution sources and
  /// compositions with the given output settings - no decoder round trip, so
  /// a canvas resize follows the pointer instead of trailing a Media
  /// Foundation reopen. Returns `Ok(false)` when a needed frame is not
  /// cached yet and the decoder has to supply it. `bake_camera` means "bake
  /// with an available camera track" - the caller folds the track's
  /// existence in, exactly like the present path does.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn redraw_still(
    &self,
    bake_camera: bool,
    primary: &ScreenshotOutputSettings,
    camera_settings: &ScreenshotOutputSettings,
    overlay: crate::editor::CameraOverlaySettings,
    drop_shadow: bool,
    camera_on_top: bool,
  ) -> Result<bool, String> {
    let batch = self.present_batch();
    {
      let Ok(mut state) = self.inner.state.lock() else {
        return Ok(false);
      };
      let camera_source = state.camera_source.clone();
      let Some(pane) = state.panes.first_mut().and_then(Option::as_mut) else {
        return Ok(false);
      };
      let (Some(composition), Some(_)) = (pane.last_composition, pane.source.as_ref()) else {
        return Ok(false);
      };
      // A redraw moves the picture, not the annotations on it. The caller's
      // settings come straight from the composition, whose annotations are only
      // resolved from the timeline clips per decoded frame, so they arrive
      // empty here; the annotations this frame actually carries - resolved and
      // already on the decoded grid - are the ones the last present cached.
      let primary = with_cached_annotations(pane, primary);
      let primary = &primary;
      if bake_camera {
        // The camera texture is only cached while baked presents run; right
        // after a bake toggle it is absent (or stale) and the decoder must
        // deliver it. Never draw a baked still without its camera.
        let Some(camera) = camera_source.as_ref() else {
          return Ok(false);
        };
        {
          let geometry = crate::editor::media_preview::bake_geometry(BakedVideoExportOptions {
            camera_drop_shadow: drop_shadow,
            camera_height: camera.size.1,
            camera_width: camera.size.0,
            overlay,
            screen_height: primary.height,
            screen_width: primary.width,
            video: VideoExportOptions {
              compression: 0,
              resolution_scale_percent: 100,
              source_scale_percent: 100,
            },
          })?;
          self.present_cached_source_with_camera(
            pane,
            primary,
            composition,
            Some((camera, geometry, drop_shadow, camera_on_top)),
          )?;
        }
      } else {
        self.present_cached_source(pane, primary, composition)?;
      }
      if !bake_camera {
        if let Some(pane) = state.panes.get_mut(1).and_then(Option::as_mut) {
          if let (Some(composition), Some(_)) = (pane.last_composition, pane.source.as_ref()) {
            let camera_settings = with_cached_annotations(pane, camera_settings);
            self.present_cached_source(pane, &camera_settings, composition)?;
          }
        }
      }
    }
    drop(batch);
    Ok(true)
  }

  #[allow(clippy::too_many_arguments)]
  // Retained-workspace entry point; not wired on Windows yet.
  #[allow(dead_code)]
  pub(crate) fn present_composed(
    &self,
    index: u32,
    source_token: u64,
    source: &CapturedImage,
    settings: &ScreenshotOutputSettings,
    seconds: f64,
    _cursor: Option<&CapturedImage>,
    _camera: Option<&CapturedImage>,
    _overlay: Option<&StillOverlay>,
    _clip_cursor_at_video_edge: bool,
  ) -> Result<bool, String> {
    let Ok(mut state) = self.inner.state.lock() else {
      return Ok(false);
    };
    let Some(pane) = state.panes.get_mut(index as usize).and_then(Option::as_mut) else {
      return Ok(false);
    };
    let source_size = (source.width, source.height);
    if pane.source_token != Some(source_token)
      || pane
        .source
        .as_ref()
        .is_none_or(|texture| texture.size != source_size)
    {
      let texture = self
        .inner
        .gpu
        .compositor
        .screenshot_source(&self.inner.gpu.device, source)?;
      pane.source = Some(texture);
      pane.source_token = Some(source_token);
    }
    let staged = self.present_cached_source(
      pane,
      settings,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only: false,
        seconds,
      },
    )?;
    redraw_stale_selection(&self.inner, &mut state);
    Ok(staged)
  }

  pub(crate) fn present_screenshot_layer(
    &self,
    index: u32,
    source_token: u64,
    source: &CapturedImage,
    settings: &ScreenshotOutputSettings,
    foreground_only: bool,
  ) -> Result<bool, String> {
    let Ok(mut state) = self.inner.state.lock() else {
      return Ok(false);
    };
    // The halo belongs to one layer; read it before the pane is borrowed.
    let halo = state
      .annotation
      .hover
      .filter(|(layer, _, _)| *layer == source_token)
      .map(|(_, index, width)| (index, width));
    let Some(pane) = state.panes.get_mut(index as usize).and_then(Option::as_mut) else {
      return Ok(false);
    };
    pane.annotation_halo = halo;
    let source_size = (source.width, source.height);
    if pane.source_token != Some(source_token)
      || pane
        .source
        .as_ref()
        .is_none_or(|texture| texture.size != source_size)
    {
      let texture = self
        .inner
        .gpu
        .compositor
        .screenshot_source(&self.inner.gpu.device, source)?;
      pane.source = Some(texture);
      pane.source_token = Some(source_token);
    }
    let staged = self.present_cached_source(
      pane,
      settings,
      ComposedFrame {
        cursor: None,
        keyboard: None,
        foreground_only,
        seconds: 0.0,
      },
    )?;
    redraw_stale_selection(&self.inner, &mut state);
    Ok(staged)
  }
}

/// `settings` with the annotations the pane last drew in place of its own,
/// which for a recording arrive unresolved and empty. The cached annotations
/// are already resolved for this frame and on its decoded grid, so they are
/// drawn as is.
fn with_cached_annotations(
  pane: &Pane,
  settings: &ScreenshotOutputSettings,
) -> ScreenshotOutputSettings {
  let mut settings = settings.clone();
  if let Some(cached) = pane.settings.as_ref() {
    settings.annotations = cached.annotations.clone();
  }
  settings
}
