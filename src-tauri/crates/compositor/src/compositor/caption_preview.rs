// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A caption style's preview: a few words said one after another, drawn by
//! the same shader and type the export uses, frame by frame, so a template's
//! tile shows exactly how its captions will look and move.

use screenwide_captions::{Caption, CaptionStyle, CaptionTrack, CaptionWord, RecordingCaptions};
use screenwide_imaging::CapturedImage;
use screenwide_output::{MeshGradientPoint, NormalizedSourceRect, ScreenshotOutputSettings};

use super::*;
use crate::annotation_gpu::{caption_overlay, prepared_arrows};

/// How long each sample word is said, and the pause after it, in
/// milliseconds.
const WORD_MS: f64 = 330.0;
const PAUSE_MS: f64 = 70.0;
/// How long the last caption holds before the preview starts again.
const HOLD_MS: f64 = 500.0;
/// How many sample words one caption holds at most: two fit a small tile.
const WORDS_PER_CAPTION: usize = 2;
/// How much larger than in a video the preview sets its type: at a tile's
/// size, type set as a video sets it would be too small to read.
const PREVIEW_SCALE: f64 = 2.5;

/// What a preview shows: its frames' size and how often it draws one.
pub struct CaptionPreview {
  pub width: u32,
  pub height: u32,
  pub frames_per_second: f64,
}

/// The sample `words` laid out as captions of a couple of words each, said
/// one after another from the start, the last held a moment.
fn sample_captions(words: &[&str], all_caps: bool) -> Vec<Caption> {
  let mut captions = Vec::new();
  let mut at = 0.0;
  for (index, chunk) in words.chunks(WORDS_PER_CAPTION).enumerate() {
    let start = at;
    let line: Vec<CaptionWord> = chunk
      .iter()
      .map(|word| {
        let text = if all_caps {
          word.to_uppercase()
        } else {
          (*word).to_owned()
        };
        let said = CaptionWord {
          text,
          start_ms: at,
          end_ms: at + WORD_MS,
          source_start_ms: at,
          source_end_ms: at + WORD_MS,
        };
        at += WORD_MS + PAUSE_MS;
        said
      })
      .collect();
    captions.push(Caption {
      id: index as u64,
      start_ms: start,
      end_ms: at,
      text: line
        .iter()
        .map(|word| word.text.as_str())
        .collect::<Vec<_>>()
        .join(" "),
      lines: vec![line],
      spaced: true,
    });
  }
  if let Some(last) = captions.last_mut() {
    last.end_ms += HOLD_MS;
  }
  captions
}

/// A dark picture for the captions to read over, lighter at the top, as a
/// frame of video behind them would be.
fn backdrop(width: u32, height: u32) -> CapturedImage {
  let mut rgba = Vec::with_capacity((width * height * 4) as usize);
  for y in 0..height {
    let share = f64::from(y) / f64::from(height.max(1));
    let shade = |top: f64, bottom: f64| (top + (bottom - top) * share).round() as u8;
    let pixel = [shade(64.0, 24.0), shade(70.0, 26.0), shade(92.0, 34.0), 255];
    for _ in 0..width {
      rgba.extend_from_slice(&pixel);
    }
  }
  CapturedImage {
    width,
    height,
    rgba,
  }
}

/// The whole canvas showing the picture, with nothing around it.
fn plain_settings(width: u32, height: u32) -> ScreenshotOutputSettings {
  ScreenshotOutputSettings {
    annotations: Vec::new(),
    background_color: "#000000".to_owned(),
    background_image_path: None,
    background_type: "solid".to_owned(),
    background_radius_percent: 0.0,
    capture_width_points: 0.0,
    capture_scale: 0.0,
    crop_height: f64::from(height),
    crop_width: f64::from(width),
    crop_preview: None,
    crop_x: 0.0,
    crop_y: 0.0,
    drop_shadow: false,
    height,
    image_width: f64::from(width),
    image_x: 0.0,
    image_y: 0.0,
    legacy_mode: None,
    scene_motion: None,
    scene_opacity: None,
    scene_camera_front: None,
    mesh_colors: Vec::new(),
    mesh_generator: String::new(),
    mesh_locked_colors: Vec::new(),
    mesh_points: Vec::<MeshGradientPoint>::new(),
    mesh_seed: 0,
    mesh_warp_percent: 0.0,
    radius_percent: 0.0,
    recenter_inset_color: None,
    source_crop: NormalizedSourceRect {
      height: 1.0,
      width: 1.0,
      x: 0.0,
      y: 0.0,
    },
    width,
  }
}

impl CaptionPreview {
  /// How many frames a preview of `words` sample words takes: one whole run
  /// of the sample, with its hold.
  pub fn frame_count(&self, words: usize) -> usize {
    let length_ms = words as f64 * (WORD_MS + PAUSE_MS) + HOLD_MS;
    (length_ms / 1_000.0 * self.frames_per_second)
      .ceil()
      .max(1.0) as usize
  }
}

/// A compositor kept for drawing caption previews. Making one compiles its
/// pipelines, which takes far longer than drawing a whole preview with it, so
/// one is made once and kept.
pub struct CaptionPreviewer {
  gpu: &'static Gpu,
  compositor: Compositor,
}

impl CaptionPreviewer {
  pub fn new(gpu: &'static Gpu) -> Result<Self, String> {
    let cursors = NativeCursors {
      size: (1, 1),
      layers: vec![0; 4 * 8],
      hotspots: [[0.0; 4]; 8],
    };
    Ok(Self {
      gpu,
      compositor: Compositor::new(gpu, cursors)?,
    })
  }

  /// `style`'s preview of the sample `words`: one RGBA frame after
  /// another, as `preview` asks, through one whole run of the sample.
  pub fn frames(
    &self,
    style: &CaptionStyle,
    words: &[&str],
    preview: &CaptionPreview,
  ) -> Result<Vec<Vec<u8>>, String> {
    let (gpu, compositor) = (self.gpu, &self.compositor);
    let (width, height) = (preview.width.max(1), preview.height.max(1));
    let mut style = style.normalized();
    style.size_percent =
      (style.size_percent * PREVIEW_SCALE).min(screenwide_captions::MAX_SIZE_PERCENT);
    let captions = sample_captions(words, style.all_caps);
    let track = CaptionTrack {
      captions,
      recording: RecordingCaptions {
        enabled: true,
        style,
        ..Default::default()
      },
    };
    let image = backdrop(width, height);
    let settings = plain_settings(width, height);
    let source = compositor.screenshot_source(&image)?;
    let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Screenwide caption preview"),
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: FORMAT,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    (0..preview.frame_count(words.len()))
      .map(|index| {
        let at_ms = index as f64 * 1_000.0 / preview.frames_per_second;
        let mut prepared =
          prepared_arrows(&[], (width, height), &settings, Some(&image), None, None)?;
        prepared.draw_above(caption_overlay(&track, at_ms, (width, height)));
        compositor.draw_with_camera(
          &view,
          &source,
          &settings,
          ComposedFrame {
            cursor: None,
            keyboard: None,
            foreground_only: false,
            seconds: 0.0,
          },
          None,
          None,
          &prepared,
        )?;
        let mut pixels = gpu.read_texture(&target)?;
        // The canvas is BGRA; a picture is handed on as RGBA.
        for pixel in pixels.as_chunks_mut::<4>().0 {
          pixel.swap(0, 2);
        }
        Ok(pixels)
      })
      .collect()
  }
}
