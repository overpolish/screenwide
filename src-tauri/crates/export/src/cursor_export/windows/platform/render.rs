// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Draws and encodes `request`'s edited video into `path`, or only
/// `stretch` of it, its samples then timed from the stretch's start.
/// Answers how the run ended and how many frames it wrote.
pub(super) fn render_video(
  request: &mut CursorExportRequest<'_>,
  path: &Path,
  stretch: Option<&media_preview::PreviewStretch>,
) -> Result<(ExportRunResult, u64), String> {
  screenwide_output::validate_output_settings(request.width, request.height, request.output)?;
  let surface = (request.surface)()?;
  let output_size = screenwide_output::output_dimensions(request.output)?;
  let compositor = WindowsExportCompositor::new(
    surface,
    (request.width, request.height),
    request
      .camera
      .map(|(_, options)| (options.camera_width, options.camera_height)),
    output_size,
  )?;
  let device = compositor.device();
  let mut reader = GpuVideoReader::open(request.screen, 0, device.clone())?;
  let mut camera_reader = request
    .camera
    .map(|(path, _)| GpuVideoReader::open(path, 0, device.clone()))
    .transpose()?;
  let mut camera_current = None;
  let mut camera_next = match camera_reader.as_mut() {
    Some(reader) => reader.next_frame()?,
    None => None,
  };
  let mut cursor = request
    .cursor
    .map(|path| CursorCompositor::open(path, request.keyboard))
    .transpose()?;
  let keyboard = request
    .keyboard
    .map(|path| {
      screenwide_keyboard_effects::KeyboardCompositor::open_with_deleted(
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
  let camera_geometry = request
    .camera
    .map(|(_, options)| {
      media_preview::bake_geometry(options)
        .map(|geometry| export_output::on_canvas(geometry, request.output))
    })
    .transpose()?;
  let mut camera_effects = crate::cursor_export::camera_effects::FrameEffects::new(
    request.camera_effects.as_ref(),
    compositor.gpu(),
  )?;
  let sink = VideoSink::new(
    path,
    device,
    output_size.0,
    output_size.1,
    super::super::super::video_bitrate(output_size.0, output_size.1, request.video.compression),
  )?;
  let mut current = reader
    .next_frame()?
    .ok_or_else(|| "Media Foundation returned no frame for export".to_owned())?;
  let timeline_origin = current.timestamp_100ns;
  let export_end_100ns = i64::try_from(request.duration_ms)
    .unwrap_or(i64::MAX / 10_000)
    .saturating_mul(10_000);
  let clips = super::super::super::frame_annotations::ExportAnnotations::for_request(
    request,
    stretch.map(media_preview::PreviewStretch::source_span_ms),
  );
  let scenes = super::super::super::frame_scene::ExportScenes::for_request(request);
  let mut progress_pace = super::super::super::progress_pace::ProgressPace::default();
  // A reordered timeline plays an earlier stretch of the source after a
  // later one. The readers only move forward, so they are taken back to the
  // start of each such piece in turn.
  let pieces = match stretch {
    Some(stretch) => screenwide_timeline::ascending_source_pieces(&stretch.ranges),
    None => request.timeline.map_or_else(Vec::new, |timeline| {
      screenwide_timeline::ascending_source_pieces(timeline.ranges())
    }),
  };
  let sample_origin_us = stretch.map_or(0, |stretch| stretch.start_us);
  let origin_ms = u64::try_from(timeline_origin.max(0) / 10_000).unwrap_or_default();
  // A stretch starts anywhere in the recording, which the readers reach by
  // seeking rather than by decoding everything before it.
  if let (Some(_), Some(&[start_us, _])) = (stretch, pieces.first()) {
    let seek_ms = origin_ms.saturating_add(start_us / 1_000);
    reader.seek(seek_ms, false)?;
    current = reader
      .next_frame()?
      .ok_or_else(|| "Media Foundation returned no frame for the preview".to_owned())?;
    if let Some(camera) = camera_reader.as_mut() {
      camera.seek(seek_ms, false)?;
      camera_current = None;
      camera_next = camera.next_frame()?;
    }
  }
  let mut piece = 0;
  let mut frames = 0_u64;
  loop {
    if request.cancelled.load(Ordering::Acquire) {
      let _ = std::fs::remove_file(path);
      return Ok((ExportRunResult::Cancelled, frames));
    }
    let next = reader.next_frame()?;
    let pts_100ns = current.timestamp_100ns.saturating_sub(timeline_origin);
    let next_pts_100ns = next.as_ref().map_or(export_end_100ns, |frame| {
      frame.timestamp_100ns.saturating_sub(timeline_origin)
    });
    let position_ms = u64::try_from(pts_100ns.max(0) / 10_000).unwrap_or_default();
    let source_us = u64::try_from(pts_100ns.max(0) / 10).unwrap_or_default();
    if let Some(&[start_us, end_us]) = pieces.get(piece) {
      if source_us >= end_us {
        piece += 1;
        let Some(&[next_start_us, _]) = pieces.get(piece) else {
          break;
        };
        let seek_ms = origin_ms.saturating_add(next_start_us / 1_000);
        reader.seek(seek_ms, false)?;
        let Some(first) = reader.next_frame()? else {
          break;
        };
        current = first;
        if let Some(camera) = camera_reader.as_mut() {
          camera.seek(seek_ms, false)?;
          camera_current = None;
          camera_next = camera.next_frame()?;
        }
        continue;
      }
      // Read on to where this piece starts: anything before it plays
      // elsewhere on the timeline, or not at all.
      if source_us < start_us {
        let Some(next) = next else {
          break;
        };
        current = next;
        continue;
      }
    }
    let output_us = request.timeline.map_or(Some(source_us), |timeline| {
      timeline.source_to_output_us(source_us)
    });
    let Some(output_us) = output_us else {
      let Some(next) = next else {
        break;
      };
      current = next;
      continue;
    };
    while camera_next
      .as_ref()
      .is_some_and(|frame| frame.timestamp_100ns.saturating_sub(timeline_origin) <= pts_100ns)
    {
      camera_current = camera_next.take();
      camera_next = match camera_reader.as_mut() {
        Some(reader) => reader.next_frame()?,
        None => None,
      };
    }
    // How much source time this frame covers, which is the window a moving
    // annotation, and a scene's moving panes, smear over.
    let window_ms = next_pts_100ns.saturating_sub(pts_100ns).max(0) as f32 / 10_000.0;
    // A frame inside a scene is drawn on its own arrangement; every other
    // frame keeps the request's composition and the camera placed once.
    let arranged = scenes.at(request.output, position_ms, window_ms);
    let output = arranged
      .as_ref()
      .map_or(request.output, |(output, _)| output);
    let geometry = match &arranged {
      Some((_, Some(options))) => Some(media_preview::bake_geometry(*options)?),
      _ => camera_geometry,
    };
    let baked_cursor = cursor.as_mut().and_then(|cursor| {
      cursor.gpu_cursor(
        position_ms,
        (current.width, current.height),
        request.cursor_effects,
      )
    });
    let camera_frame = camera_current.as_ref().and_then(|frame| {
      request.camera.map(|(_, options)| {
        (
          &frame.texture,
          frame.subresource,
          geometry.expect("camera geometry exists with camera options"),
          options.camera_drop_shadow,
          request.camera_on_top,
        )
      })
    });
    let annotations = clips.with_camera_at(
      position_ms,
      window_ms,
      output,
      arranged.as_ref().and_then(|(_, options)| *options),
    );
    // The redactions and highlights drawn on the camera change its own
    // pixels, so they are composed into its frame, decoded at its own size.
    let camera_composition = annotations
      .camera
      .as_ref()
      .zip(request.camera)
      .map(|((camera, source), (_, options))| {
        screenwide_compositor::compositor::CameraComposition::new(
          camera,
          *source,
          (options.camera_width, options.camera_height),
        )
      })
      .transpose()?
      .flatten();
    let texture = compositor.compose_with_camera(
      &current.texture,
      current.subresource,
      output,
      ComposedFrame {
        cursor: baked_cursor,
        keyboard: keyboard.as_ref().and_then(|keyboard| {
          keyboard.evaluate_fitted_with_timeline(
            position_ms,
            request.keyboard_effects,
            (request.output.width, request.output.height),
            request.timeline,
          )
        }),
        foreground_only: false,
        seconds: position_ms as f64 / 1_000.0,
      },
      camera_frame,
      camera_composition.as_ref(),
      &annotations.screen,
      // Exposed over the frame's window, as its annotations are.
      request.captions.map(|track| {
        (
          track,
          screenwide_text::captions::CaptionMoment {
            at_ms: output_us as f64 / 1_000.0,
            exposure_ms: f64::from(window_ms),
          },
        )
      }),
      camera_effects.as_mut(),
    )?;
    sink.write(
      &texture,
      i64::try_from(output_us.saturating_sub(sample_origin_us))
        .unwrap_or(i64::MAX / 10)
        .saturating_mul(10),
      next_pts_100ns.saturating_sub(pts_100ns),
    )?;
    frames += 1;
    if progress_pace.due(std::time::Instant::now(), next.is_none()) {
      (request.on_progress)(
        output_us
          .div_ceil(1_000)
          .min(
            request
              .timeline
              .map_or(request.duration_ms, |timeline| timeline.duration_ms()),
          )
          .saturating_mul(GPU_PROGRESS_PERCENT)
          / 100,
      );
    }
    let Some(next) = next else {
      break;
    };
    current = next;
  }
  sink.finish()?;
  Ok((ExportRunResult::Completed, frames))
}
