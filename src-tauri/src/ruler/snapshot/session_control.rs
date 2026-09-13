// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RulerState {
  pub(in crate::ruler) fn cancel(&self) -> bool {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.generation = session.generation.wrapping_add(1);
    session.displays.clear();
    session.visual = None;
    session.copied_until = None;
    session.tolerance_until = None;
    session.option_active = false;
    session.boxes.clear();
    session.drag = Drag::default();
    session.document = Document::default();
    session.undo.clear();
    session.redo.clear();
    session.hovered_target = None;
    session.label_drag = None;
    session.range = None;
    session.guide = None;
    session.guide_drag = None;
    session.radius = None;
    session.settle = None;
    std::mem::replace(&mut session.active, false)
  }
}
