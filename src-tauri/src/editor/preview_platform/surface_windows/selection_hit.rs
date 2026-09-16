// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn radius_point(frame: PreviewSurfaceRect, radius_percent: f64) -> (f64, f64) {
  let offset =
    frame.width.min(frame.height) * radius_percent.clamp(0.0, 50.0) / 100.0 * 0.55 + 10.0;
  (frame.x + offset, frame.y + offset)
}

pub(super) fn shared_selection_hit(
  inner: &SurfaceInner,
  state: &SurfaceState,
  point: (f64, f64),
) -> Option<(PreviewSelection, u8)> {
  let mut selections = state.selection_targets.clone();
  if let Some(current) = state.selection {
    if let Some(target) = selections
      .iter_mut()
      .find(|target| target.pane_index == current.pane_index && target.layer_id == current.layer_id)
    {
      *target = current;
    } else {
      selections.push(current);
    }
  }
  let targets = selections
    .iter()
    .enumerate()
    .filter_map(|(index, selection)| {
      let selected = state.selection.is_some_and(|current| {
        current.pane_index == selection.pane_index && current.layer_id == selection.layer_id
      });
      let rect = if selected {
        display_selection(state, *selection)?
      } else {
        keyboard_hit::frame(inner, state, *selection)?
      };
      Some(DisplayTarget {
        id: (u64::from(selection.pane_index) << 32) | u64::from(selection.layer_id),
        rect: DisplayRect {
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
        },
        radius_enabled: u8::from(selection.crop_mode == 0 && selection.radius_disabled == 0),
        radius_percent: selection.radius_percent,
        z_order: index as i32,
        selected: u8::from(selected),
        visible: 1,
      })
    })
    .collect::<Vec<_>>();
  let hit = hit_test_display(&targets, point, 8.0)?;
  let pane_index = (hit.target_id >> 32) as u32;
  let layer_id = hit.target_id as u32;
  let selection = selections
    .into_iter()
    .find(|selection| selection.pane_index == pane_index && selection.layer_id == layer_id)?;
  if hit.handle == 0 && selection.layer_id == u32::MAX - 1 {
    let frame = keyboard_hit::frame(inner, state, selection)?;
    if point.0 < frame.x
      || point.0 > frame.x + frame.width
      || point.1 < frame.y
      || point.1 > frame.y + frame.height
    {
      return None;
    }
  }
  Some((selection, hit.handle))
}

pub(super) fn shared_handle_edges(handle: u8) -> u32 {
  match handle {
    1 => 4,
    2 => 8,
    3 => 2,
    4 => 1,
    5 => 2 | 4,
    6 => 1 | 4,
    7 => 2 | 8,
    8 => 1 | 8,
    _ => 0,
  }
}

pub(super) fn selection_pane_rect(
  state: &SurfaceState,
  selection: PreviewSelection,
) -> PreviewSurfaceRect {
  state
    .panes
    .get(selection.pane_index as usize)
    .and_then(Option::as_ref)
    .map_or(
      PreviewSurfaceRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
      },
      |pane| pane_canvas_rect(pane, state.frame_resize.is_some()),
    )
}

pub(super) fn clear_selection_snap_guides(state: &mut SurfaceState) {
  state.selection_snap_guide_x = None;
  state.selection_snap_guide_y = None;
}

pub(super) fn selection_snap_targets(
  state: &SurfaceState,
  start: PreviewSelection,
  horizontal: bool,
) -> Vec<(u32, f64, f64)> {
  let Some(start_pane) = state
    .panes
    .get(start.pane_index as usize)
    .and_then(Option::as_ref)
  else {
    return Vec::new();
  };
  let same_frame = |target: &PreviewSelection| {
    state
      .panes
      .get(target.pane_index as usize)
      .and_then(Option::as_ref)
      .is_some_and(|pane| {
        let first = start_pane.base_rect;
        let second = pane.base_rect;
        (first.x - second.x).abs() < 0.000_001
          && (first.y - second.y).abs() < 0.000_001
          && (first.width - second.width).abs() < 0.000_001
          && (first.height - second.height).abs() < 0.000_001
      })
  };
  state
    .selection_targets
    .iter()
    .filter(|target| same_frame(target))
    .map(|target| {
      if horizontal {
        (target.layer_id, target.x, target.width)
      } else {
        (target.layer_id, target.y, target.height)
      }
    })
    .collect()
}

pub(super) fn cursor_for_state(
  inner: &SurfaceInner,
  state: &SurfaceState,
  point: (f64, f64),
) -> editor::CursorKind {
  // The arrow tool speaks first, exactly as `annotation_cursor` does: an
  // arrow under the pointer is something to take hold of, and empty picture
  // is something to draw on. The select tool leaves empty picture to the
  // layer underneath.
  if let Some(kind) = annotation::cursor_for(state, point) {
    return kind;
  }
  let Some((selection, handle)) = shared_selection_hit(inner, state, point) else {
    return editor::CursorKind::Arrow;
  };
  if handle == 0 && selection.layer_id == FRAME_LAYER_ID {
    return editor::CursorKind::Arrow;
  }
  let edges = shared_handle_edges(handle);
  match handle {
    0 => editor::CursorKind::Move,
    9 => editor::CursorKind::ResizeNwse,
    _ if edges == 1 || edges == 2 => editor::CursorKind::ResizeHorizontal,
    _ if edges == 4 || edges == 8 => editor::CursorKind::ResizeVertical,
    _ if edges == (1 | 4) || edges == (2 | 8) => editor::CursorKind::ResizeNwse,
    _ => editor::CursorKind::ResizeNesw,
  }
}
