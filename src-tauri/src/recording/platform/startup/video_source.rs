// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;
use crate::capture_geometry::{physical_capture_rect, video_capture_rect};
use crate::capture_kit::{desktop_layout, windows_to_exclude};
use crate::desktop_capture::{self, CapturePlan, DesktopDisplay, OutputLimits};
use crate::recording::cursor::{CursorSource, CursorSourceKind};
use crate::recording::Region;

pub(super) struct PrimaryVideo {
  pub cursor_source: CursorSource,
  pub filter: arc::R<sc::ContentFilter>,
  pub fps: u32,
  pub height: u32,
  pub is_window: bool,
  pub show_cursor: bool,
  pub source_rect: Option<cg::Rect>,
  pub source_scale_factor: f32,
  pub width: u32,
}

pub(super) struct DesktopVideo {
  pub cursor_source: CursorSource,
  pub displays: Vec<DesktopDisplay>,
  pub fps: u32,
  pub plan: CapturePlan,
  pub show_cursor: bool,
}

pub(super) enum ResolvedVideo {
  Direct(PrimaryVideo),
  Desktop(DesktopVideo),
}

impl ResolvedVideo {
  pub fn cursor_source(&self) -> &CursorSource {
    match self {
      Self::Direct(video) => &video.cursor_source,
      Self::Desktop(video) => &video.cursor_source,
    }
  }

  pub fn dimensions(&self) -> (u32, u32, u32) {
    match self {
      Self::Direct(video) => (video.width, video.height, video.fps),
      Self::Desktop(video) => (video.plan.width, video.plan.height, video.fps),
    }
  }

  pub fn source_scale_factor(&self) -> f32 {
    match self {
      Self::Direct(video) => video.source_scale_factor,
      Self::Desktop(video) => video.plan.output_scale as f32,
    }
  }

  pub fn can_capture_all_audio(&self) -> bool {
    !matches!(self, Self::Direct(video) if video.is_window)
  }
}

fn pixel_dimension(points: f64, scale: f32) -> Option<u32> {
  let pixels = points * f64::from(scale);
  if !pixels.is_finite() || pixels < 2.0 || pixels > f64::from(u32::MAX) {
    return None;
  }
  let pixels = pixels.round() as u32;
  let pixels = even(pixels);
  (pixels > 0).then_some(pixels)
}

fn window_cursor_source(
  window_id: u32,
  content_rect: cg::Rect,
  window_frame: cg::Rect,
  video_width: u32,
  video_height: u32,
) -> CursorSource {
  CursorSource {
    height: content_rect.size.height,
    kind: CursorSourceKind::Window,
    platform_id: window_id.to_string(),
    video_height,
    video_width,
    width: content_rect.size.width,
    // A desktop-independent filter reports a window-local content rect,
    // while CGEvent reports the pointer in global desktop coordinates.
    x: window_frame.origin.x,
    y: window_frame.origin.y,
  }
}

fn display_video(
  content: &sc::ShareableContent,
  fps: u32,
  include_own_windows: bool,
  monitor_id: u32,
  region: Option<Region>,
  show_cursor: bool,
) -> Result<PrimaryVideo, String> {
  let displays = content.displays();
  let display = displays
    .iter()
    .find(|display| display.display_id().0 == monitor_id)
    .ok_or_else(|| "The selected monitor is no longer available".to_owned())?;
  let (scale, monitor_width, monitor_height) = monitor_geometry(monitor_id)?;
  let monitor = xcap::Monitor::all()
    .map_err(|error| error.to_string())?
    .into_iter()
    .find(|monitor| monitor.id().ok() == Some(monitor_id))
    .ok_or_else(|| "The selected monitor is no longer available".to_owned())?;
  let monitor_x = f64::from(monitor.x().map_err(|error| error.to_string())?);
  let monitor_y = f64::from(monitor.y().map_err(|error| error.to_string())?);
  let source_rect = region
    .map(|region| {
      physical_capture_rect(region, scale, monitor_width, monitor_height)
        .and_then(video_capture_rect)
        .ok_or_else(|| "The selected region is too small or outside the monitor".to_owned())
    })
    .transpose()?;
  let (width, height) = source_rect
    .map(|rect| (rect.width, rect.height))
    .unwrap_or_else(|| (even(monitor_width), even(monitor_height)));
  let source_rect = source_rect.map(|rect| {
    cg::Rect::new(
      f64::from(rect.x) / scale,
      f64::from(rect.y) / scale,
      f64::from(rect.width) / scale,
      f64::from(rect.height) / scale,
    )
  });
  let logical_rect = source_rect.unwrap_or_else(|| {
    cg::Rect::new(
      0.0,
      0.0,
      f64::from(width) / scale,
      f64::from(height) / scale,
    )
  });
  let excluded = windows_to_exclude(content, include_own_windows);

  Ok(PrimaryVideo {
    cursor_source: CursorSource {
      height: logical_rect.size.height,
      kind: if region.is_some() {
        CursorSourceKind::Region
      } else {
        CursorSourceKind::Screen
      },
      platform_id: monitor_id.to_string(),
      video_height: height,
      video_width: width,
      width: logical_rect.size.width,
      x: monitor_x + logical_rect.origin.x,
      y: monitor_y + logical_rect.origin.y,
    },
    filter: sc::ContentFilter::with_display_excluding_windows(display, &excluded),
    fps,
    height,
    is_window: false,
    show_cursor,
    source_rect,
    source_scale_factor: scale as f32,
    width,
  })
}

