// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  pub(crate) fn draw(&mut self) {
    if self.drawing {
      self.pending = true;
      return;
    }
    self.drawing = true;
    loop {
      self.pending = false;
      if let Err(error) = self.render() {
        eprintln!("The Windows region OSC frame was dropped: {error}");
      }
      if !self.pending {
        break;
      }
    }
    self.drawing = false;
  }

  pub(super) fn render(&mut self) -> Result<(), String> {
    let (width, height) = self.client_size();
    if !should_show(self.is_root(), self.visible, self.desktop_presented)
      || width == 0
      || height == 0
    {
      if self.shown {
        self.shown = false;
        let _ = unsafe { ShowWindowAsync(self.hwnd, SW_HIDE) };
      }
      self.release_drawables();
      return Ok(());
    }
    self.drawables_released = false;
    // Layout is only re-asserted when it actually changes: a pointer move
    // must not post a `SetWindowPos` per frame.
    if !self.shown || self.window_size != (width, height) {
      self.shown = true;
      self.window_size = (width, height);
      let _ = unsafe { ShowWindowAsync(self.hwnd, SW_SHOWNOACTIVATE) };
      self.raise();
    }

    let scale = self.scale().max(0.1);
    // Geometry is built in logical points, exactly as the macOS master frame
    // passed `host.bounds.size`; only the magnifier quad works in the physical
    // pixels its constants and `SV_Position` use.
    let view = Size {
      width: f64::from(width) / scale,
      height: f64::from(height) / scale,
    };
    let pixels = Size {
      width: f64::from(width),
      height: f64::from(height),
    };
    let canvas = Rect::from_xywh(0.0, 0.0, view.width, view.height);

    let light = light_mode();
    let now = std::time::Instant::now();
    let mut constants = RenderConstants::new(light);
    // The ruler animation row is frame-global: the world halos read its hover
    // alpha and width, and the loupe reads the copied and tolerance progress.
    constants.ruler_sample = [
      f32::from((self.ruler.color >> 24) as u8) / 255.0,
      f32::from((self.ruler.color >> 16) as u8) / 255.0,
      f32::from((self.ruler.color >> 8) as u8) / 255.0,
      f32::from(self.ruler.color as u8) / 255.0,
    ];
    constants.ruler_animation = [
      self.ruler.copied_amount(now) as f32,
      self.ruler.hover_alpha() as f32,
      (self.ruler.hover_width(now) * scale) as f32,
      self.ruler.tolerance_amount(now) as f32,
    ];
    // The lens belongs to the surface the pointer is on; peers never carry an
    // anchor (`updateMagnifier` required `s == root`).
    let magnifier_active = self.show_frame
      && self.input_enabled
      && self.magnifier.is_some()
      && self.magnifier_source.is_some();
    if magnifier_active {
      if let (Some(anchor), Some(source)) = (self.magnifier, self.magnifier_source.as_ref()) {
        constants.set_magnifier(
          anchor.point,
          scale,
          anchor.edges,
          source.size,
          (
            (anchor.point.x / view.width.max(1.0)) as f32,
            (anchor.point.y / view.height.max(1.0)) as f32,
          ),
          (0.0, 0.0),
          (1.0, 1.0),
        );
      }
    } else {
      constants.clear_magnifier();
    }

    let mut vertices = std::mem::take(&mut self.vertices);
    vertices.clear();
    let mut segments: Vec<Segment> = Vec::new();
    // macOS drew the non-composited snapshot through a `CALayer` under the
    // Metal layer. One full-viewport quad in the same pass is visually
    // equivalent and saves a second DirectComposition visual; only the blend
    // state still distinguishes the composited path.
    let snapshot_drawn = self.snapshot_presented && self.snapshot.is_some();
    let snapshot_source = if snapshot_drawn && self.snapshot_composited {
      self.ruler.snapshot_uv(view)
    } else {
      Rect::from_xywh(0.0, 0.0, 1.0, 1.0)
    };
    if snapshot_drawn {
      let (source_width, source_height) =
        self.snapshot.as_ref().map_or((1, 1), |source| source.size);
      constants.chrome_backdrop = [
        pixels.width as f32,
        pixels.height as f32,
        1.0 / source_width.max(1) as f32,
        1.0 / source_height.max(1) as f32,
      ];
      constants.chrome_source = [
        snapshot_source.origin.x as f32,
        snapshot_source.origin.y as f32,
        snapshot_source.size.width as f32,
        snapshot_source.size.height as f32,
      ];
    }
    if snapshot_drawn {
      // In the composited (Ruler) mode the uv window is the display's zoomed
      // viewport, so panning and zooming move the frozen desktop itself.
      renderer::add_texture_quad(&mut vertices, view, canvas, snapshot_source, 33);
    }
    // `region` is desktop-global; every builder works in surface-local points.
    let region = self.local_rect(self.region);
    if is_empty(region) {
      renderer::add_quad(&mut vertices, view, canvas, 6);
    } else {
      renderer::add_crop_with_handles(
        &mut vertices,
        view,
        region,
        canvas,
        scale,
        0.0, // A screen capture region has square corners.
        self.show_frame,
        self.show_handles,
      );
    }
    // Highlights sit in world space with the region, so they join the base
    // run rather than a chrome draw call of their own.
    self
      .ocr
      .add_world_vertices(&mut vertices, view, region, scale);
    self.ruler.add_world_vertices(
      &mut vertices,
      view,
      scale,
      self.display_id,
      self.desktop_offset,
      now,
    );
    let chrome_start = vertices.len();
    // macOS drew every control into its own material surface; folding them in
    // here costs one draw call per control, because each carries its own fill
    // and foreground in the constant buffer.
    self.ocr.add_chrome_vertices(
      self.gpu.device(),
      &mut vertices,
      &mut segments,
      view,
      region,
      scale,
      light,
    );
    // Ruler labels and the loupe fold in the same way, one segment each.
    self.ruler.add_chrome_vertices(
      self.gpu.device(),
      &mut vertices,
      &mut segments,
      view,
      self.display_id,
      self.desktop_offset,
      scale,
      light,
      now,
    );
    if let Some(base) = Segment::base(0, chrome_start) {
      segments.insert(0, base);
    }
    // The lens is drawn last: nothing may overdraw it, which is why the Metal
    // cutout disappeared with the compute pass.
    let lens_start = vertices.len();
    renderer::add_magnifier(&mut vertices, pixels, &constants);
    segments.extend(Segment::base(lens_start, vertices.len()));
    let result = self.submit(&vertices, &segments, &constants, (width, height));
    self.vertices = vertices;
    // Control transitions, the confirm crossfade and every ruler transition
    // are time based, so a frame is retimed while one is running
    // (`+ocr_cancel.m:92-102`, `+ruler.m:577-587`).
    self.set_animating(self.ocr.is_animating() || self.ruler.is_animating(now));
    result
  }
}
