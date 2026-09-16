// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn render_video(
  request: &mut CursorExportRequest<'_>,
  path: &Path,
) -> Result<ExportRunResult, String> {
  crate::screenshots::validate_output_settings(request.width, request.height, request.output)?;
  // The recording workspace's own window owns the GPU device this export
  // composites on; the screenshot window has a separate one.
  let surface = RecordingPreviewSurface::existing_for(crate::editor::EditorKind::Recording)?;
  let mut reader = GpuVideoReader::open(request.screen, 0, surface.clone())?;
  let mut camera_reader = request
    .camera
    .map(|(path, _)| GpuVideoReader::open(path, 0, surface.clone()))
    .transpose()?;
  let mut camera_current = None;
  let mut camera_next = match camera_reader.as_mut() {
    Some(reader) => reader.next_frame()?,
    None => None,
  };
  let mut cursor = request.cursor.map(CursorCompositor::open).transpose()?;
  let keyboard = request
    .keyboard
    .map(|path| {
      let compositor = crate::editor::keyboard_effects::KeyboardCompositor::open_with_deleted(
        path,
        request
          .timeline
          .map_or(&[], |timeline| timeline.deleted_keyboard_shortcut_ids()),
        request
          .timeline
          .map_or(&[], |timeline| timeline.deleted_keyboard_shortcut_ranges()),
      )?;
      if let Some(timeline) = request.timeline {
        compositor.set_shortcut_positions(timeline.keyboard_shortcut_positions());
      }
      Ok::<_, String>(compositor)
    })
    .transpose()?;
  let output_size = crate::screenshots::output_dimensions(request.output)?;
  let compositor = request.camera.map_or_else(
    || surface.export_compositor((request.width, request.height), output_size),
    |(_, options)| {
      surface.export_compositor_with_camera(
        (request.width, request.height),
        (options.camera_width, options.camera_height),
        output_size,
      )
    },
  )?;
  let camera_geometry = request
    .camera
    .map(|(_, options)| media_preview::bake_geometry(options))
    .transpose()?;
  let sink = VideoSink::new(
    path,
    &surface.device(),
    output_size.0,
    output_size.1,
    super::super::video_bitrate(output_size.0, output_size.1, request.video.compression),
  )?;
  let mut current = reader
    .next_frame()?
    .ok_or_else(|| "Media Foundation returned no frame for export".to_owned())?;
  let timeline_origin = current.timestamp_100ns;
  let export_end_100ns = i64::try_from(request.duration_ms)
    .unwrap_or(i64::MAX / 10_000)
    .saturating_mul(10_000);
  // Marks are authored against the source at its own scale while the stroke
  // follows the output, exactly as `timed_annotations::for_request` prepares
  // them for the Metal export. Scaled once rather than per frame.
  let annotation_clips: Vec<_> = request
    .timeline
    .map_or(&[][..], |timeline| timeline.annotation_clips())
    .iter()
    .map(|clip| {
      let mut clip = clip.clone();
      clip.annotation.style.width *= f64::from(request.video.resolution_scale_percent)
        / f64::from(request.video.source_scale_percent.max(1));
      clip
    })
    .collect();
  loop {
    if request.cancelled.load(Ordering::Acquire) {
      let _ = std::fs::remove_file(path);
      return Ok(ExportRunResult::Cancelled);
    }
    let next = reader.next_frame()?;
    let pts_100ns = current.timestamp_100ns.saturating_sub(timeline_origin);
    let next_pts_100ns = next.as_ref().map_or(export_end_100ns, |frame| {
      frame.timestamp_100ns.saturating_sub(timeline_origin)
    });
    let position_ms = u64::try_from(pts_100ns.max(0) / 10_000).unwrap_or_default();
    let source_us = u64::try_from(pts_100ns.max(0) / 10).unwrap_or_default();
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
          camera_geometry.expect("camera geometry exists with camera options"),
          options.camera_drop_shadow,
          request.camera_on_top,
        )
      })
    });
    let texture = compositor.compose_with_camera(
      &current.texture,
      current.subresource,
      request.output,
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
      &crate::editor::annotations::timing::revealed_annotations(
        &annotation_clips,
        request.annotation_track,
        position_ms,
        // How much source time this frame covers, which is the window a
        // moving mark smears over.
        next_pts_100ns.saturating_sub(pts_100ns).max(0) as f32 / 10_000.0,
      ),
    )?;
    sink.write(
      &texture,
      i64::try_from(output_us)
        .unwrap_or(i64::MAX / 10)
        .saturating_mul(10),
      next_pts_100ns.saturating_sub(pts_100ns),
    )?;
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
    let Some(next) = next else {
      break;
    };
    current = next;
  }
  sink.finish()?;
  Ok(ExportRunResult::Completed)
}
