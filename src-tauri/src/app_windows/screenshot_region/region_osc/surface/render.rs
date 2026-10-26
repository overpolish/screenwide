// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// How wide the picking backdrop's blur is, in points: slight, so the other
/// windows stay recognisable.
const BACKDROP_BLUR: f64 = 4.0;

/// How wide the edge between a picked window and the screen round it
/// feathers, in points, half inside the window and half out.
const CUTOUT_FEATHER: f64 = 3.0;

/// How much of the overlay's shade goes over the picking backdrop, which the
/// blur already pushes back.
const BACKDROP_SHADE: f32 = 0.75;

/// How round a window's corners are, in points, so a picked window's cutout
/// sits on it: macOS 26 rounds its windows well beyond the 10 points before
/// it; Windows 11 rounds them 8 pixels.
#[cfg(target_os = "macos")]
fn window_corner_radius() -> f64 {
  let version = objc2_foundation::NSProcessInfo::processInfo().operatingSystemVersion();
  if version.majorVersion >= 26 {
    16.0
  } else {
    10.0
  }
}

#[cfg(not(target_os = "macos"))]
fn window_corner_radius() -> f64 {
  8.0
}

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
    let now = std::time::Instant::now();
    // A picked window's cutout fades out after it is hidden, so the surface
    // stays up until it has gone.
    let cut = self.window_highlight && self.visible && !is_empty(self.region);
    self.cutout.head_for(cut, now);
    let cutout = self.cutout.opacity(now);
    let showing = self.visible || (self.window_highlight && cutout > 0.0);
    let owns_frame = self.is_root() || self.is_preview();
    if !should_show(owns_frame, showing, self.desktop_presented) || width == 0 || height == 0 {
      if self.shown {
        self.shown = false;
        self.show_window(false);
      }
      self.release_drawables();
      self.set_animating(false);
      return Ok(());
    }
    self.drawables_released = false;
    // Layout is only re-asserted when it actually changes: a pointer move
    // must not post a `SetWindowPos` per frame.
    if !self.shown || self.window_size != (width, height) {
      self.shown = true;
      self.window_size = (width, height);
      self.show_window(true);
      self.raise();
    }

    let scale = self.scale().max(0.1);
    // Geometry is built in logical points, exactly as the macOS master frame
    // passed `host.bounds.size`; only the magnifier quad works in the physical
    // pixels its constants and the fragment position use.
    let view = Size {
      width: f64::from(width) / scale,
      height: f64::from(height) / scale,
    };
    let pixels = Size {
      width: f64::from(width),
      height: f64::from(height),
    };
    let canvas = Rect::from_xywh(0.0, 0.0, view.width, view.height);

    let light = host().light_mode();
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
    } else if self.window_highlight {
      let blur = match self.backdrop.as_ref() {
        Some(backdrop) => {
          let (source_width, source_height) = backdrop.size;
          constants.chrome_backdrop = [
            pixels.width as f32,
            pixels.height as f32,
            1.0 / source_width.max(1) as f32,
            1.0 / source_height.max(1) as f32,
          ];
          constants.chrome_source = [0.0, 0.0, 1.0, 1.0];
          constants.overlay_shade[3] *= BACKDROP_SHADE;
          BACKDROP_BLUR * scale
        }
        None => 0.0,
      };
      constants.window_cutout = [
        (region.origin.x * scale) as f32,
        (region.origin.y * scale) as f32,
        (region.size.width * scale) as f32,
        (region.size.height * scale) as f32,
      ];
      constants.window_shape = [
        (window_corner_radius() * scale) as f32,
        cutout,
        blur as f32,
        (CUTOUT_FEATHER * scale) as f32,
      ];
      renderer::add_window_highlight(&mut vertices, view, canvas);
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
      .add_world_vertices(&mut vertices, view, region, scale, now);
    self.ruler.add_world_vertices(
      &mut vertices,
      view,
      scale,
      self.display_id,
      self.desktop_offset,
      now,
    );
    let chrome_start = vertices.len();
    // Ruler labels and the loupe are chrome: one draw call each, because each
    // carries its own fill and foreground in the constant buffer.
    self.ruler.add_chrome_vertices(
      self.gpu.device(),
      &mut vertices,
      &mut segments,
      view,
      self.display_id,
      self.desktop_offset,
      scale,
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
    // The reading sheen and every ruler transition are time based, so a
    // frame is retimed while one is running.
    self.set_animating(
      self.ocr.is_animating() || self.ruler.is_animating(now) || self.cutout.is_fading(now),
    );
    result
  }
}
