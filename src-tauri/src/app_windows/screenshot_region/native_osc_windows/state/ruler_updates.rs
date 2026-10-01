// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Port of the eight `native_osc_ruler_*` pulls (`native_osc_macos/state.rs:48-285`).
/// macOS needed a count-then-fill C convention; here the document hands back
/// vectors, so one pass produces the whole draw set.
pub(super) fn pull_ruler(context: &Context) -> Option<RulerData> {
  if context.runtime.purpose != Purpose::Ruler {
    return None;
  }
  let app = context.runtime.window.app_handle();
  let state = app.state::<crate::ruler::RulerState>();
  let (centerlines, inner_objects) = state.center_aids();
  Some(RulerData {
    measurements: state.measurements().iter().map(Into::into).collect(),
    viewports: state.viewports().iter().map(Into::into).collect(),
    probes: state.probes().iter().map(Into::into).collect(),
    guides: state.guides().iter().map(Into::into).collect(),
    guide_gaps: state.guide_gaps().iter().map(Into::into).collect(),
    radii: state.radii().iter().map(Into::into).collect(),
    centerlines: centerlines.iter().map(Into::into).collect(),
    inner_objects: inner_objects.iter().map(Into::into).collect(),
  })
}

/// Port of `screenwide_region_osc_apply_ruler_result` (`+ruler.m:919-1071`):
/// pull every dataset, mirror the flags onto each surface, diff, re-assign the
/// label pools and redraw.
pub(crate) fn apply_ruler_result(context: &Context, result: &OscResult) -> bool {
  if result.ruler_flags & 1 == 0 {
    return false;
  }
  let Some(data) = pull_ruler(context) else {
    return false;
  };
  let now = Instant::now();
  let crosshair = result.ruler_flags & 2 != 0;
  let copied = result.ruler_flags & 4 != 0;
  let interaction_active = result.ruler_flags & 64 != 0;
  let tolerance_requested = result.ruler_flags & 8 != 0;
  let tolerance_mode = (result.ruler_flags >> 4) & 3;
  let (hover_key, hover_opacity) = ruler::hovered_artifact_key(&data);
  let animating = ruler::animation_active(&data, result.ruler_flags);
  let Ok(mut set) = context.surfaces.lock() else {
    return false;
  };
  let root = set.root_mut();
  let tolerance_started = tolerance_requested
    && (!root.ruler.tolerance_visible || root.ruler.tolerance_mode != tolerance_mode);
  let hover_changed = root.ruler.hovered_artifact_key() != hover_key;
  let hover_started = if hover_changed {
    now
  } else {
    root.ruler.hover_started()
  };
  let hwnd = set.root_hwnd();

  let mut world = Vec::new();
  for surface in set.all_mut() {
    let viewport = data
      .viewports
      .iter()
      .find(|viewport| viewport.display_id == surface.display_id);
    let zoom = viewport.map_or(1.0, |viewport| viewport.zoom);
    let origin = viewport.map_or_else(Point::default, |viewport| Point {
      x: viewport.origin_x,
      y: viewport.origin_y,
    });
    let viewport_changed =
      surface.ruler.viewport_zoom != zoom || surface.ruler.viewport_origin != origin;
    surface.ruler.viewport_zoom = zoom;
    surface.ruler.viewport_origin = origin;
    surface.ruler.visible = true;
    surface.ruler.crosshair = crosshair;
    surface.ruler.interaction_active = interaction_active;
    surface.ruler.color = result.ruler_color;
    let offset = surface.desktop_offset();
    surface.ruler.point = Point {
      x: result.x - offset.x,
      y: result.y - offset.y,
    };
    if tolerance_requested {
      surface.ruler.tolerance_mode = tolerance_mode;
    }
    surface.ruler.tolerance_visible = tolerance_requested;
    surface
      .ruler
      .set_tolerance(tolerance_requested, tolerance_started, now);
    surface
      .ruler
      .set_hover(hover_key, hover_opacity, hover_started);
    surface.ruler.replace_data(&data, viewport_changed);
    surface.ruler.set_copied(copied, now);
    let bounds = surface.logical_size();
    world.push(surface.ruler.visible_world_rect(offset, bounds));
  }
  // Which display owns a label can only be answered with every viewport in
  // hand, so the four pools are assigned here rather than per surface.
  let owned = ruler::assign_labels(&world, &data);
  for (index, surface) in set.all_mut().enumerate() {
    surface
      .ruler
      .set_labels(owned.get(index).cloned().unwrap_or_default());
    // The loupe follows the pointer inside the one swap chain, so every ruler
    // result redraws; macOS could skip the world pass because its readout was
    // a separate layer.
    surface.draw();
  }
  drop(set);

  if animating {
    let _ = unsafe { SetTimer(Some(hwnd), input::RULER_SETTLE_TIMER, 16, None) };
  }
  if copied {
    let _ = unsafe {
      SetTimer(
        Some(hwnd),
        input::RULER_COPIED_TIMER,
        ruler::EXPIRY.as_millis() as u32,
        None,
      )
    };
  }
  if tolerance_started {
    let _ = unsafe {
      SetTimer(
        Some(hwnd),
        input::RULER_TOLERANCE_TIMER,
        ruler::EXPIRY.as_millis() as u32,
        None,
      )
    };
  }
  true
}

