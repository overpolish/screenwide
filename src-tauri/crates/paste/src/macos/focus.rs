// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What has the keyboard in the app in front, as Accessibility tells it. A
//! field, a text view or a terminal says it is text, or has a value that can
//! be set or a selection in its text, or sits inside an editable web element;
//! a web page that can be edited nowhere, Screenwide's own pages among them,
//! or Finder's list of files says none of that. An Electron app that has not
//! been asked for its accessibility says nothing at all, and is taken to have
//! pasted, as it most often has.

use cidre::{ax, cf};

use crate::Focus;

/// How long the app in front is given to answer.
const ANSWER: f32 = 0.25;
/// A web element inside one the person can edit, which Safari and Chrome
/// tell for every node of a content-editable area.
const EDITABLE_ANCESTOR: &str = "AXEditableAncestor";
/// A web page itself, which has the keyboard when none of its fields does.
const WEB_AREA: &str = "AXWebArea";

/// What has the keyboard in the app whose process is `pid`. That app's own
/// element is asked: the system-wide element answered that it could not
/// complete the request when asked from a paste's thread.
pub(super) fn focus(pid: i32) -> Focus {
  if !ax::is_process_trusted() {
    return Focus::Unknown;
  }
  let app = ax::UiElement::with_app_pid(pid);
  let _ = app.set_messaging_timeout_secs(ANSWER);
  let Ok(element) = app.focused_ui_element() else {
    return Focus::Unknown;
  };
  if takes_text(&element) {
    Focus::TakesText
  } else {
    Focus::Elsewhere
  }
}

fn takes_text(element: &ax::UiElement) -> bool {
  let role = element.role().ok();
  let is = |kind: &ax::Role| role.as_ref().is_some_and(|role| role.equal(kind));
  let editable_ancestor = cf::String::from_str(EDITABLE_ANCESTOR);
  let editable = element.is_settable(ax::attr::value()).unwrap_or(false)
    || element
      .attr_value(ax::Attr::with_string(&editable_ancestor))
      .is_ok();
  // A page tells its caret's place for reading even where nothing on it can
  // be edited, so only being editable makes a page take text.
  let web_area = cf::String::from_str(WEB_AREA);
  if is(ax::Role::with_string(&web_area)) {
    return editable;
  }
  is(ax::role::text_field())
    || is(ax::role::text_area())
    || is(ax::role::combo_box())
    || editable
    || element.attr_value(ax::attr::selected_text_range()).is_ok()
}
