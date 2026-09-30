// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The presses that change the choice rather than one annotation: a carry of
//! several chosen together, a click with the toggle modifier, and a marquee
//! band. The twin of the group, toggle and marquee branches of
//! `recording_preview_surface_macos+annotation.m`. Like the rest of the
//! gesture, everything here resolves under the surface lock and hands its
//! samples back to be reported without it.

use super::drag::Press;
use super::sample::{press_toggles, resolve, resolve_layer, Sample};
use super::*;
use crate::editor::annotations::gesture::MODE_MARQUEE;
use crate::editor::annotations::handles::HANDLE_FLAG_GROUPED;

/// Arms a press on empty picture that the choice owns: a marquee band under
/// the marquee, or a carry of the group inside its box under the select tool.
/// `false` when the press is neither.
pub(super) fn arm_empty(state: &mut SurfaceState, point: (f64, f64)) -> bool {
  let toggles = press_toggles();
  if state.annotation.mode == MODE_MARQUEE {
    let Some(layer) = marquee_layer_at_point(state, point) else {
      return false;
    };
    state.annotation.drag = Some(
      Drag::pending(TARGET_MARQUEE, 0, HANDLE_BODY, point)
        .with_press(Press::Marquee { additive: toggles }, layer),
    );
    return true;
  }
  let Some(layer) = group_layer_at_point(state, point).filter(|_| !toggles) else {
    return false;
  };
  state.annotation.drag =
    Some(Drag::pending(TARGET_GROUP, u32::MAX, HANDLE_BODY, point).with_press(Press::Group, layer));
  true
}

/// Arms a press on the annotation at `shaft` that the choice owns: one of a
/// group carries the group, and the toggle modifier waits for the release to
/// add or remove it. `false` leaves the press to choose the annotation.
pub(super) fn arm_shaft(
  state: &mut SurfaceState,
  shaft: usize,
  handle: u32,
  point: (f64, f64),
) -> bool {
  let Some(item) = state.annotation.handles.get(shaft).copied() else {
    return false;
  };
  let drag = if press_toggles() {
    Drag::pending(TARGET_EXISTING, shaft as u32, handle, point).with_press(Press::Toggle, -1)
  } else if item.flags & HANDLE_FLAG_GROUPED != 0 {
    Drag::pending(TARGET_GROUP, shaft as u32, HANDLE_BODY, point)
      .with_press(Press::Group, item.layer_id)
  } else {
    return false;
  };
  state.annotation.drag = Some(drag);
  true
}

/// The drag a press becomes once it travels. A toggle press is an ordinary
/// press by then: one of a group carries the group, and any other annotation
/// is chosen, as the press would have chosen it, and then carried.
pub(super) fn travelled(state: &mut SurfaceState, drag: Drag, samples: &mut Vec<Sample>) -> Drag {
  if drag.press != Press::Toggle {
    return drag;
  }
  let index = drag.index as usize;
  match state.annotation.handles.get(index).copied() {
    Some(item) if item.flags & HANDLE_FLAG_GROUPED != 0 => drag
      .retargeted(TARGET_GROUP, HANDLE_BODY)
      .with_press(Press::Group, item.layer_id),
    _ => {
      samples.extend(super::choose(state, index, drag.origin));
      drag.with_press(Press::Ordinary, -1)
    }
  }
}

/// One sample of a drag that is measured on a layer rather than on one
/// annotation, or `None` for an ordinary drag. A marquee band is drawn from
/// here and only its two ends are reported.
pub(super) fn layer_sample(
  state: &mut SurfaceState,
  drag: Drag,
  phase: SelectionGesturePhase,
  point: (f64, f64),
) -> Option<Option<Sample>> {
  match drag.press {
    Press::Group => Some(resolve_layer(state, phase, TARGET_GROUP, drag.layer, point)),
    Press::Marquee { .. } => {
      state.annotation.marquee =
        (!matches!(phase, SelectionGesturePhase::End)).then(|| PreviewSurfaceRect {
          x: drag.origin.0.min(point.0),
          y: drag.origin.1.min(point.1),
          width: (point.0 - drag.origin.0).abs(),
          height: (point.1 - drag.origin.1).abs(),
        });
      Some(
        (!matches!(phase, SelectionGesturePhase::Update))
          .then(|| resolve_layer(state, phase, TARGET_MARQUEE, drag.layer, point))
          .flatten(),
      )
    }
    Press::Ordinary | Press::Toggle => None,
  }
}

/// What a press that never travelled does on release: a toggle adds its
/// annotation to the choice or takes it out; a click on one of a group
/// chooses it alone, and one inside the group's box but on no member lets the
/// group go; a click with the marquee lets the choice go unless the toggle
/// modifier is held at either end.
pub(super) fn clicked(state: &mut SurfaceState, drag: Drag, point: (f64, f64)) -> Vec<Sample> {
  let clear = |state: &SurfaceState| {
    resolve(
      state,
      SelectionGesturePhase::Begin,
      TARGET_NONE,
      0,
      HANDLE_BODY,
      point,
    )
  };
  match drag.press {
    Press::Toggle => resolve(
      state,
      SelectionGesturePhase::Begin,
      TARGET_TOGGLE,
      drag.index,
      HANDLE_BODY,
      point,
    )
    .into_iter()
    .collect(),
    Press::Group if drag.index != u32::MAX => super::choose(state, drag.index as usize, point)
      .into_iter()
      .collect(),
    Press::Group => clear(state).into_iter().collect(),
    Press::Marquee { additive } if !additive && !press_toggles() => {
      clear(state).into_iter().collect()
    }
    Press::Marquee { .. } | Press::Ordinary => Vec::new(),
  }
}