/// The 16ms settle frame macOS scheduled with `dispatch_after`
/// (`schedule_settle_frame`, `+ruler.m:836-849`).
pub(crate) fn ruler_settle_frame(context: &Context) {
  let live = with_set(context, |set| {
    set.root_mut().input_enabled && set.root_mut().visible
  })
  .unwrap_or(false);
  if !live {
    return;
  }
  let result = dispatch_input(context, 15, Point::default(), 0);
  if result.status != 255 {
    apply_ruler_result(context, &result);
  }
}

/// The copied checkmark's 900ms expiry (`+ruler.m:1058-1070`).
pub(crate) fn ruler_expire_copied(context: &Context) {
  with_set(context, |set| {
    let now = Instant::now();
    for surface in set.all_mut() {
      surface.ruler.set_copied(false, now);
      surface.draw();
    }
  });
}

/// The tolerance notice's 900ms expiry (`+ruler.m:1044-1057`).
pub(crate) fn ruler_expire_tolerance(context: &Context) {
  with_set(context, |set| {
    let now = Instant::now();
    for surface in set.all_mut() {
      surface.ruler.tolerance_visible = false;
      surface.ruler.set_tolerance(false, false, now);
      surface.draw();
    }
  });
}

pub(super) fn with_set<T>(context: &Context, work: impl FnOnce(&mut SurfaceSet) -> T) -> Option<T> {
  context.surfaces.lock().ok().map(|mut set| work(&mut set))
}

/// Port of `screenwide_region_osc_ruler_refresh_pointer` (`+input.m:354-379`):
/// re-samples the pointer so the readout resumes without waiting for a move.
pub(crate) fn refresh_ruler_pointer(window: &WebviewWindow) -> bool {
  let Some(context) = context_arc(window) else {
    return false;
  };
  let Some(pointer) = surface::cursor_position() else {
    return false;
  };
  let target = with_set(&context, |set| {
    set
      .all_mut()
      .find(|surface| {
        surface.input_enabled && surface.visible && surface.contains_screen_point(pointer)
      })
      .map(|surface| {
        let (x, y) = surface.screen_to_client(pointer.x, pointer.y);
        surface.desktop_point(surface.logical_point(x, y))
      })
  })
  .flatten();
  let Some(point) = target else {
    return false;
  };
  let result = dispatch_input(&context, 1, point, 0);
  if result.status == 255 {
    return false;
  }
  apply_ruler_result(&context, &result)
}

/// Port of `screenwide_region_osc_ruler_set_transient_chrome`
/// (`+ruler.m:1121-1143`): hides the loupe and the live probes while a
/// screenshot is being taken through the frozen desktop.
pub(crate) fn set_ruler_transient_chrome(window: &WebviewWindow, visible: bool) -> bool {
  with_surfaces(window, |set| {
    for surface in set.all_mut() {
      surface.ruler.transient_chrome = visible;
      surface.draw();
    }
  })
  .is_some()
}
