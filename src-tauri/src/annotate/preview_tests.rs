// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The Annotate preview drawn offscreen. `SCREENWIDE_PREVIEW_FRAMES` names a
//! folder the frames are written to, so a run can be looked at.

use super::*;

use crate::settings_preview::tool_points;

/// The settings picture's strip, in points, at a Retina display's scale.
pub(super) const POINTS: (f64, f64) = (1600.0, 160.0);
pub(super) const SCALE: f64 = 2.0;

pub(super) const DOWN: u32 = 0;
pub(super) const DRAG: u32 = 1;
pub(super) const UP: u32 = 2;

/// Where `annotate-scene.ts` lays its window out on the strip, in the
/// picture's points: the arrow's tail on the desktop and its head at the
/// window's heading, and the Publish button the counter marks (its width
/// from the label set in the scene's type).
pub(super) const TAIL: (f64, f64) = (600.0, 32.0);
pub(super) const HEAD: (f64, f64) = (718.0, 66.0);
pub(super) const BUTTON: (f64, f64, f64, f64) = (907.0, 104.0, 75.0, 24.0);
/// The twin of `TAIL_REACH` in the scene and `COUNTER_TAIL_REACH` in the
/// annotations crate.
const TAIL_REACH: f64 = 1.5;
/// The twin of `TOOL_SCALE`: how big the picture is beside the desktop.
const TOOL_SCALE: f64 = 0.5;

/// Where a counter's tail touches the button: on its edge, on the side the
/// tail comes from (`counterTip` in the scene).
pub(super) fn counter_tip(angle: f64) -> (f64, f64) {
  let (x, y, width, height) = BUTTON;
  let (dx, dy) = (angle.cos(), angle.sin());
  let half = (width / 2.0, height / 2.0);
  let across = |half: f64, d: f64| {
    if d == 0.0 {
      f64::INFINITY
    } else {
      half / d.abs()
    }
  };
  let reach = across(half.0, dx).min(across(half.1, dy));
  (x + half.0 - dx * reach, y + half.1 - dy * reach)
}

/// Where the pointer drops a counter `size` across for its tail to land on
/// `tip` (`counterCentre` in the scene).
pub(super) fn counter_centre(tip: (f64, f64), size: f64, angle: f64) -> (f64, f64) {
  let reach = size / 2.0 * TOOL_SCALE * TAIL_REACH;
  (tip.0 - angle.cos() * reach, tip.1 - angle.sin() * reach)
}

/// A point in the page's picture, where the tool sees it.
pub(super) fn point((x, y): (f64, f64)) -> AnnotationPoint {
  AnnotationPoint {
    x: tool_points(x),
    y: tool_points(y),
  }
}

/// The glass as the settings window opens it: the picture's pixels, over a
/// desktop at the tool's scale, its counter `counter_size` across.
pub(super) fn glass(counter_size: f64, counter_angle: f64, keep: bool) -> Glass {
  let size = ((POINTS.0 * SCALE) as u32, (POINTS.1 * SCALE) as u32);
  let scale = f64::from(size.0) / tool_points(POINTS.0);
  let mut glass = Glass::new(size, scale).expect("the glass");
  glass.set_look(AnnotateLook {
    color: "#ff3b30".to_owned(),
    keep,
    counter_size,
    counter_angle,
  });
  glass
}

/// The glass drawn into a texture and read back, as BGRA rows.
pub(super) fn drawn(glass: &Glass) -> Vec<u8> {
  let gpu = glass.renderer.gpu();
  let (width, height) = glass.size;
  let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
    label: None,
    size: wgpu::Extent3d {
      width,
      height,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: screenwide_gpu::surface::FORMAT,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    view_formats: &[],
  });
  glass
    .draw_into(&target.create_view(&Default::default()))
    .expect("the glass draws");
  gpu.read_texture(&target).expect("the frame reads back")
}

/// The pixels the glass covers, as (x, y).
pub(super) fn covered(glass: &Glass, pixels: &[u8]) -> Vec<(f64, f64)> {
  let width = glass.size.0 as usize;
  pixels
    .as_chunks::<4>()
    .0
    .iter()
    .enumerate()
    .filter(|(_, pixel)| pixel[3] > 0)
    .map(|(index, _)| ((index % width) as f64, (index / width) as f64))
    .collect()
}

/// Writes the frame over a grey desktop with the scene's button outlined,
/// where `SCREENWIDE_PREVIEW_FRAMES` names a folder.
pub(super) fn keep_frame(glass: &Glass, pixels: &[u8], name: &str) {
  let Some(folder) = std::env::var_os("SCREENWIDE_PREVIEW_FRAMES") else {
    return;
  };
  let (width, height) = glass.size;
  let (bx, by, bw, bh) = BUTTON;
  let on_button_edge = |x: f64, y: f64| {
    let inside = |x0: f64, y0: f64, x1: f64, y1: f64| x >= x0 && x < x1 && y >= y0 && y < y1;
    let (x0, y0, x1, y1) = (bx * SCALE, by * SCALE, (bx + bw) * SCALE, (by + bh) * SCALE);
    inside(x0, y0, x1, y1) && !inside(x0 + 2.0, y0 + 2.0, x1 - 2.0, y1 - 2.0)
  };
  let rgba: Vec<u8> = pixels
    .as_chunks::<4>()
    .0
    .iter()
    .enumerate()
    .flat_map(|(index, bgra)| {
      let (x, y) = ((index as u32 % width) as f64, (index as u32 / width) as f64);
      let ground = if on_button_edge(x, y) { 230.0 } else { 96.0 };
      let under = ground * (1.0 - f32::from(bgra[3]) / 255.0);
      let [r, g, b] =
        [bgra[2], bgra[1], bgra[0]].map(|channel| (f32::from(channel) + under).round() as u8);
      [r, g, b, 255]
    })
    .collect();
  let image = image::RgbaImage::from_raw(width, height, rgba).expect("the frame's size");
  std::fs::create_dir_all(&folder).expect("the frames' folder");
  image
    .save(std::path::Path::new(&folder).join(format!("annotate-{name}.png")))
    .expect("the frame saves");
}

