// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cidre::{arc, ax, cf, cg};
use core_graphics::geometry::CGPoint;

/// The one unavoidable guess. It stands in for a titlebar whose height the
/// Accessibility API refuses to describe - an Electron-style custom titlebar
/// exposes no buttons, no title element and no toolbar - and it doubles as the
/// window-top tolerance that decides whether a toolbar or tab group is chrome
/// or content. It is never mixed into a band the API *was* able to measure.
pub(super) const FALLBACK_TITLEBAR_HEIGHT: f64 = 32.0;
/// Slack below the lowest chrome element, so the last row of titlebar pixels
/// still starts a gesture.
pub(super) const CHROME_MARGIN: f64 = 6.0;
/// `kAXToolbarAttribute`, which cidre does not expose as a constant.
const TOOLBAR_ATTRIBUTE: &str = "AXToolbar";
/// Matches `owning_window`'s bound: deep enough for a button inside a stack
/// inside a toolbar, shallow enough to stay cheap on every drag sample.
const ANCESTOR_HOPS: usize = 8;

/// What the Accessibility hit test found under a point. An own-process hit is
/// reported as such rather than resolved - self-queries return husks - so the
/// caller can pay the native path's main-thread hops only when they are due.
pub(super) enum AxHit {
  Foreign(arc::R<ax::UiElement>, arc::R<ax::UiElement>),
  OwnProcess,
  Miss,
}

/// The titlebar verdict for a point. A foreign hit outside the chrome band is
/// a miss; an own-process hit is handed back for the native resolver.
pub(super) enum AxTitlebar {
  Titlebar(arc::R<ax::UiElement>, cg::Rect),
  OwnProcess,
  Miss,
}

/// The window under a global point, hit-tested over Accessibility alone.
pub(super) fn ax_window_at(point: CGPoint) -> AxHit {
  hit_at(point)
}

/// The window a titlebar point belongs to, with the frame the verdict was
/// reached against. A session captures both at its anchor: once the window has
/// been moved away, hit-testing that anchor again would find a different one.
pub(super) fn ax_titlebar_at(point: CGPoint) -> AxTitlebar {
  let (element, window) = match hit_at(point) {
    AxHit::Foreign(element, window) => (element, window),
    AxHit::OwnProcess => return AxTitlebar::OwnProcess,
    AxHit::Miss => return AxTitlebar::Miss,
  };
  let Some(frame) = window.frame().ok().and_then(|value| value.cg_rect()) else {
    return AxTitlebar::Miss;
  };
  let measured = [
    ax::attr::close_button(),
    ax::attr::minimize_button(),
    ax::attr::title_ui_element(),
    ax::attr::zoom_button(),
  ]
  .into_iter()
  .filter_map(|attr| accessible_frame(&window, attr))
  .map(|control| control.origin.y + control.size.height)
  .chain(toolbar_bottom(&window, frame))
  .reduce(f64::max);
  // Real measurements are trusted on their own - a window with chrome shorter
  // than the fallback gets the smaller, truthful band.
  let chrome_bottom = measured.unwrap_or(frame.origin.y + FALLBACK_TITLEBAR_HEIGHT) + CHROME_MARGIN;
  // What the point landed *on* beats where it landed, provided the containing
  // toolbar or tab group is itself confined to the measured chrome. Safari's
  // web content is a window-sized AXTabGroup rooted at the top of the window;
  // treating that ancestor as chrome makes every page pixel start Glide.
  if is_chrome_element(&element, &window, frame, chrome_bottom) {
    return AxTitlebar::Titlebar(window, frame);
  }
  let hit = point.x >= frame.origin.x
    && point.x <= frame.origin.x + frame.size.width
    && point.y >= frame.origin.y
    && point.y <= chrome_bottom.min(frame.origin.y + frame.size.height);
  if hit {
    AxTitlebar::Titlebar(window, frame)
  } else {
    AxTitlebar::Miss
  }
}

/// The element under the point and the window that owns it. Screenwide's own
/// windows are never resolved here: asking the Accessibility API about our own
/// process means our tap thread interrogating our own busy run loop through
/// WebKit's remote proxies, which returns husks - the own-process outcome tells
/// the caller to resolve natively instead, and only then pay for it.
fn hit_at(point: CGPoint) -> AxHit {
  if !ax::is_process_trusted() {
    return AxHit::Miss;
  }
  let system = ax::UiElement::sys_wide();
  let _ = system.set_messaging_timeout_secs(0.05);
  let Ok(element) = system.element_at_pos(point.x as f32, point.y as f32) else {
    return AxHit::Miss;
  };
  if element.pid().ok() == Some(std::process::id() as i32) {
    return AxHit::OwnProcess;
  }
  let Some(window) = owning_window(element.retained()) else {
    return AxHit::Miss;
  };
  AxHit::Foreign(element, window)
}

