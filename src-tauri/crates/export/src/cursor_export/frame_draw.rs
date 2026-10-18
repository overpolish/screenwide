// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One exported frame drawn: what the scenes, the annotations, the cursor,
//! the keyboard, the captions and the camera put on the canvas at a moment,
//! decided here for every platform. The platform decodes the frames into
//! textures, hands them over, and encodes the canvas this draws.

use screenwide_compositor::annotation_gpu::{caption_overlay, prepared_arrows};
use screenwide_compositor::compositor::cursor_placement::placed_cursor;
use screenwide_compositor::compositor::{
  BakeGeometry, CameraComposition, ComposedFrame, Compositor, SourceTexture,
};
use screenwide_cursor_effects::{CursorCompositor, CursorEffectSettings};
use screenwide_keyboard_effects::{KeyboardCompositor, KeyboardEffectSettings};
use screenwide_output::ScreenshotOutputSettings;
use screenwide_text::captions::{CaptionMoment, CaptionTrack};
use screenwide_timeline_edit::TimelinePlan;

use super::camera_effects::FrameEffects;
use super::export_output::on_canvas;
use super::frame_annotations::ExportAnnotations;
use super::frame_scene::ExportScenes;
use super::CursorExportRequest;
use crate::media_preview;

/// The slot a baked camera's annotations are composed into its frame in.
const CAMERA_SLOT: usize = 0;

pub(super) struct FrameDraw<'a> {
  output: &'a ScreenshotOutputSettings,
  timeline: Option<&'a TimelinePlan>,
  annotations: ExportAnnotations,
  scenes: ExportScenes,
  cursor: Option<CursorCompositor>,
  cursor_effects: CursorEffectSettings,
  keyboard: Option<KeyboardCompositor>,
  keyboard_effects: KeyboardEffectSettings,
  captions: Option<&'a CaptionTrack>,
  /// Where the camera sits outside every scene, whether it casts a shadow,
  /// and whether it is drawn over the screen's annotations.
  camera: Option<(BakeGeometry, bool, bool)>,
  /// The camera's effects, on the baked-in camera or on the screen where the
  /// screen is the camera.
  effects: Option<FrameEffects>,
}

impl<'a> FrameDraw<'a> {
  /// `annotations_within_ms` is the stretch a preview draws; see
  /// `ExportAnnotations`.
  pub(super) fn new<'r: 'a>(
    request: &CursorExportRequest<'r>,
    gpu: &'static screenwide_gpu::Gpu,
    annotations_within_ms: Option<[u64; 2]>,
  ) -> Result<Self, String> {
    let cursor = request
      .cursor
      .map(|path| CursorCompositor::open(path, request.keyboard))
      .transpose()?;
    let keyboard = request
      .keyboard
      .map(|path| {
        KeyboardCompositor::open_with_deleted(
          path,
          request
            .timeline
            .map_or(&[], |timeline| timeline.deleted_keyboard_shortcut_ids()),
          request
            .timeline
            .map_or(&[], |timeline| timeline.deleted_keyboard_shortcut_ranges()),
        )
      })
      .transpose()?;
    // The camera is placed at the output's reference size; a scaled export
    // draws on a canvas of another size.
    let camera = request
      .camera
      .map(|(_, options)| {
        Ok::<_, String>((
          on_canvas(media_preview::bake_geometry(options)?, request.output),
          options.camera_drop_shadow,
          request.camera_on_top,
        ))
      })
      .transpose()?;
    Ok(Self {
      output: request.output,
      timeline: request.timeline,
      annotations: ExportAnnotations::for_request(request, annotations_within_ms),
      scenes: ExportScenes::for_request(request),
      cursor,
      cursor_effects: request.cursor_effects,
      keyboard,
      keyboard_effects: request.keyboard_effects,
      captions: request.captions,
      camera,
      effects: FrameEffects::new(request.camera_effects.as_ref(), gpu)?,
    })
  }