/// A counter dropped where the scene drops it lands with its tail's tip on
/// the button's edge and its disc outside the button, at the size and aim
/// the settings give it.
#[test]
fn the_counter_marks_the_button_where_the_scene_drops_it() {
  for (size, angle, name) in [
    (28.0, 0.0, "smallest-east"),
    (80.0, 0.0, "largest-east"),
    (56.0, std::f64::consts::FRAC_PI_2, "down"),
    (56.0, 2.4, "down-west"),
  ] {
    let mut glass = glass(size, angle, false);
    let tip = counter_tip(angle);
    let centre = counter_centre(tip, size, angle);
    glass.shape = AnnotationKind::Counter;
    glass.pointer(DOWN, point(centre));
    glass.pointer(UP, point(centre));
    let frame = drawn(&glass);
    keep_frame(&glass, &frame, &format!("counter-{name}"));
    let pixels = covered(&glass, &frame);
    // Along the tail, the farthest pixel is the tip and the nearest the
    // back of the disc; across it, the disc's sides sit either side of its
    // centre.
    let (dx, dy) = (angle.cos(), angle.sin());
    let along = |(x, y): (f64, f64)| x * dx + y * dy;
    let across = |(x, y): (f64, f64)| y * dx - x * dy;
    let extremes = |measure: &dyn Fn((f64, f64)) -> f64| {
      pixels
        .iter()
        .fold((f64::MAX, f64::MIN), |(low, high), &(x, y)| {
          let at = measure((x + 0.5, y + 0.5));
          (low.min(at), high.max(at))
        })
    };
    let ((back, front), (left, right)) = (extremes(&along), extremes(&across));
    let radius = size / 2.0 * TOOL_SCALE * SCALE;
    let scripted = |(x, y): (f64, f64)| (x * SCALE, y * SCALE);
    let (tip, centre) = (scripted(tip), scripted(centre));
    let drawn_tip = front - along(tip);
    let drawn_centre = (
      back + radius - along(centre),
      (left + right) / 2.0 - across(centre),
    );
    assert!(
      drawn_tip.abs() <= 2.0,
      "{name}: the tip is {drawn_tip:.1}px off the button's edge"
    );
    assert!(
      drawn_centre.0.abs() <= 2.0 && drawn_centre.1.abs() <= 2.0,
      "{name}: the disc's centre is {drawn_centre:?}px off the scripted point"
    );
    // Only the tip may reach over the button, never the disc onto its label.
    let (bx, by, bw, bh) = BUTTON;
    let over = pixels
      .iter()
      .filter(|&&(x, y)| {
        x > bx * SCALE + 3.0
          && x < (bx + bw) * SCALE - 3.0
          && y > by * SCALE + 3.0
          && y < (by + bh) * SCALE - 3.0
      })
      .count();
    assert_eq!(over, 0, "{name}: the counter covers the button's label");
  }
}

/// Annotate's largest counter and widest arrow, as set in its settings, sit
/// with the picture's small window at the picture's scale rather than at a
/// real screen's: a counter of 80 points is 40 of the picture's.
#[test]
fn annotations_are_drawn_at_the_pictures_scale() {
  let mut glass = glass(80.0, 0.0, false);
  glass.settings.default_width = 24.0;
  glass.shape = AnnotationKind::Counter;
  glass.pointer(DOWN, point((878.0, 82.0)));
  glass.pointer(UP, point((878.0, 82.0)));
  let counter = drawn(&glass);
  keep_frame(&glass, &counter, "largest-counter");
  let pixels = covered(&glass, &counter);
  let (top, bottom) = pixels
    .iter()
    .fold((f64::MAX, f64::MIN), |(top, bottom), &(_, y)| {
      (top.min(y), bottom.max(y))
    });
  let disc = 80.0 * SCALE * TOOL_SCALE;
  assert!(
    (bottom + 1.0 - top - disc).abs() <= disc / 5.0,
    "the counter is {} pixels tall, not about {disc}",
    bottom + 1.0 - top
  );

  glass.reset();
  glass.pointer(DOWN, point(TAIL));
  glass.pointer(
    DRAG,
    point(((TAIL.0 + HEAD.0) / 2.0, (TAIL.1 + HEAD.1) / 2.0)),
  );
  glass.pointer(UP, point(HEAD));
  keep_frame(&glass, &drawn(&glass), "widest-arrow");
}
