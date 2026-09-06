// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Retained-workspace pane reflow, fit rebasing, and geometry publication.

use super::{
  draw_selection, rebase_display_fit_mode, set_pane_geometry, DisplayRect, FrameResizeStart,
  PreviewSurfaceRect, SurfaceInner, SurfaceState,
};

pub(super) fn union_rect(
  left: PreviewSurfaceRect,
  right: PreviewSurfaceRect,
) -> PreviewSurfaceRect {
  let x = left.x.min(right.x);
  let y = left.y.min(right.y);
  let right_edge = (left.x + left.width).max(right.x + right.width);
  let bottom = (left.y + left.height).max(right.y + right.height);
  PreviewSurfaceRect {
    x,
    y,
    width: right_edge - x,
    height: bottom - y,
  }
}

/// Re-flows the sibling panes around the one a Frame resize is dragging, the
/// Windows counterpart of `reflow_recording_workspace_panes`: the row keeps
/// its gesture-start gaps and side ordering while every pane stays centred on
/// the row, so growing one canvas pushes its neighbours instead of
/// overlapping them.
pub(super) fn reflow_workspace_panes(
  starts: &[(usize, PreviewSurfaceRect)],
  selected: usize,
  resized: PreviewSurfaceRect,
) -> Vec<(usize, PreviewSurfaceRect)> {
  let mut order = (0..starts.len()).collect::<Vec<_>>();
  order.sort_by(|left, right| {
    starts[*left]
      .1
      .x
      .partial_cmp(&starts[*right].1.x)
      .unwrap_or(std::cmp::Ordering::Equal)
  });
  let mut next = starts.to_vec();
  let Some(selected_position) = order
    .iter()
    .position(|position| starts[*position].0 == selected)
  else {
    return next;
  };
  next[order[selected_position]].1 = resized;
  let tallest = order.iter().fold(0.0_f64, |tallest, position| {
    tallest.max(next[*position].1.height)
  });
  let group_top = resized.y - (tallest - resized.height) / 2.0;
  for position in &order {
    next[*position].1.y = group_top + (tallest - next[*position].1.height) / 2.0;
  }
  for position in selected_position + 1..order.len() {
    let previous = order[position - 1];
    let index = order[position];
    let gap = starts[index].1.x - (starts[previous].1.x + starts[previous].1.width);
    next[index].1.x = next[previous].1.x + next[previous].1.width + gap;
  }
  for position in (0..selected_position).rev() {
    let index = order[position];
    let following = order[position + 1];
    let gap = starts[following].1.x - (starts[index].1.x + starts[index].1.width);
    next[index].1.x = next[following].1.x - gap - next[index].1.width;
  }
  next
}

/// Centres a composition of `content` aspect inside `rect`. A committed
/// canvas resize keeps arriving from the DOM at the session's fixed source
/// aspect for a layout or two; fitting rather than filling means the pane
/// shows the composed canvas whole instead of stretching it.
pub(super) fn aspect_fit_rect(rect: PreviewSurfaceRect, content: (u32, u32)) -> PreviewSurfaceRect {
  let content_width = f64::from(content.0.max(1));
  let content_height = f64::from(content.1.max(1));
  let first = content_width * rect.height;
  let second = content_height * rect.width;
  let scale = first.max(second).max(1.0);
  if rect.width <= 0.0 || rect.height <= 0.0 || (first - second).abs() / scale < 0.005 {
    return rect;
  }
  let fit = (rect.width / content_width).min(rect.height / content_height);
  let width = content_width * fit;
  let height = content_height * fit;
  PreviewSurfaceRect {
    x: rect.x + (rect.width - width) / 2.0,
    y: rect.y + (rect.height - height) / 2.0,
    width,
    height,
  }
}