  /// Draws the frame `source_us` into the recording into `target`, a canvas
  /// of the output's size. `frame_ms` is how much output time the frame
  /// covers, which moving annotations and the captions are exposed over.
  /// `camera` is the decoded camera frame where the camera is baked in.
  pub(super) fn draw(
    &mut self,
    compositor: &Compositor,
    target: &wgpu::TextureView,
    screen: &SourceTexture,
    camera: Option<&SourceTexture>,
    (source_us, frame_ms): (u64, f32),
  ) -> Result<(), String> {
    // Annotated frames draw with the shader holding every kind; see
    // `hold_every_annotation_kind`.
    let _every = compositor.hold_every_annotation_kind();
    let source_ms = (source_us + 500) / 1_000;
    let screen = match self.effects.as_mut().filter(|effects| effects.on_screen()) {
      Some(effects) => effects.drawn(compositor, screen, source_ms)?,
      None => screen.clone(),
    };
    // A frame inside a scene is drawn on its own arrangement; every other
    // frame keeps the request's composition and the camera placed once.
    let arranged = self.scenes.at(self.output, source_ms, frame_ms);
    let settings = arranged
      .as_ref()
      .map_or(self.output, |(settings, _)| settings);
    let placement = match (&arranged, self.camera) {
      (Some((settings, Some(options))), Some((_, shadow, on_top))) => Some((
        on_canvas(media_preview::bake_geometry(*options)?, settings),
        shadow,
        on_top,
      )),
      _ => self.camera,
    };
    let annotations = self.annotations.with_camera_at(
      source_ms,
      frame_ms,
      settings,
      arranged.as_ref().and_then(|(_, options)| *options),
    );
    let camera = camera
      .zip(placement)
      .map(|(camera, placement)| {
        let camera = match self.effects.as_mut().filter(|effects| !effects.on_screen()) {
          Some(effects) => effects.drawn(compositor, camera, source_ms)?,
          None => camera.clone(),
        };
        // The redactions and highlights drawn on the camera change its own
        // pixels, so they are composed into its frame first.
        let composition = annotations
          .camera
          .as_ref()
          .map(|(settings, source)| CameraComposition::new(settings, *source, camera.size))
          .transpose()?
          .flatten();
        let camera = match composition {
          Some(composition) => {
            compositor.composed_camera(&camera, &composition, None, CAMERA_SLOT, false)?
          }
          None => camera,
        };
        Ok::<_, String>((camera, placement))
      })
      .transpose()?;
    let mut prepared =
      prepared_arrows(&annotations.screen, screen.size, settings, None, None, None)?;
    if let Some(track) = self.captions {
      let output_us = self
        .timeline
        .and_then(|plan| plan.source_to_output_us(source_us))
        .unwrap_or(source_us);
      prepared.draw_above(caption_overlay(
        track,
        CaptionMoment {
          at_ms: output_us as f64 / 1_000.0,
          exposure_ms: f64::from(frame_ms),
        },
        (self.output.width, self.output.height),
      ));
    }
    let cursor = match self.cursor.as_ref() {
      Some(cursor) => placed_cursor(
        cursor,
        source_ms,
        self.cursor_effects,
        screen.size,
        settings,
      )?,
      None => None,
    };
    let keyboard = self.keyboard.as_ref().and_then(|keyboard| {
      keyboard.evaluate_fitted_with_timeline(
        source_ms,
        self.keyboard_effects,
        (self.output.width, self.output.height),
        self.timeline,
      )
    });
    compositor.draw_with_camera(
      target,
      &screen,
      settings,
      ComposedFrame {
        cursor,
        keyboard,
        foreground_only: false,
        seconds: source_us as f64 / 1_000_000.0,
      },
      camera
        .as_ref()
        .map(|(camera, (geometry, shadow, on_top))| (camera, *geometry, *shadow, *on_top)),
      None,
      &prepared,
    )
  }
}