fn region_video(
  content: &sc::ShareableContent,
  fps: u32,
  include_own_windows: bool,
  monitor_id: u32,
  region: Region,
  show_cursor: bool,
) -> Result<ResolvedVideo, String> {
  let displays = desktop_layout()?;
  let Some(plan) = composed_region_plan(&displays, monitor_id, region)? else {
    return display_video(
      content,
      fps,
      include_own_windows,
      monitor_id,
      Some(region),
      show_cursor,
    )
    .map(ResolvedVideo::Direct);
  };
  let desktop = plan.desktop_region;
  Ok(ResolvedVideo::Desktop(DesktopVideo {
    cursor_source: CursorSource {
      height: desktop.height,
      kind: CursorSourceKind::Region,
      platform_id: "desktop".to_owned(),
      video_height: plan.height,
      video_width: plan.width,
      width: desktop.width,
      x: desktop.x,
      y: desktop.y,
    },
    displays,
    fps,
    plan,
    show_cursor,
  }))
}

fn composed_region_plan(
  displays: &[DesktopDisplay],
  monitor_id: u32,
  region: Region,
) -> Result<Option<CapturePlan>, String> {
  let unbounded = desktop_capture::plan(displays, monitor_id, region, OutputLimits::UNBOUNDED)?;
  if unbounded.pieces.len() < 2 {
    return Ok(None);
  }
  desktop_capture::plan(displays, monitor_id, region, OutputLimits::VIDEO).map(Some)
}

fn window_video(
  content: &sc::ShareableContent,
  fps: u32,
  show_cursor: bool,
  window_id: u32,
) -> Result<PrimaryVideo, String> {
  let windows = content.windows();
  let window = windows
    .iter()
    .find(|window| window.id() == window_id)
    .ok_or_else(|| "The selected window is no longer available".to_owned())?;
  let filter = sc::ContentFilter::with_desktop_independent_window(window);
  let rect = filter.content_rect();
  let frame = window.frame();
  let scale = filter.point_pixel_scale();
  let width = pixel_dimension(rect.size.width, scale)
    .ok_or_else(|| "The selected window has no usable width".to_owned())?;
  let height = pixel_dimension(rect.size.height, scale)
    .ok_or_else(|| "The selected window has no usable height".to_owned())?;

  Ok(PrimaryVideo {
    cursor_source: window_cursor_source(window_id, rect, frame, width, height),
    filter,
    fps,
    height,
    is_window: true,
    show_cursor,
    source_rect: None,
    source_scale_factor: scale,
    width,
  })
}

pub(super) fn resolve(
  content: &sc::ShareableContent,
  include_own_windows: bool,
  primary: &PrimaryCaptureSource,
) -> Result<Option<ResolvedVideo>, String> {
  match primary {
    PrimaryCaptureSource::Screen {
      fps,
      monitor_id,
      show_cursor,
    } => display_video(
      content,
      *fps,
      include_own_windows,
      *monitor_id,
      None,
      *show_cursor,
    )
    .map(ResolvedVideo::Direct)
    .map(Some),
    PrimaryCaptureSource::Region {
      fps,
      monitor_id,
      region,
      show_cursor,
    } => region_video(
      content,
      *fps,
      include_own_windows,
      *monitor_id,
      *region,
      *show_cursor,
    )
    .map(Some),
    PrimaryCaptureSource::Window {
      fps,
      show_cursor,
      window_id,
    } => window_video(content, *fps, *show_cursor, *window_id)
      .map(ResolvedVideo::Direct)
      .map(Some),
    PrimaryCaptureSource::Camera | PrimaryCaptureSource::Audio => Ok(None),
  }
}

#[cfg(test)]
mod tests;
