// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::ruler::island::{RulerIslandState, RulerTool};

impl RulerState {
  /// What the island shows of the session, read off it rather than kept
  /// beside it: a guide or radius gesture in progress is the tool in hand,
  /// whether a held key or the island started it. None while no session is
  /// up.
  pub(crate) fn island_state(&self) -> Option<RulerIslandState> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active {
      return None;
    }
    let tool = match (&session.guide, &session.radius) {
      (Some(guide), _) => match guide.visual.axis {
        GuideAxis::Vertical => RulerTool::VerticalGuide,
        GuideAxis::Horizontal => RulerTool::HorizontalGuide,
      },
      (None, Some(_)) => RulerTool::Radius,
      (None, None) => RulerTool::Measure,
    };
    Some(RulerIslandState {
      tool,
      tolerance: session.tolerance.into(),
      centerlines: session.centerlines_visible,
      crosshair: session.visual.is_some_and(|visual| visual.crosshair),
      can_undo: !session.undo.is_empty(),
      can_copy: latest_target(&session.document).is_some(),
    })
  }
}
