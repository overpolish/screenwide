// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) struct ResolvedSource {
  pub(super) width: u32,
  pub(super) height: u32,
  pub(super) fps: u32,
  pub(super) primary_kind: crate::recording::encoding::PrimaryRecordingKind,
  pub(super) source_scale_factor: f32,
  pub(super) cursor_source: Option<CursorSource>,
  pub(super) wall_timestamped_frames: bool,
  pub(super) graphics_source: Option<(CaptureTarget, u32, u32, bool)>,
  pub(super) desktop_plan: Option<(CapturePlan, bool)>,
  pub(super) source_crop: Option<CaptureRect>,
}

pub(super) fn resolve_source(
  primary: PrimaryCaptureSource,
  camera_spec: Option<&camera::CameraSpec>,
) -> Result<ResolvedSource, String> {
  let mut graphics_source = None;
  let mut desktop_plan = None;
  let mut source_crop: Option<CaptureRect> = None;
  let (
    width,
    height,
    fps,
    primary_kind,
    source_scale_factor,
    cursor_source,
    wall_timestamped_frames,
  ) = match primary {
    PrimaryCaptureSource::Screen {
      fps,
      monitor_id,
      show_cursor,
    } => {
      let monitor = xcap::Monitor::all()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|monitor| monitor.id().ok() == Some(monitor_id))
        .ok_or_else(|| "The selected monitor is no longer available".to_owned())?;
      let source_scale_factor = monitor.scale_factor().map_err(|error| error.to_string())?;
      let monitor_x = monitor.x().map_err(|error| error.to_string())?;
      let monitor_y = monitor.y().map_err(|error| error.to_string())?;
      let width = monitor.width().map_err(|error| error.to_string())? & !1;
      let height = monitor.height().map_err(|error| error.to_string())? & !1;
      if width < 2 || height < 2 {
        return Err("The selected monitor has no recordable area".to_owned());
      }
      graphics_source = Some((
        CaptureTarget::Monitor(monitor_id),
        width,
        height,
        show_cursor,
      ));
      (
        width,
        height,
        fps,
        super::super::encoding::PrimaryRecordingKind::Screen,
        source_scale_factor,
        Some(CursorSource {
          height: f64::from(height),
          kind: CursorSourceKind::Screen,
          platform_id: monitor_id.to_string(),
          video_height: height,
          video_width: width,
          width: f64::from(width),
          x: f64::from(monitor_x),
          y: f64::from(monitor_y),
        }),
        false,
      )
    }
    PrimaryCaptureSource::Region {
      fps,
      monitor_id,
      region,
      show_cursor,
    } => {
      let monitors = xcap::Monitor::all().map_err(|error| error.to_string())?;
      let monitor = monitors
        .iter()
        .find(|monitor| monitor.id().ok() == Some(monitor_id))
        .ok_or_else(|| "The selected monitor is no longer available".to_owned())?;
      if let Some(plan) = composed_region_plan(&monitors, monitor_id, region)? {
        let desktop = plan.desktop_region;
        let result = (
          plan.width,
          plan.height,
          fps,
          super::super::encoding::PrimaryRecordingKind::Screen,
          plan.output_scale as f32,
          Some(CursorSource {
            height: desktop.height,
            kind: CursorSourceKind::Region,
            platform_id: "desktop".to_owned(),
            video_height: plan.height,
            video_width: plan.width,
            width: desktop.width,
            x: desktop.x,
            y: desktop.y,
          }),
          true,
        );
        desktop_plan = Some((plan, show_cursor));
        result
      } else {
        let source_scale_factor = monitor.scale_factor().map_err(|error| error.to_string())?;
        let monitor_x = monitor.x().map_err(|error| error.to_string())?;
        let monitor_y = monitor.y().map_err(|error| error.to_string())?;
        let monitor_width = monitor.width().map_err(|error| error.to_string())?;
        let monitor_height = monitor.height().map_err(|error| error.to_string())?;
        let crop = physical_capture_rect(
          region,
          f64::from(source_scale_factor),
          monitor_width,
          monitor_height,
        )
        .and_then(video_capture_rect)
        .ok_or_else(|| "The selected region is too small or outside the monitor".to_owned())?;
        source_crop = Some(crop);
        graphics_source = Some((
          CaptureTarget::Monitor(monitor_id),
          monitor_width,
          monitor_height,
          show_cursor,
        ));
        (
          crop.width,
          crop.height,
          fps,
          super::super::encoding::PrimaryRecordingKind::Screen,
          source_scale_factor,
          Some(CursorSource {
            height: f64::from(crop.height),
            kind: CursorSourceKind::Region,
            platform_id: monitor_id.to_string(),
            video_height: crop.height,
            video_width: crop.width,
            width: f64::from(crop.width),
            x: f64::from(monitor_x) + f64::from(crop.x),
            y: f64::from(monitor_y) + f64::from(crop.y),
          }),
          true,
        )
      }
    }
    PrimaryCaptureSource::Window {
      fps,
      show_cursor,
      window_id,
    } => {
      let window = xcap::Window::all()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|window| window.id().ok() == Some(window_id))
        .ok_or_else(|| "The selected window is no longer available".to_owned())?;
      if window.is_minimized().unwrap_or(true) {
        return Err("The selected window is minimized".to_owned());
      }
      let target = CaptureTarget::Window(window_id);
      let (width, height) = capture::target_size(target)?;
      let window_x = window.x().map_err(|error| error.to_string())?;
      let window_y = window.y().map_err(|error| error.to_string())?;
      let window_width = window.width().map_err(|error| error.to_string())?;
      let window_height = window.height().map_err(|error| error.to_string())?;
      let source_scale_factor = window
        .current_monitor()
        .and_then(|monitor| monitor.scale_factor())
        .unwrap_or(1.0);
      graphics_source = Some((target, width, height, show_cursor));
      (
        width,
        height,
        fps,
        super::super::encoding::PrimaryRecordingKind::Screen,
        source_scale_factor,
        Some(CursorSource {
          height: f64::from(window_height),
          kind: CursorSourceKind::Window,
          platform_id: window_id.to_string(),
          video_height: height,
          video_width: width,
          width: f64::from(window_width),
          x: f64::from(window_x),
          y: f64::from(window_y),
        }),
        true,
      )
    }
    PrimaryCaptureSource::Camera => {
      let camera = camera_spec.as_ref().expect("checked above");
      (
        camera.width,
        camera.height,
        camera.fps,
        super::super::encoding::PrimaryRecordingKind::Camera,
        1.0,
        None,
        false,
      )
    }
    _ => return Err("This recording source is not yet available on Windows".to_owned()),
  };

  Ok(ResolvedSource {
    width,
    height,
    fps,
    primary_kind,
    source_scale_factor,
    cursor_source,
    wall_timestamped_frames,
    graphics_source,
    desktop_plan,
    source_crop,
  })
}

