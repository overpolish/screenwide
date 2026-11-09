// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Ruler and OCR previews played over time, offscreen, as the settings
//! window's thread plays them: the page's inputs at their moments, and a
//! frame whenever the renderer asks for one. `SCREENWIDE_PREVIEW_SCENE`
//! names a folder holding a scene exported from the page (`ruler.json` and
//! `ruler.rgba`, `ocr.json` and `ocr.rgba`) to play instead of the stand-in
//! below; `SCREENWIDE_PREVIEW_FRAMES` names a folder the frames are written
//! to, a tenth of a second apart.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use screenwide_imaging::CapturedImage;
use screenwide_osc::geometry::Size;
use screenwide_settings_preview::scene::PreviewInput;
use screenwide_settings_preview::Renderer;
use serde::Deserialize;

use super::ToolRenderer;
use crate::app_windows::screenshot_region::region_osc::preview::PreviewSurface;
use crate::settings_preview::tool_points;

/// The settings picture's strip, in points, at a Retina display's scale.
const POINTS: (f64, f64) = (1600.0, 160.0);
const SCALE: f64 = 2.0;
/// How far apart the frames written are.
const EVERY: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Deserialize)]
struct Step {
  /// Seconds into the pass.
  t: f64,
  input: PreviewInput,
  x: f64,
  y: f64,
}

#[derive(Deserialize)]
struct Scene {
  width: u32,
  height: u32,
  points: (f64, f64),
  /// How long the pass lasts, past its last input.
  duration: f64,
  steps: Vec<Step>,
}

/// A grey desktop with two light windows, where the Ruler's scene has them.
fn stand_in_picture() -> CapturedImage {
  let (width, height) = ((POINTS.0 * SCALE) as u32, (POINTS.1 * SCALE) as u32);
  let windows = [(550.0, 18.0, 214.0, 124.0), (822.0, 40.0, 206.0, 112.0)];
  let mut rgba = Vec::with_capacity((width * height * 4) as usize);
  for y in 0..height {
    for x in 0..width {
      let (px, py) = (f64::from(x) / SCALE, f64::from(y) / SCALE);
      let inside = windows
        .iter()
        .any(|&(wx, wy, ww, wh)| px >= wx && px < wx + ww && py >= wy && py < wy + wh);
      let shade = if inside { 240 } else { 70 };
      rgba.extend_from_slice(&[shade, shade, shade + 10, 255]);
    }
  }
  CapturedImage {
    rgba,
    width,
    height,
  }
}

/// The gap hovered, the second window measured by a drag across it, then
/// the pointer resting inside the first.
fn stand_in_steps() -> Vec<Step> {
  let step = |t, input, x, y| Step { t, input, x, y };
  let mut steps = vec![step(0.0, PreviewInput::Reset, 0.0, 0.0)];
  steps.extend((0..10).map(|i| step(0.05 * f64::from(i), PreviewInput::Hover, 793.0, 96.0)));
  steps.push(step(0.6, PreviewInput::Down, 814.0, 32.0));
  steps.extend((1..=10).map(|i| {
    let amount = f64::from(i) / 10.0;
    step(
      0.6 + 0.05 * f64::from(i),
      PreviewInput::Drag,
      814.0 + 222.0 * amount,
      32.0 + 128.0 * amount,
    )
  }));
  steps.push(step(1.15, PreviewInput::Up, 1036.0, 160.0));
  steps.push(step(1.3, PreviewInput::Hover, 667.0, 92.0));
  steps
}

/// The scene exported from the page for `name`, if a folder is given.
fn exported(name: &str) -> Option<(CapturedImage, Size, f64, Vec<Step>)> {
  let folder = PathBuf::from(std::env::var_os("SCREENWIDE_PREVIEW_SCENE")?);
  let json = std::fs::read(folder.join(format!("{name}.json"))).expect("the scene's script");
  let scene: Scene = serde_json::from_slice(&json).expect("the scene's script reads");
  let rgba = std::fs::read(folder.join(format!("{name}.rgba"))).expect("the scene's picture");
  let picture = CapturedImage {
    rgba,
    width: scene.width,
    height: scene.height,
  };
  let size = Size {
    width: scene.points.0,
    height: scene.points.1,
  };
  // Two passes, as the page loops them: the recognizer's first answer in a
  // process can take seconds while its models load.
  let passes = (0..2)
    .flat_map(|pass| {
      scene.steps.iter().map(move |step| Step {
        t: step.t + scene.duration * f64::from(pass),
        ..*step
      })
    })
    .collect();
  Some((picture, size, scene.duration * 2.0, passes))
}

