// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the text caret is, as Accessibility tells it: the box of the
//! focused control's selection, or of the character before an empty one,
//! and for a field that will not say, the field itself. Accessibility
//! measures from the top left of the main display in points, as the cursor
//! file does.

use cidre::{arc, ax, cf};

use super::super::CaretRect;

/// How long the app in front is given to answer each question.
const ANSWER: f32 = 0.05;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
  fn AXUIElementCopyParameterizedAttributeValue(
    element: &ax::UiElement,
    attribute: &ax::ParamAttr,
    parameter: &cf::Type,
    result: *mut Option<arc::R<cf::Type>>,
  ) -> ax::Error;
}

/// Where the caret is in the control that has the keyboard, or `None` where
/// nothing has it or it does not say.
pub(super) fn caret() -> Option<CaretRect> {
  let system = ax::UiElement::sys_wide();
  let _ = system.set_messaging_timeout_secs(ANSWER);
  let focused = system.focused_ui_element().ok()?;
  let _ = focused.set_messaging_timeout_secs(ANSWER);
  selection(&focused).or_else(|| field(&focused))
}

fn selection(element: &ax::UiElement) -> Option<CaretRect> {
  let range = element
    .ax_value(ax::attr::selected_text_range())
    .ok()?
    .cf_range()?;
  if let Some(rect) = bounds(element, range) {
    return Some(rect);
  }
  // Many controls give no box for an empty selection; the character before
  // the caret ends where the caret is.
  if range.len == 0 && range.loc > 0 {
    let before = bounds(element, cf::Range::new(range.loc - 1, 1))?;
    return Some(CaretRect {
      x: before.x + before.width,
      width: 1.0,
      ..before
    });
  }
  None
}

fn bounds(element: &ax::UiElement, range: cf::Range) -> Option<CaretRect> {
  let parameter = ax::Value::with_cf_range(&range);
  let mut result = None;
  // SAFETY: the element and the parameter are live CF objects, and the
  // result is written only on success, owned by the caller.
  let status = unsafe {
    AXUIElementCopyParameterizedAttributeValue(
      element,
      ax::param_attr::bounds_for_range(),
      &parameter,
      &mut result,
    )
  };
  status.result().ok()?;
  let value = result?;
  if value.get_type_id() != ax::Value::type_id() {
    return None;
  }
  // SAFETY: the type was checked above.
  let value: arc::R<ax::Value> = unsafe { std::mem::transmute(value) };
  let rect = value.cg_rect()?;
  (rect.size.height > 0.0).then_some(CaretRect {
    x: rect.origin.x,
    y: rect.origin.y,
    width: rect.size.width.max(1.0),
    height: rect.size.height,
  })
}

/// A one-line field's own box, where it says nothing of its selection: the
/// text goes somewhere in it. A larger control's box would say no more than
/// the window does.
fn field(element: &ax::UiElement) -> Option<CaretRect> {
  let role = element.role().ok()?;
  let one_line = [ax::role::text_field(), ax::role::combo_box()]
    .iter()
    .any(|kind| role.equal(kind));
  if !one_line {
    return None;
  }
  let origin = element.pos().ok()?.cg_point()?;
  let size = element.size().ok()?.cg_size()?;
  (size.height > 0.0).then_some(CaretRect {
    x: origin.x,
    y: origin.y,
    width: size.width,
    height: size.height,
  })
}
