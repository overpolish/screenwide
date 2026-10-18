// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A caption style's preview: a few words said one after another, drawn by
//! the same shader and type the export uses, frame by frame, so a template's
//! tile shows exactly how its captions will look and move.

use screenwide_imaging::CapturedImage;
use screenwide_output::{MeshGradientPoint, NormalizedSourceRect, ScreenshotOutputSettings};
use screenwide_text::captions::{
  caption_fit, sample_caption, Caption, CaptionFrame, CaptionMoment, CaptionTrack, CaptionWord,
  Measure, RecordingCaptions,
};
use screenwide_text::{TextArrangement, TextPreviewBackdrop, TextStyle, MIN_SIZE_PERCENT};

use super::*;
use crate::annotation_gpu::{caption_measure, caption_overlay, prepared_arrows, CaptionFaces};

/// How long each sample word is said, and the pause after it, in
/// milliseconds.
const WORD_MS: f64 = 330.0;
const PAUSE_MS: f64 = 70.0;
/// How long the last caption holds before the preview starts again.
const HOLD_MS: f64 = 500.0;
/// How many sample words one caption holds at most: two fit a small tile on
/// one line, and three show a stack or a staircase breaking into lines.
/// Words rolling through all go in one caption, so the tile shows them
/// roll.
fn words_per_caption(style: &TextStyle) -> usize {
  if style.rolling_words().is_some() {
    usize::MAX
  } else if style.arrangement == TextArrangement::Lines {
    2
  } else {
    3
  }
}
/// How much larger than in a video the preview sets its type: at a tile's
/// size, type set as a video sets it would be too small to read.
const PREVIEW_SCALE: f64 = 2.5;

/// What a preview shows: its frames' size and how often it draws one.
pub struct CaptionPreview {
  pub width: u32,
  pub height: u32,
  pub frames_per_second: f64,
}

/// The sample `words` laid out as captions of a few words each, as `style`
/// lays a caption out in `frame`, measured by `measure`, said one after
/// another from the start, the last held a moment.
fn sample_captions(
  words: &[&str],
  style: &TextStyle,
  frame: CaptionFrame,
  measure: &mut Measure,
) -> Vec<Caption> {
  let mut captions = Vec::new();
  let mut at = 0.0;
  for chunk in words.chunks(words_per_caption(style)) {
    let start = at;
    let said: Vec<CaptionWord> = chunk
      .iter()
      .map(|word| {
        let said = CaptionWord {
          text: (*word).to_owned(),
          said: (*word).to_owned(),
          start_ms: at,
          end_ms: at + WORD_MS,
          source_start_ms: at,
          source_end_ms: at + WORD_MS,
          emphasis: false,
        };
        at += WORD_MS + PAUSE_MS;
        said
      })
      .collect();
    let mut caption = sample_caption(&said, style, frame, measure);
    caption.start_ms = start;
    caption.end_ms = at;
    captions.push(caption);
  }
  if let Some(last) = captions.last_mut() {
    last.end_ms += HOLD_MS;
  }
  captions
}

/// A picture for the captions to read over, lighter at the top, as a frame
/// of video behind them would be: light, or dark where the template reads
/// best over that.
fn backdrop(width: u32, height: u32, kind: TextPreviewBackdrop) -> CapturedImage {
  // Each channel's value at the top and at the bottom.
  let [red, green, blue] = match kind {
    TextPreviewBackdrop::Light => [(244.0, 217.0), (245.0, 220.0), (248.0, 227.0)],
    TextPreviewBackdrop::Dark => [(64.0, 24.0), (70.0, 26.0), (92.0, 34.0)],
  };
  let mut rgba = Vec::with_capacity((width * height * 4) as usize);
  for y in 0..height {
    let share = f64::from(y) / f64::from(height.max(1));
    let shade = |(top, bottom): (f64, f64)| (top + (bottom - top) * share).round() as u8;
    let pixel = [shade(red), shade(green), shade(blue), 255];
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
    rotation: None,
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
    Ok(Self {
      gpu,
      compositor: Compositor::new(gpu)?,
    })
  }

  /// `style`'s preview of the sample `words` over `backdrop`: one RGBA frame
  /// after another, as `preview` asks, through one whole run of the sample.
  pub fn frames(
    &self,
    style: &TextStyle,
    words: &[&str],
    preview: &CaptionPreview,
    backdrop: TextPreviewBackdrop,
  ) -> Result<Vec<Vec<u8>>, String> {
    let (gpu, compositor) = (self.gpu, &self.compositor);
    let (width, height) = (preview.width.max(1), preview.height.max(1));
    let mut style = style.normalized();
    style.size_percent =
      (style.size_percent * PREVIEW_SCALE).min(screenwide_text::MAX_SIZE_PERCENT);
    let frame = CaptionFrame { width, height };
    let faces = CaptionFaces::of(&style);
    let mut measure = caption_measure(&faces);
    let mut captions = sample_captions(words, &style, frame, &mut measure);
    // Type too large for the tile is all set smaller alike, so the largest
    // caption fits and every one keeps its size beside the others.
    let fit = captions
      .iter()
      .map(|caption| caption_fit(caption, &style, frame, &mut measure))
      .fold(1.0, f64::min);
    if fit < 1.0 {
      style.size_percent = (style.size_percent * fit).max(MIN_SIZE_PERCENT);
      captions = sample_captions(words, &style, frame, &mut measure);
    }
    let track = CaptionTrack {
      captions,
      recording: RecordingCaptions {
        enabled: true,
        style,
        ..Default::default()
      },
      editing: None,
    };
    let image = self::backdrop(width, height, backdrop);
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
    // Each frame is exposed over the time since the one before, as a video's
    // is, so a tile shows the motion blurred as an export draws it.
    let frame_ms = 1_000.0 / preview.frames_per_second;
    (0..preview.frame_count(words.len()))
      .map(|index| {
        let moment = CaptionMoment {
          at_ms: index as f64 * frame_ms,
          exposure_ms: frame_ms,
        };
        let mut prepared =
          prepared_arrows(&[], (width, height), &settings, Some(&image), None, None)?;
        prepared.draw_above(caption_overlay(&track, moment, (width, height)));
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
