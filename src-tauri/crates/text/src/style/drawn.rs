// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A style's control points and the elements drawn to them: which of them a
//! style can hold, and the tail it draws.

use super::{TextBackground, TextStyle};
use crate::controls::TextControl;
use crate::elements::{TailShape, TextElement};

impl TextStyle {
  /// The elements that name a control point the style declares, at most
  /// one tail among them.
  pub(super) fn normalized_elements(&self) -> Vec<TextElement> {
    let declared = self.normalized_controls();
    let mut elements: Vec<TextElement> = Vec::new();
    for element in &self.elements {
      let named = declared
        .iter()
        .any(|control| control.id == element.control());
      let tail = element.tail().is_some();
      let has_tail = elements.iter().any(|kept| kept.tail().is_some());
      if named && !(tail && has_tail) {
        elements.push(element.normalized());
      }
    }
    elements
  }

  /// The control points with an id of their own that an element is drawn
  /// to: a point that drives nothing would be a grip that moves nothing.
  pub(super) fn normalized_controls(&self) -> Vec<TextControl> {
    let mut controls: Vec<TextControl> = Vec::new();
    for control in &self.controls {
      let used = self
        .elements
        .iter()
        .any(|element| element.control() == control.id);
      let taken = controls.iter().any(|kept| kept.id == control.id);
      if control.has_valid_id() && used && !taken {
        controls.push(control.normalized());
      }
    }
    controls
  }

  /// The tail the style draws from its box, and the control point it is
  /// drawn to: none without a box to draw it from.
  pub fn tail(&self) -> Option<(&TextControl, TailShape)> {
    if self.background != TextBackground::Box {
      return None;
    }
    self.elements.iter().find_map(|element| {
      let shape = element.tail()?;
      let control = self
        .controls
        .iter()
        .find(|control| control.id == element.control())?;
      Some((control, shape))
    })
  }
}
