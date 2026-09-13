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
    let Some(pane) = state.panes.get_mut(index as usize).and_then(Option::as_mut) else {
      return Ok(false);
    };
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
      let Some(pane) = state.panes.first_mut().and_then(Option::as_mut) else {
        return Ok(false);
      };
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
