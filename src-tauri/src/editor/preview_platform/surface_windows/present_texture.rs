// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  pub(crate) fn present_composed_texture(
    &self,
    index: u32,
    texture: &ID3D11Texture2D,
    subresource: u32,
    size: (u32, u32),
    settings: &ScreenshotOutputSettings,
    composition: ComposedFrame,
  ) -> Result<bool, String> {
    let Ok(mut state) = self.inner.state.lock() else {
      return Ok(false);
    };
    // A recording's annotations belong to the pane they were drawn over, so the
    // halo is keyed by pane index here, where a screenshot keys it by layer.
    let halo = recording_halo(&state, index);
    let Some(pane) = state.panes.get_mut(index as usize).and_then(Option::as_mut) else {
      return Ok(false);
    };
    pane.annotation_halo = halo;
    if pane
      .source
      .as_ref()
      .is_none_or(|source| source.size != size)
    {
      pane.source = Some(
        self
          .inner
          .gpu
          .compositor
          .source(&self.inner.gpu.device, size)?,
      );
      pane.source_token = None;
    }
    let source = pane
      .source
      .as_ref()
      .ok_or_else(|| "The preview source texture is unavailable".to_owned())?;
    compositor::Compositor::copy_source(&self.inner.gpu.context, source, texture, subresource)?;
    pane.source_token = None;
    let staged = self.present_cached_source(pane, settings, composition)?;
    redraw_stale_selection(&self.inner, &mut state);
    Ok(staged)
  }

  #[allow(clippy::too_many_arguments)]
  pub(crate) fn present_baked_camera_texture(
    &self,
    index: u32,
    texture: &ID3D11Texture2D,
    subresource: u32,
    size: (u32, u32),
    settings: &ScreenshotOutputSettings,
    overlay: crate::editor::CameraOverlaySettings,
    drop_shadow: bool,
    camera_on_top: bool,
    composition: ComposedFrame,
  ) -> Result<bool, String> {
    let Ok(mut state) = self.inner.state.lock() else {
      return Ok(false);
    };
    if index == 1 {
      if state
        .camera_source
        .as_ref()
        .is_none_or(|source| source.size != size)
      {
        state.camera_source = Some(
          self
            .inner
            .gpu
            .compositor
            .source(&self.inner.gpu.device, size)?,
        );
      }
      if let Some(camera) = &state.camera_source {
        compositor::Compositor::copy_source(&self.inner.gpu.context, camera, texture, subresource)?;
      }
    } else {
      state.primary_composition = Some(composition);
      let halo = recording_halo(&state, 0);
      let Some(pane) = state.panes.first_mut().and_then(Option::as_mut) else {
        return Ok(false);
      };
      pane.annotation_halo = halo;
      if pane
        .source
        .as_ref()
        .is_none_or(|source| source.size != size)
      {
        pane.source = Some(
          self
            .inner
            .gpu
            .compositor
            .source(&self.inner.gpu.device, size)?,
        );
        pane.source_token = None;
      }
      let source = pane
        .source
        .as_ref()
        .ok_or_else(|| "The preview source texture is unavailable".to_owned())?;
      compositor::Compositor::copy_source(&self.inner.gpu.context, source, texture, subresource)?;
      pane.source_token = None;
    }
    let Some(camera) = state.camera_source.clone() else {
      let Some(pane) = state.panes.first_mut().and_then(Option::as_mut) else {
        return Ok(true);
      };
      if pane.source.is_none() {
        return Ok(true);
      }
      let staged = self.present_cached_source(pane, settings, composition)?;
      redraw_stale_selection(&self.inner, &mut state);
      return Ok(staged);
    };
    let composition = if index == 1 {
      state.primary_composition.unwrap_or(composition)
    } else {
      composition
    };
    let geometry = crate::editor::media_preview::bake_geometry(BakedVideoExportOptions {
      camera_drop_shadow: drop_shadow,
      camera_height: camera.size.1,
      camera_width: camera.size.0,
      overlay,
      screen_height: settings.height,
      screen_width: settings.width,
      video: VideoExportOptions {
        compression: 0,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
    })?;
    let Some(pane) = state.panes.first_mut().and_then(Option::as_mut) else {
      return Ok(true);
    };
    if pane.source.is_none() {
      return Ok(true);
    }
    let staged = self.present_cached_source_with_camera(
      pane,
      settings,
      composition,
      Some((&camera, geometry, drop_shadow, camera_on_top)),
    )?;
    redraw_stale_selection(&self.inner, &mut state);
    Ok(staged)
  }
}

impl RecordingPreviewSurface {
  /// Re-presents the frame a pane already holds with `annotations` in place of
  /// the annotations it was last composed with. This is what an arrow drag
  /// needs per pointer sample: the picture has not changed, only the
  /// annotations over it, so asking the decoder for the frame again would put a
  /// seek between the hand and the arrow. The Metal workspace does the same
  /// through `redraw_recording_workspace`.
  ///
  /// `source` is the full-resolution grid the annotations are authored in; they
  /// are moved onto the decoded frame's own grid here, as a live present does.
  pub(crate) fn redraw_recording_annotations(
    &self,
    index: u32,
    annotations: &[crate::editor::annotations::Annotation],
    source: (u32, u32),
  ) -> bool {
    let Ok(mut state) = self.inner.state.lock() else {
      return false;
    };
    let halo = recording_halo(&state, index);
    let camera_source = state.camera_source.clone();
    let Some(pane) = state.panes.get_mut(index as usize).and_then(Option::as_mut) else {
      return false;
    };
    let (Some(mut settings), Some(composition), Some(decoded)) = (
      pane.settings.clone(),
      pane.last_composition,
      pane.source.as_ref().map(|source| source.size),
    ) else {
      return false;
    };
    pane.annotation_halo = halo;
    settings.annotations = annotations.to_vec();
    let output_width = settings.width;
    crate::editor::recording_preview_player::annotation_preview::remap_source(
      &mut settings,
      source,
      decoded,
      output_width,
    );
    let camera = match (pane.last_camera, camera_source.as_ref()) {
      (Some((geometry, drop_shadow, camera_on_top)), Some(camera)) => {
        Some((camera, geometry, drop_shadow, camera_on_top))
      }
      (Some(_), None) => return false,
      (None, _) => None,
    };
    self
      .present_cached_source_with_camera(pane, &settings, composition, camera)
      .unwrap_or(false)
  }
}

/// The halo this pane's annotations carry, if the hovered arrow is one of them.
fn recording_halo(state: &SurfaceState, index: u32) -> Option<(usize, f32)> {
  state
    .annotation
    .hover
    .filter(|(pane, _, _)| *pane == u64::from(index))
    .map(|(_, index, width)| (index, width))
}
