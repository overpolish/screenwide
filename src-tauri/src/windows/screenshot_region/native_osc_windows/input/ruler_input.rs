// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Port of `processRulerViewportInput` (`+input.m:381-398`) with the anchor
/// already in this surface's client pixels.
pub(super) fn ruler_viewport(hwnd: HWND, operation: u32, x: f64, y: f64, delta: Point) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let anchor = {
    let Ok(mut set) = context.surfaces.lock() else {
      return false;
    };
    let Some(surface) = set.find_mut(hwnd) else {
      return false;
    };
    if !surface.input_enabled {
      return false;
    }
    (surface.display_id, surface.logical_point(x, y))
  };
  let result = context.ruler_viewport_input(anchor.0, operation, anchor.1, delta);
  if result.status == STATUS_INVALID {
    return false;
  }
  if let Ok(mut set) = context.surfaces.lock() {
    if let Some(surface) = set.find_mut(hwnd) {
      surface.claim_pointer();
    }
  }
  state::apply_ruler_result(&context, &result);
  apply_result_cursor(&context, hwnd, &result);
  true
}

/// The wheel reports screen coordinates, unlike every other pointer message.
pub(super) fn ruler_viewport_screen(
  hwnd: HWND,
  operation: u32,
  screen_x: i32,
  screen_y: i32,
  delta: Point,
) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  let client = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .find_mut(hwnd)
      .map(|s| s.screen_to_client(screen_x, screen_y))
  });
  let Some((x, y)) = client else {
    return false;
  };
  ruler_viewport(hwnd, operation, x, y, delta)
}

pub(super) fn ruler_pan_begin(hwnd: HWND, x: f64, y: f64) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let point = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .find_mut(hwnd)
      .filter(|surface| surface.input_enabled)
      .map(|surface| surface.logical_point(x, y))
  });
  let Some(point) = point else {
    return false;
  };
  let started = context
    .ruler
    .lock()
    .map(|mut session| {
      session.pan_last = Some(point);
    })
    .is_ok();
  started
}

pub(super) fn ruler_pan_drag(hwnd: HWND, x: f64, y: f64) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let Some(last) = context
    .ruler
    .lock()
    .ok()
    .and_then(|session| session.pan_last)
  else {
    return false;
  };
  let point = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .find_mut(hwnd)
      .map(|surface| surface.logical_point(x, y))
  });
  let Some(point) = point else {
    return false;
  };
  if let Ok(mut session) = context.ruler.lock() {
    session.pan_last = Some(point);
  }
  ruler_viewport(
    hwnd,
    2,
    x,
    y,
    Point {
      x: point.x - last.x,
      y: point.y - last.y,
    },
  )
}

pub(super) fn ruler_pan_end(hwnd: HWND) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  context
    .ruler
    .lock()
    .map(|mut session| session.pan_last.take().is_some())
    .unwrap_or(false)
}

/// Port of `processRulerLabelRightClick` (`+input.m:321-344`): a label under
/// the pointer is hidden, an empty spot toggles the labels there.
pub(super) fn ruler_right_click(hwnd: HWND, x: f64, y: f64) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let located = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .find_mut(hwnd)
      .filter(|surface| surface.input_enabled && surface.ruler.visible)
      .map(|surface| {
        let point = surface.logical_point(x, y);
        (surface.ruler.label_hit(point), surface.desktop_point(point))
      })
  });
  let Some((hit, desktop_point)) = located else {
    return false;
  };
  let (operation, kind, id) = hit.map_or((6, 0, 0), |hit| (5, hit.kind, hit.id));
  let result = context.ruler_label_input(operation, kind, id, desktop_point, Point::default());
  if result.status == STATUS_INVALID {
    return false;
  }
  state::apply_ruler_result(&context, &result);
  apply_result_cursor(&context, hwnd, &result);
  true
}

/// Steps 4 and 5 of `processInput` (`+input.m:193-239`): an active label drag
/// owns every drag and up, then a hover or press over a label rectangle is a
/// label interaction rather than a region gesture. `true` means consumed.
pub(super) fn ruler_label_step(
  context: &Context,
  hwnd: HWND,
  phase: u32,
  point: Point,
  desktop_point: Point,
) -> bool {
  if !context.is_ruler() {
    return false;
  }
  let dragging = context
    .ruler
    .lock()
    .map(|session| session.label_drag_active)
    .unwrap_or(false);
  if dragging && (phase == PHASE_DRAG || phase == PHASE_UP) {
    let result = context.ruler_label_input(
      if phase == PHASE_DRAG { 2 } else { 3 },
      0,
      0,
      desktop_point,
      Point::default(),
    );
    if phase == PHASE_UP {
      if let Ok(mut session) = context.ruler.lock() {
        session.label_drag_active = false;
      }
    }
    if result.status != STATUS_INVALID {
      state::apply_ruler_result(context, &result);
      apply_result_cursor(context, hwnd, &result);
    }
    return true;
  }
  if phase != PHASE_MOVE && phase != PHASE_DOWN {
    return false;
  }
  // A guide is being placed with a held key: the pointer belongs to it.
  if context
    .ruler
    .lock()
    .map(|session| session.guide_key != 0)
    .unwrap_or(false)
  {
    return false;
  }
  let located = context.surfaces.lock().ok().and_then(|mut set| {
    set.find_mut(hwnd).and_then(|surface| {
      surface
        .ruler
        .visible
        .then(|| surface.ruler.label_hit(point))
        .flatten()
        .map(|hit| (hit, surface.desktop_point(hit.center)))
    })
  });
  let Some((hit, center)) = located else {
    return false;
  };
  let result = if phase == PHASE_DOWN {
    let begin = context.ruler_label_input(1, hit.kind, hit.id, desktop_point, center);
    if begin.status != STATUS_INVALID {
      if let Ok(mut session) = context.ruler.lock() {
        session.label_drag_active = true;
      }
    }
    begin
  } else {
    context.ruler_label_input(7, hit.kind, hit.id, desktop_point, Point::default())
  };
  if result.status != STATUS_INVALID {
    state::apply_ruler_result(context, &result);
    apply_result_cursor(context, hwnd, &result);
  }
  true
}
