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
    // Keep one stable output surface for the current preview resolution. The
    // source texture is cached separately and edits only redraw this target.
    // The surface is only reallocated when the output outgrows it (with
    // headroom, so an interactive resize reallocates rarely rather than per
    // pointer move - per-move reallocation churn stalls the compositor).
    // The visual's scale transform and clip in `update_geometry` map and
    // bound exactly the drawn `content_size` region, so the unused margin of
    // the larger surface is never composed. (`SetSourceSize` cannot express
    // this here: with the mandatory stretch scaling of a composition swap
    // chain it rescales the region to the buffer bounds and warps.)
    let gpu = self.inner.gpu.shared;
    let output_size = (settings.width, settings.height);
    let resized = pane.content_size != output_size;
    let held = pane.surface.size();
    if held.0 < output_size.0 || held.1 < output_size.1 {
      // No frame may be held across a resize; a parked one is redrawn below.
      pane.parked = None;
      pane.surface.resize(
        gpu,
        (
          output_size.0.max(held.0).next_multiple_of(256),
          output_size.1.max(held.1).next_multiple_of(256),
        ),
      );
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
    // A frame drawn twice inside one batch is drawn over again rather than
    // acquiring another; only the last drawing is ever presented.
    let frame = match pane.parked.take() {
      Some(frame) => frame,
      None => match pane.surface.acquire(gpu)? {
        Frame::Ready(frame) => frame,
        Frame::Skipped => return Ok(false),
      },
    };
    // A foreground layer blends over the target, and a fresh swap-chain
    // texture is undefined: its uncovered pixels must read as transparent,
    // not as stale frame data.
    if composition.foreground_only {
      pane::clear(gpu, &frame.texture, wgpu::Color::TRANSPARENT);
    }
    let mut prepared = crate::editor::preview_platform::annotation_gpu::prepared_arrows(
      &settings.annotations,
      source.size,
      settings,
      source.picture.as_deref(),
      pane.annotation_halo,
      pane.annotation_typing,
    )?;
    // The composed canvas is scaled onto the pane's box by the visual, so
    // one drawn pixel covers this many canvas pixels. The canvas is drawn at
    // its own resolution and magnified by the visual when zoomed in, so a
    // canvas pixel is the finest thing it holds: edges smoothed over less
    // than one come out hard and are enlarged into steps.
    prepared.pixel_scale = if pane.display_size.0 > 0 {
      (output_size.0 as f32 / pane.display_size.0 as f32).max(1.0)
    } else {
      1.0
    };
    self.inner.gpu.compositor.draw_with_camera(
      &frame.texture.create_view(&Default::default()),
      source,
      settings,
      composition,
      camera,
      pane.magnifier,
      &prepared,
    )?;
    // Inside an open batch the frame is parked: the closing guard presents
    // every pane and commits every pending geometry in one flush, so sibling
    // layers change on the same compositor pass.
    if self.inner.batch_depth.load(Ordering::Acquire) > 0 {
      pane.parked = Some(frame);
      if resized {
        pane.pending_geometry = true;
      }
      return Ok(true);
    }
    gpu.queue.present(frame);
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
