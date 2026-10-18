// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's windows: the anchor view laid over the region webview, and
//! a borderless non-activating panel on every other display. Each holds a
//! Metal layer the engine draws into. Every view lives on the main thread,
//! named to the engine by its address.

use std::cell::RefCell;
use std::collections::HashMap;

use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly, Message};
use objc2_app_kit::{
  NSAutoresizingMaskOptions, NSBackingStoreType, NSColor, NSPanel, NSView,
  NSWindowCollectionBehavior, NSWindowOrderingMode, NSWindowSharingType, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect};
use objc2_quartz_core::CAMetalLayer;
use screenwide_osc::geometry::Rect;

use super::super::region_osc::host::WindowId;
use super::geometry::{self, Placement};

pub(super) struct Overlay {
  pub(super) view: Retained<NSView>,
  pub(super) layer: Retained<CAMetalLayer>,
  /// The panel a peer is, `None` for the anchor.
  pub(super) panel: Option<Retained<NSPanel>>,
  /// The region window's content view an anchor covers, `None` for a peer.
  /// The engine measures the anchor by this view, as it measures the
  /// webview's window on Windows.
  pub(super) host: Option<Retained<NSView>>,
}

thread_local! {
  /// Only ever touched on the main thread.
  static OVERLAYS: RefCell<HashMap<isize, Overlay>> = RefCell::new(HashMap::new());
}

pub(super) fn with<R>(window: WindowId, work: impl FnOnce(&Overlay) -> R) -> Option<R> {
  OVERLAYS.with(|overlays| overlays.borrow().get(&window.0).map(work))
}

/// Every overlay window, for routing events.
pub(super) fn each(mut work: impl FnMut(WindowId, &Overlay) -> bool) -> Option<WindowId> {
  OVERLAYS.with(|overlays| {
    overlays
      .borrow()
      .iter()
      .find(|(id, overlay)| work(WindowId(**id), overlay))
      .map(|(id, _)| WindowId(*id))
  })
}

pub(super) fn is_empty() -> bool {
  OVERLAYS.with(|overlays| overlays.borrow().is_empty())
}

/// Records `window`'s placement again from its view, which a resize may have
/// moved since: an overlay by its own view, a host by the content view its
/// anchor covers. Main thread only, where the views are.
pub(super) fn refresh_placement(window: WindowId) {
  OVERLAYS.with(|overlays| {
    let overlays = overlays.borrow();
    if let Some(overlay) = overlays.get(&window.0) {
      if overlay.panel.is_none() {
        record_placement(window.0, &overlay.view);
      }
      return;
    }
    if let Some(host) = overlays
      .values()
      .filter_map(|overlay| overlay.host.as_ref())
      .find(|host| Retained::as_ptr(host) as isize == window.0)
    {
      record_placement(window.0, host);
    }
  });
}

/// A view whose layer is a Metal layer, for the engine to draw into.
fn metal_view(mtm: MainThreadMarker, frame: NSRect) -> (Retained<NSView>, Retained<CAMetalLayer>) {
  let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
  let layer = CAMetalLayer::new();
  layer.setOpaque(false);
  view.setLayer(Some(&layer));
  view.setWantsLayer(true);
  (view, layer)
}

/// Records where `view` is, as the engine reads it.
pub(super) fn record_placement(id: isize, view: &NSView) {
  let Some(window) = view.window() else {
    return;
  };
  let in_window = view.convertRect_toView(view.bounds(), None);
  let on_screen = window.convertRectToScreen(in_window);
  let scale = window.backingScaleFactor();
  geometry::set_placement(
    id,
    Placement {
      frame: geometry::pixels(on_screen, scale),
      scale,
    },
  );
}

/// The anchor overlay: a view over `host`, the region window's content view,
/// above its webview.
pub(super) fn create_anchor(mtm: MainThreadMarker, host: &NSView) -> WindowId {
  let (view, layer) = metal_view(mtm, host.bounds());
  view.setAutoresizingMask(
    NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
  );
  let page = crate::app_windows::platform::webview_view(host);
  host.addSubview_positioned_relativeTo(&view, NSWindowOrderingMode::Above, page.as_deref());
  let id = Retained::as_ptr(&view) as isize;
  record_placement(id, &view);
  record_placement(host as *const NSView as isize, host);
  OVERLAYS.with(|overlays| {
    overlays.borrow_mut().insert(
      id,
      Overlay {
        view,
        layer,
        panel: None,
        host: Some(host.retain()),
      },
    )
  });
  WindowId(id)
}

