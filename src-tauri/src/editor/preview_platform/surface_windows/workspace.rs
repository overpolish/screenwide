// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  #[allow(dead_code)]
  pub(crate) fn present(&self, _index: u32, _image: &CapturedImage) -> bool {
    false
  }

  #[allow(clippy::too_many_arguments)]
  // Retained-workspace entry point; not wired on Windows yet.
  #[allow(dead_code)]
  pub(crate) fn present_recording_workspace(
    &self,
    layers: &[RecordingWorkspaceLayer<'_>],
  ) -> Result<bool, String> {
    if layers.is_empty() {
      return Ok(false);
    }
    let batch = self.present_batch();
    let mut presented = false;
    for layer in layers {
      let Some(source) = layer.source else {
        // Keep this explicit rather than silently reading back a native
        // texture. The Windows decoder's D3D11 path will provide this branch
        // once its zero-copy source contract is promoted.
        if layer.source_pixels.is_some() {
          return Err("Windows recording workspace pixel sources are not wired".to_owned());
        }
        return Err("Recording workspace layer has no source".to_owned());
      };
      // The retained pane topology is already laid out by
      // `layout_recording_workspace`; the shared workspace transform is
      // applied by the pane visuals, so no CPU composition or intermediate
      // webview transport is introduced here.
      presented |= self.present_composed(
        layer.pane_index,
        layer.source_token,
        source,
        &layer.settings,
        layer.seconds,
        layer.cursor,
        layer.camera,
        layer.overlay,
        layer.clip_cursor_at_video_edge,
      )?;
    }
    drop(batch);
    Ok(presented)
  }

  #[allow(clippy::too_many_arguments)]
  #[allow(dead_code)]
  pub(crate) fn present_composed_pixels(
    &self,
    _index: u32,
    _source_token: u64,
    _source_pixels: *mut std::ffi::c_void,
    _source_size: (u32, u32),
    _settings: &ScreenshotOutputSettings,
    _seconds: f64,
    _cursor: Option<&CapturedImage>,
    _camera: Option<&CapturedImage>,
    _camera_pixels: Option<*mut std::ffi::c_void>,
    _overlay: Option<&StillOverlay>,
    _clip_cursor_at_video_edge: bool,
  ) -> Result<bool, String> {
    Ok(false)
  }

  pub(crate) fn hide(&self) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.camera_source = None;
      state.primary_composition = None;
      let context = &self.inner.gpu.context;
      unsafe {
        let vertex_buffer: Option<ID3D11Buffer> = None;
        let stride = 0_u32;
        let offset = 0_u32;
        context.IASetVertexBuffers(
          0,
          1,
          Some(&raw const vertex_buffer),
          Some(&raw const stride),
          Some(&raw const offset),
        );
        context.PSSetShaderResources(0, Some(&[None, None, None, None, None]));
        context.OMSetRenderTargets(None, None);
      }
      if let Ok(mut selection) = self.inner.gpu.selection.lock() {
        if let Err(error) = selection.release_drawables(context) {
          eprintln!("{error}");
        }
      }
      self.inner.gpu.backdrop.hide();
      for pane in state.panes.iter_mut().flatten() {
        pane.hide();
        if let Err(error) = pane.release_drawables(context) {
          eprintln!("{error}");
        }
        // Windows surfaces live for the editor window, not for one preview
        // session. Reusable screenshot item IDs therefore cannot be allowed
        // to keep the previous session's immutable source texture alive.
        pane.source = None;
        pane.source_token = None;
        pane.settings = None;
        pane.magnifier = None;
        pane.pending_present = false;
      }
      state.editor_active = false;
      state.selection = None;
      state.selection_targets.clear();
      state.gesture = None;
      // A hidden surface has no drag to finish; never keep the DOM locked out
      // of the pane geometry behind it.
      state.frame_resize = None;
      state.frame_resize_committed = false;
      state.move_auto_fit = None;
      self.inner.editor.set_active(false);
      draw_selection(&self.inner, &state);
      let _ = unsafe { self.inner.gpu.composition.Commit() };
    }
  }
}