/// Walks from the hit element up towards - but never into - the window, asking
/// each node whether it is window chrome. Stopping short of the window matters:
/// the window itself owns every pixel, so it can never answer this question.
fn is_chrome_element(
  element: &ax::UiElement,
  window: &ax::UiElement,
  frame: cg::Rect,
  chrome_bottom: f64,
) -> bool {
  let title = window.attr_value(ax::attr::title_ui_element()).ok();
  let mut node = element.retained();
  for _ in 0..ANCESTOR_HOPS {
    if node.equal(window) {
      return false;
    }
    if is_chrome_role(&node, frame, chrome_bottom) {
      return true;
    }
    if title.as_ref().is_some_and(|title| title.equal(&node)) {
      return true;
    }
    let Ok(parent) = node.parent() else {
      return false;
    };
    node = parent;
  }
  false
}

/// A toolbar or tab group is chrome only while it hangs off the top of the
/// window; the same roles appear mid-content as segmented controls and tabbed
/// panes, which must keep scrolling and clicking normally. Window buttons are
/// chrome wherever the window decides to put them.
fn is_chrome_role(node: &ax::UiElement, frame: cg::Rect, chrome_bottom: f64) -> bool {
  let top_anchored_band = node
    .role()
    .is_ok_and(|role| role.equal(ax::role::toolbar()) || role.equal(ax::role::tab_group()))
    && node
      .frame()
      .ok()
      .and_then(|value| value.cg_rect())
      .is_some_and(|bounds| top_anchored_chrome_band(bounds, frame, chrome_bottom));
  if top_anchored_band {
    return true;
  }
  node.attr_value(ax::attr::subrole()).is_ok_and(|subrole| {
    [
      ax::sub_role::close_button(),
      ax::sub_role::minimize_button(),
      ax::sub_role::zoom_button(),
      ax::sub_role::full_screen_button(),
    ]
    .into_iter()
    .any(|button| subrole.equal(button))
  })
}

fn top_anchored_chrome_band(bounds: cg::Rect, frame: cg::Rect, chrome_bottom: f64) -> bool {
  bounds.origin.y <= frame.origin.y + FALLBACK_TITLEBAR_HEIGHT
    && bounds.origin.y + bounds.size.height <= chrome_bottom
}

/// A unified toolbar anchored at the top of the window is part of the same
/// visual band as the titlebar, and the buttons the geometry above measures sit
/// halfway down it. A toolbar further down the window does not widen the band.
fn toolbar_bottom(window: &ax::UiElement, frame: cg::Rect) -> Option<f64> {
  let name = cf::String::from_str(TOOLBAR_ATTRIBUTE);
  let toolbar = accessible_frame(window, ax::Attr::with_string(&name))?;
  (toolbar.origin.y <= frame.origin.y + FALLBACK_TITLEBAR_HEIGHT)
    .then_some(toolbar.origin.y + toolbar.size.height)
}

fn owning_window(element: arc::R<ax::UiElement>) -> Option<arc::R<ax::UiElement>> {
  if has_window_role(&element) {
    return Some(element);
  }
  if let Ok(window) = element.window() {
    return Some(window);
  }

  let mut ancestor = element;
  for _ in 0..ANCESTOR_HOPS {
    ancestor = ancestor.parent().ok()?;
    if has_window_role(&ancestor) {
      return Some(ancestor);
    }
    if let Ok(window) = ancestor.window() {
      return Some(window);
    }
  }
  None
}

fn has_window_role(element: &ax::UiElement) -> bool {
  element
    .role()
    .is_ok_and(|role| role.equal(ax::role::window()))
}

fn accessible_frame(window: &ax::UiElement, attr: &ax::Attr) -> Option<cg::Rect> {
  let value = window.attr_value(attr).ok()?;
  let element: arc::R<ax::UiElement> = unsafe { cf::Type::retain(&value) };
  element.frame().ok()?.cg_rect()
}

#[cfg(test)]
mod tests {
  use super::top_anchored_chrome_band;
  use cidre::cg;

  fn rect(x: f64, y: f64, width: f64, height: f64) -> cg::Rect {
    cg::Rect {
      origin: cg::Point { x, y },
      size: cg::Size { width, height },
    }
  }

  #[test]
  fn accepts_a_tab_group_confined_to_measured_chrome() {
    let window = rect(100.0, 50.0, 900.0, 700.0);
    assert!(top_anchored_chrome_band(
      rect(100.0, 50.0, 900.0, 76.0),
      window,
      132.0,
    ));
  }

  #[test]
  fn rejects_a_top_anchored_tab_group_that_fills_safari_content() {
    let window = rect(100.0, 50.0, 900.0, 700.0);
    assert!(!top_anchored_chrome_band(window, window, 132.0));
  }

  #[test]
  fn rejects_a_tab_group_below_the_window_chrome() {
    let window = rect(100.0, 50.0, 900.0, 700.0);
    assert!(!top_anchored_chrome_band(
      rect(100.0, 200.0, 900.0, 76.0),
      window,
      132.0,
    ));
  }
}