/// A peer covering `bounds` on another display, at `parent`'s level, kept
/// out of captures unless `capturable`. Pointer events pass through until
/// the engine claims them.
pub(super) fn create_peer(
  mtm: MainThreadMarker,
  parent: &NSView,
  bounds: Rect,
  capturable: bool,
) -> WindowId {
  let (frame, scale) = geometry::points(bounds);
  let local = NSRect::new(NSPoint::new(0.0, 0.0), frame.size);
  let panel = NSPanel::initWithContentRect_styleMask_backing_defer_screen(
    NSPanel::alloc(mtm),
    local,
    NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel,
    NSBackingStoreType::Buffered,
    false,
    None,
  );
  panel.setFrame_display(frame, false);
  panel.setBackgroundColor(Some(&NSColor::clearColor()));
  panel.setCollectionBehavior(
    NSWindowCollectionBehavior::CanJoinAllSpaces
      | NSWindowCollectionBehavior::FullScreenAuxiliary
      | NSWindowCollectionBehavior::Stationary,
  );
  panel.setHasShadow(false);
  panel.setHidesOnDeactivate(false);
  if let Some(parent) = parent.window() {
    panel.setLevel(parent.level());
  }
  panel.setOpaque(false);
  // Safety: the registry keeps the panel until it is destroyed.
  unsafe { panel.setReleasedWhenClosed(false) };
  panel.setSharingType(sharing(capturable));
  panel.setIgnoresMouseEvents(true);
  panel.setAcceptsMouseMovedEvents(true);
  let (view, layer) = metal_view(mtm, local);
  panel.setContentView(Some(&view));
  let id = Retained::as_ptr(&view) as isize;
  geometry::set_placement(
    id,
    Placement {
      frame: bounds,
      scale,
    },
  );
  OVERLAYS.with(|overlays| {
    overlays.borrow_mut().insert(
      id,
      Overlay {
        view,
        layer,
        panel: Some(panel),
        host: None,
      },
    )
  });
  WindowId(id)
}

pub(super) fn sharing(capturable: bool) -> NSWindowSharingType {
  if capturable {
    NSWindowSharingType::ReadOnly
  } else {
    NSWindowSharingType::None
  }
}

pub(super) fn destroy(window: WindowId) {
  let removed = OVERLAYS.with(|overlays| overlays.borrow_mut().remove(&window.0));
  geometry::forget(window.0);
  if let Some(overlay) = removed {
    if let Some(host) = &overlay.host {
      geometry::forget(Retained::as_ptr(host) as isize);
    }
    overlay.view.removeFromSuperview();
    if let Some(panel) = overlay.panel {
      panel.orderOut(None);
      panel.close();
    }
  }
}

/// Puts a peer over `bounds` and in front.
pub(super) fn place_peer(window: WindowId, bounds: Rect) {
  let (frame, scale) = geometry::points(bounds);
  geometry::set_placement(
    window.0,
    Placement {
      frame: bounds,
      scale,
    },
  );
  with(window, |overlay| {
    if let Some(panel) = &overlay.panel {
      panel.setFrame_display(frame, false);
      panel.orderFrontRegardless();
    }
  });
}

/// Brings the anchor view back above its webview, over the whole content.
pub(super) fn raise(window: WindowId) {
  with(window, |overlay| {
    // Safety: the view is alive in the registry.
    let Some(host) = (unsafe { overlay.view.superview() }) else {
      return;
    };
    overlay.view.setFrame(host.bounds());
    overlay.view.removeFromSuperviewWithoutNeedingDisplay();
    host.addSubview_positioned_relativeTo(&overlay.view, NSWindowOrderingMode::Above, None);
    record_placement(window.0, &overlay.view);
    record_placement(Retained::as_ptr(&host) as isize, &host);
  });
}

pub(super) fn show(window: WindowId, visible: bool) {
  with(window, |overlay| match &overlay.panel {
    Some(panel) if visible => panel.orderFrontRegardless(),
    Some(panel) => panel.orderOut(None),
    None => overlay.view.setHidden(!visible),
  });
}