pub(super) fn desktop_layout(monitors: &[xcap::Monitor]) -> Result<Vec<DesktopDisplay>, String> {
  monitors
    .iter()
    .map(|monitor| {
      let scale = f64::from(monitor.scale_factor().map_err(|error| error.to_string())?);
      if !scale.is_finite() || scale <= 0.0 {
        return Err("Windows returned an invalid monitor scale".to_owned());
      }
      Ok(DesktopDisplay {
        id: monitor.id().map_err(|error| error.to_string())?,
        x: f64::from(monitor.x().map_err(|error| error.to_string())?) / scale,
        y: f64::from(monitor.y().map_err(|error| error.to_string())?) / scale,
        width: f64::from(monitor.width().map_err(|error| error.to_string())?) / scale,
        height: f64::from(monitor.height().map_err(|error| error.to_string())?) / scale,
        scale,
      })
    })
    .collect()
}

pub(super) fn composed_region_plan(
  monitors: &[xcap::Monitor],
  monitor_id: u32,
  region: crate::recording::Region,
) -> Result<Option<CapturePlan>, String> {
  let displays = desktop_layout(monitors)?;
  let unbounded = desktop_capture::plan(&displays, monitor_id, region, OutputLimits::UNBOUNDED)?;
  if unbounded.pieces.len() < 2 {
    return Ok(None);
  }
  desktop_capture::plan(&displays, monitor_id, region, OutputLimits::VIDEO).map(Some)
}
