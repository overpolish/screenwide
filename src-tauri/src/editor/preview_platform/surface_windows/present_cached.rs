// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  pub(super) fn present_cached_source(
    &self,
    pane: &mut Pane,
    settings: &ScreenshotOutputSettings,
    composition: ComposedFrame,
  ) -> Result<bool, String> {
    self.present_cached_source_with_camera(pane, settings, composition, None)
  }

  pub(super) fn present_cached_source_with_camera(
    &self,
    pane: &mut Pane,
    settings: &ScreenshotOutputSettings,
    composition: ComposedFrame,
    camera: Option<(&compositor::SourceTexture, BakeGeometry, bool, bool)>,
  ) -> Result<bool, String> {
    crate::screenshots::output_dimensions(settings)?;
    // Keep one stable output chain for the current preview resolution. The
    // source texture is cached separately and edits only redraw this target.
    // Buffers are only reallocated when the output outgrows them (with
    // headroom, so an interactive resize reallocates rarely rather than per
    // pointer move - per-move ResizeBuffers churn stalls the compositor).
    // The visual's scale transform and clip in `update_geometry` map and
    // bound exactly the drawn `content_size` region, so the unused margin of
    // the larger buffer is never composed. (`SetSourceSize` cannot express
    // this here: with the mandatory stretch scaling of a composition swap
    // chain it rescales the region to the buffer bounds and warps.)
    let output_size = (settings.width, settings.height);
    let resized = pane.content_size != output_size;
    if pane.buffer_size.0 < output_size.0 || pane.buffer_size.1 < output_size.1 {
      let buffer = (
        output_size.0.max(pane.buffer_size.0).next_multiple_of(256),
        output_size.1.max(pane.buffer_size.1).next_multiple_of(256),
      );
      unsafe {
        pane.swap_chain.ResizeBuffers(
          2,
          buffer.0,
          buffer.1,
          DXGI_FORMAT_B8G8R8A8_UNORM,
          DXGI_SWAP_CHAIN_FLAG(0),
        )
      }
      .map_err(|error| format!("The Windows composed preview could not resize: {error}"))?;
      pane.buffer_size = buffer;
    }
    if resized {
      pane.content_size = output_size;
      pane.selection_stale = true;
    }
    pane.last_composition = Some(composition);
    pane.last_camera = camera
      .map(|(_, geometry, drop_shadow, camera_on_top)| (geometry, drop_shadow, camera_on_top));
    pane.settings = Some(settings.clone());
    let source = pane
      .source
      .as_ref()
      .ok_or_else(|| "The preview source texture is unavailable".to_owned())?;
    // D3D11 flip-discard rotates the buffer identities after Present; buffer
    // zero is the writable back buffer for the next draw.
    let target = unsafe { pane.swap_chain.GetBuffer::<ID3D11Texture2D>(0) }
      .map_err(|error| format!("The composed preview has no back buffer: {error}"))?;
    let composed = &target;
    // A foreground layer blends over the existing target, and a flip-discard
    // back buffer is undefined after each present: its uncovered pixels must
    // read as transparent, not as stale frame data.
    if composition.foreground_only {
      let resource: ID3D11Resource = composed.cast().map_err(|error| error.to_string())?;
      let mut view: Option<ID3D11RenderTargetView> = None;
      unsafe {
        self
          .inner
          .gpu
          .device
          .CreateRenderTargetView(&resource, None, Some(&mut view))
      }
      .map_err(|error| format!("The layer preview could not clear its target: {error}"))?;
      if let Some(view) = view {
        unsafe {
          self
            .inner
            .gpu
            .context
            .ClearRenderTargetView(&view, &[0.0; 4])
        };
      }
    }
    let mut prepared = annotation::prepared_arrows(
      &settings.annotations,
      source.size,
      settings,
      source.picture.as_deref(),
      pane.annotation_halo,
      pane.annotation_typing,
    )?;
    // The composed canvas is scaled onto the pane's box by the visual, so
    // one drawn pixel covers this many canvas pixels.
    prepared.pixel_scale = if pane.display_size.0 > 0 {
      output_size.0 as f32 / pane.display_size.0 as f32
    } else {
      1.0
    };
    self.inner.gpu.compositor.draw_with_camera(
      &self.inner.gpu.context,
      composed,
      source,
      settings,
      composition,
      camera,
      pane.magnifier,
      &prepared,
    )?;
    unsafe { self.inner.gpu.context.Flush() };
    // Inside an open batch the frame is parked: the closing guard presents
    // every pane and commits every pending geometry in one flush, so sibling
    // layers change on the same compositor pass.
    if self.inner.batch_depth.load(Ordering::Acquire) > 0 {
      pane.pending_present = true;
      if resized {
        pane.pending_geometry = true;
      }
      return Ok(true);
    }
    unsafe { pane.swap_chain.Present(0, DXGI_PRESENT(0)) }
      .ok()
      .map_err(|error| format!("The composed preview could not present: {error}"))?;
    // Publish resized or deferred geometry only after the replacement frame
    // exists, immediately behind its present so both land in one pass.
    if resized || pane.pending_geometry {
      pane.pending_geometry = false;
      pane.update_geometry().map_err(|error| error.to_string())?;
      unsafe { self.inner.gpu.composition.Commit() }.map_err(|error| error.to_string())?;
    }
    Ok(true)
  }
}