/// Re-expresses the resized workspace against a fresh centred fit without
/// moving a single displayed pixel, mirroring
/// `rebase_recording_workspace_fit`. `start` supplies the gesture's immutable
/// transform, so `displayed` is where the panes actually are on screen; only
/// the fit-relative zoom/pan representation changes, which is what makes the
/// toolbar percentage follow the drag and the commit land without a jump.
pub(super) fn rebase_workspace_fit(state: &mut SurfaceState, start: &FrameResizeStart) {
  let active = state
    .panes
    .iter()
    .enumerate()
    .filter_map(|(index, pane)| {
      pane
        .as_ref()
        .filter(|pane| pane.seen)
        .map(|pane| (index, pane.base_rect))
    })
    .collect::<Vec<_>>();
  let Some((_, first)) = active.first().copied() else {
    return;
  };
  let bounds = active
    .iter()
    .skip(1)
    .fold(first, |bounds, (_, rect)| union_rect(bounds, *rect));
  let start_bounds = active
    .iter()
    .map(|(index, rect)| {
      start
        .pane_rects
        .iter()
        .find(|(start_index, _)| start_index == index)
        .map_or(*rect, |(_, start_rect)| *start_rect)
    })
    .reduce(union_rect)
    .unwrap_or(bounds);
  let first_display = start.transform.apply(state.viewport, first);
  let displayed = active
    .iter()
    .skip(1)
    .fold(first_display, |bounds, (_, rect)| {
      union_rect(bounds, start.transform.apply(state.viewport, *rect))
    });
  // Match Metal's live resize math: retain the fractional natural size for
  // the fit rebase, and only round the retained/output dimensions. Rounding
  // before the rebase can turn a sub-pixel drag into an observable zoom jump.
  let natural = start
    .natural_size
    .map_or((bounds.width, bounds.height), |(width, height)| {
      let natural_width = f64::from(width) * bounds.width / start_bounds.width.max(1.0);
      let natural_height = f64::from(height) * bounds.height / start_bounds.height.max(1.0);
      state.workspace_natural_size = Some((
        natural_width.round().max(1.0) as u32,
        natural_height.round().max(1.0) as u32,
      ));
      (natural_width, natural_height)
    });
  // Pane rects and `WorkspaceTransform::apply` are viewport-relative, so the
  // displayed union and the fit stay in that space; adding the viewport origin
  // here would shift every pane by it on each move.
  let rebased = rebase_display_fit_mode(
    (state.viewport.width, state.viewport.height),
    DisplayRect {
      x: displayed.x,
      y: displayed.y,
      width: displayed.width,
      height: displayed.height,
    },
    natural,
    8.0,
    state.workspace_allows_upscale,
  );
  let fit = PreviewSurfaceRect {
    x: rebased.fit.x,
    y: rebased.fit.y,
    width: rebased.fit.width,
    height: rebased.fit.height,
  };
  let scale_x = fit.width / bounds.width.max(1.0);
  let scale_y = fit.height / bounds.height.max(1.0);
  for (index, rect) in active {
    if let Some(pane) = state.panes.get_mut(index).and_then(Option::as_mut) {
      pane.base_rect = PreviewSurfaceRect {
        x: fit.x + (rect.x - bounds.x) * scale_x,
        y: fit.y + (rect.y - bounds.y) * scale_y,
        width: rect.width * scale_x,
        height: rect.height * scale_y,
      };
    }
  }
  state.workspace_transform.zoom = rebased.zoom;
  state.workspace_transform.pan_x = rebased.pan_x;
  state.workspace_transform.pan_y = rebased.pan_y;
}

/// Publishes the workspace transform to every pane. `defer_geometry` parks the
/// pane boxes instead of committing them: a canvas resize changes the box and
/// the composition together, and the re-composed still that follows this call
/// publishes the parked geometry with its own present, so the pane never shows
/// the previous canvas letterboxed into the new box for a frame. The selection
/// overlay is always redrawn immediately - it tracks the box, not the pixels.
pub(super) fn apply_workspace_transform(
  inner: &SurfaceInner,
  state: &mut SurfaceState,
  defer_geometry: bool,
) {
  let transform = state.workspace_transform;
  let viewport = state.viewport;
  let scale = state.scale;
  for pane in state.panes.iter_mut().flatten().filter(|pane| pane.seen) {
    let rect = transform.apply(viewport, pane.base_rect);
    set_pane_geometry(pane, viewport, rect, scale, defer_geometry);
  }
  draw_selection(inner, state);
  let _ = unsafe { inner.gpu.composition.Commit() };
}