fn target(picture: &CapturedImage) -> wgpu::Texture {
  screenwide_gpu::shared()
    .expect("the GPU")
    .device
    .create_texture(&wgpu::TextureDescriptor {
      label: None,
      size: wgpu::Extent3d {
        width: picture.width,
        height: picture.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: screenwide_gpu::surface::FORMAT,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    })
}

/// What the preview shows, as BGRA rows.
fn read(texture: &wgpu::Texture) -> Vec<u8> {
  let gpu = screenwide_gpu::shared().expect("the GPU");
  gpu.read_texture(texture).expect("the frame reads back")
}

fn keep(folder: &Path, name: &str, index: usize, texture: &wgpu::Texture, pixels: &[u8]) {
  let rgba: Vec<u8> = pixels
    .as_chunks::<4>()
    .0
    .iter()
    .flat_map(|bgra| [bgra[2], bgra[1], bgra[0], 255])
    .collect();
  let image =
    image::RgbaImage::from_raw(texture.width(), texture.height(), rgba).expect("the frame's size");
  std::fs::create_dir_all(folder).expect("the frames' folder");
  image
    .save(folder.join(format!("{name}-{index:03}.png")))
    .expect("the frame saves");
}

/// Plays `name`'s pass in real time and answers the frames read at each
/// tenth of a second.
fn play(name: &str, ruler: bool) -> Vec<Vec<u8>> {
  let (picture, size, duration, steps) = exported(name).unwrap_or_else(|| {
    (
      stand_in_picture(),
      Size {
        width: POINTS.0,
        height: POINTS.1,
      },
      0.0,
      stand_in_steps(),
    )
  });
  // The page's picture at the tool's scale, as the app opens it.
  let size = Size {
    width: tool_points(size.width),
    height: tool_points(size.height),
  };
  let texture = target(&picture);
  let scale = f64::from(picture.width) / size.width;
  let surface = PreviewSurface::offscreen(texture.clone(), scale).expect("the surface");
  let mut renderer = ToolRenderer::on(surface, ruler, picture, size);
  let frames_folder = std::env::var_os("SCREENWIDE_PREVIEW_FRAMES").map(PathBuf::from);
  let length = duration.max(steps.last().map_or(0.0, |step| step.t) + 0.8);
  let start = Instant::now();
  let mut due = None;
  let mut next_step = 0;
  let mut frames = Vec::new();
  while start.elapsed().as_secs_f64() < length {
    let now = start.elapsed().as_secs_f64();
    while let Some(step) = steps.get(next_step).filter(|step| step.t <= now) {
      if let Some(wait) = renderer.input(step.input, tool_points(step.x), tool_points(step.y)) {
        due = Some(due.map_or(Instant::now() + wait, |due: Instant| {
          due.min(Instant::now() + wait)
        }));
      }
      next_step += 1;
    }
    if due.is_some_and(|due| due <= Instant::now()) {
      due = renderer.frame().map(|wait| Instant::now() + wait);
    }
    if start.elapsed() >= EVERY * frames.len() as u32 {
      let pixels = read(&texture);
      if let Some(folder) = &frames_folder {
        keep(folder, name, frames.len(), &texture, &pixels);
      }
      frames.push(pixels);
    }
    std::thread::sleep(Duration::from_millis(4));
  }
  frames
}

/// How many pixels differ between two frames.
fn changed(before: &[u8], after: &[u8]) -> usize {
  before
    .as_chunks::<4>()
    .0
    .iter()
    .zip(after.as_chunks::<4>().0)
    .filter(|(a, b)| a != b)
    .count()
}

#[test]
#[cfg_attr(
  all(screenwide_no_gpu, target_os = "macos"),
  ignore = "requires a hardware GPU"
)]
fn the_ruler_measures_over_time() {
  let frames = play("ruler", true);
  let first = frames.first().expect("a frame");
  assert!(
    frames.iter().any(|frame| changed(first, frame) > 0),
    "the Ruler draws what the pointer measures"
  );
}

#[test]
#[cfg_attr(screenwide_no_gpu, ignore = "requires a hardware GPU")]
fn ocr_chooses_an_area_over_time() {
  let frames = play("ocr", false);
  let first = frames.first().expect("a frame");
  assert!(
    frames.iter().any(|frame| changed(first, frame) > 0),
    "OCR draws the area being chosen"
  );
}
