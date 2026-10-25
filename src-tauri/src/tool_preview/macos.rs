// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a preview is drawn on macOS: a `CAMetalLayer` in a view of its own
//! beside the webview. A tool that draws its screen sits immediately below
//! the webview, so the page shows it through a transparent box and paints
//! over it everywhere else; one that draws on glass sits over the page's
//! own picture. A clip layer holds the drawing to the part of the page it
//! may be seen in, such as a scroll area's viewport.

use std::ptr::NonNull;

use dispatch2::MainThreadBound;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol};
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
  NSAutoresizingMaskOptions, NSColor, NSResponder, NSView, NSWindowOrderingMode,
};
use objc2_foundation::{NSArray, NSEdgeInsets, NSNumber, NSObject, NSPoint, NSRect, NSSize};
use objc2_quartz_core::{CAGradientLayer, CALayer, CAMetalLayer, CATransaction};
use tauri::WebviewWindow;

use super::{Fade, Placement};

define_class!(
  // Safety: NSView has no subclassing requirements this class breaks, and
  // it implements no `Drop`.
  #[unsafe(super(NSView, NSResponder, NSObject))]
  #[thread_kind = MainThreadOnly]
  #[name = "ScreenwideToolPreviewView"]
  struct PreviewView;

  impl PreviewView {
    /// AppKit sets a hosted layer's `geometryFlipped` from its view, so the
    /// layers are placed from the top left, as the page measures.
    #[unsafe(method(isFlipped))]
    fn is_flipped(&self) -> bool {
      true
    }

    /// The webview takes every event, whichever side of it the view is on.
    #[unsafe(method_id(hitTest:))]
    fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> {
      None
    }
  }

  unsafe impl NSObjectProtocol for PreviewView {}
);

/// One preview's view and layers, which live on the main thread.
pub(super) struct Slot {
  /// The webview, whose page may start inside its frame.
  page: Retained<NSView>,
  view: Retained<PreviewView>,
  clip: Retained<CALayer>,
  /// The clip's mask while the scroll area it is in fades an end.
  fade: Retained<CAGradientLayer>,
  /// The page's box, rounded, which shows the middle of the picture.
  frame: Retained<CALayer>,
  layer: Retained<CAMetalLayer>,
}

impl Drop for Slot {
  fn drop(&mut self) {
    self.view.removeFromSuperview();
  }
}

/// Makes a preview's view beside `window`'s webview, below it or over it,
/// and a GPU surface on its layer. A view over the page lets every event
/// through to it.
pub(super) fn create(
  window: &WebviewWindow,
  scale: f64,
  over_page: bool,
) -> Result<(MainThreadBound<Slot>, screenwide_gpu::surface::Surface), String> {
  let host = window.ns_view().map_err(|error| error.to_string())? as usize;
  let gpu = screenwide_gpu::shared()?;
  dispatch2::run_on_main(move |mtm| {
    // Safety: Tauri hands over the window's live content view.
    let host = unsafe { Retained::retain(host as *mut NSView) }
      .ok_or_else(|| "The window has no content view".to_owned())?;
    let page = crate::app_windows::platform::webview_view(&host)
      .ok_or_else(|| "The window has no webview".to_owned())?;
    let view = new_view(mtm, page.frame());
    let root = CALayer::new();
    let clip = CALayer::new();
    clip.setMasksToBounds(true);
    let frame = CALayer::new();
    frame.setMasksToBounds(true);
    let layer = CAMetalLayer::new();
    layer.setOpaque(false);
    layer.setPresentsWithTransaction(false);
    layer.setAllowsNextDrawableTimeout(false);
    layer.setContentsScale(scale);
    root.addSublayer(&clip);
    clip.addSublayer(&frame);
    frame.addSublayer(&layer);
    // A layer-hosting view: the layer is set before the view wants one.
    view.setLayer(Some(&root));
    view.setWantsLayer(true);
    view.setAutoresizingMask(
      NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    let order = if over_page {
      NSWindowOrderingMode::Above
    } else {
      NSWindowOrderingMode::Below
    };
    host.addSubview_positioned_relativeTo(&view, order, Some(&page));
    let slot = Slot {
      page,
      view,
      clip,
      fade: fade_mask(),
      frame,
      layer,
    };
    slot.place(Placement::hidden());
    let pointer = NonNull::from(&*slot.layer).cast::<std::ffi::c_void>();
    // Safety: on the main thread, on a live layer the slot keeps.
    let surface = unsafe { screenwide_gpu::surface::Surface::on_metal_layer(gpu, pointer) }?;
    Ok((MainThreadBound::new(slot, mtm), surface))
  })
}

fn new_view(mtm: MainThreadMarker, frame: NSRect) -> Retained<PreviewView> {
  // Safety: NSView's designated initialiser.
  unsafe { msg_send![PreviewView::alloc(mtm), initWithFrame: frame] }
}

/// A mask that is opaque between its two faded ends. Only its alpha counts.
fn fade_mask() -> Retained<CAGradientLayer> {
  let mask = CAGradientLayer::new();
  let clear = NSColor::clearColor().CGColor();
  let opaque = NSColor::blackColor().CGColor();
  // Safety: a `CGColor` is toll-free bridged to an Objective-C object, which
  // is what the layer's `colors` holds.
  let (clear, opaque) = unsafe {
    (
      &*(&raw const *clear).cast::<AnyObject>(),
      &*(&raw const *opaque).cast::<AnyObject>(),
    )
  };
  // Safety: the array holds `CGColor`s, as `colors` requires.
  unsafe { mask.setColors(Some(&NSArray::from_slice(&[clear, opaque, opaque, clear]))) };
  mask
}

impl Slot {
  /// Where the page's top left is in the webview. A window whose title bar
  /// sits over its content has the webview run under the bar, and WebKit
  /// starts the page below it: page points are offset from the webview's.
  fn page_origin(&self) -> (f64, f64) {
    let responds = |name: &std::ffi::CStr| -> bool {
      let selector = objc2::runtime::Sel::register(name);
      // Safety: `respondsToSelector:` takes any selector.
      unsafe { msg_send![&*self.page, respondsToSelector: selector] }
    };
    if responds(c"obscuredContentInsets") {
      // Safety: WKWebView's own property, macOS 26 and later.
      let insets: NSEdgeInsets = unsafe { msg_send![&*self.page, obscuredContentInsets] };
      (insets.left, insets.top)
    } else if responds(c"_topContentInset") {
      // Safety: the same inset's name before it was public, a `CGFloat`.
      let top: f64 = unsafe { msg_send![&*self.page, _topContentInset] };
      (0.0, top)
    } else {
      (0.0, 0.0)
    }
  }

  /// Puts the drawing at `placement`, in the page's points.
  pub(super) fn place(&self, placement: Placement) {
    let Placement {
      frame,
      picture,
      clip,
      fade,
      radius,
      visible,
      scale,
    } = placement;
    let (left, top) = self.page_origin();
    transaction(|| {
      self.view.setHidden(!visible);
      // Only the clip is placed in the webview's space; the layers in it
      // are placed relative to their parent, in page points alike.
      self
        .clip
        .setFrame(rect(clip.x + left, clip.y + top, clip.width, clip.height));
      self.frame.setFrame(rect(
        frame.x - clip.x,
        frame.y - clip.y,
        frame.width,
        frame.height,
      ));
      self.frame.setCornerRadius(radius);
      self.layer.setFrame(rect(
        picture.x - frame.x,
        picture.y - frame.y,
        picture.width,
        picture.height,
      ));
      self.layer.setContentsScale(scale);
      self.set_fade(fade, clip.x, clip.y);
    });
  }

  /// Masks the clip as the scroll area fades its content, with the mask laid
  /// over the scroll area's viewport in the clip's own space.
  fn set_fade(&self, fade: Option<Fade>, clip_x: f64, clip_y: f64) {
    let Some(Fade {
      area,
      vertical,
      start,
      end,
    }) = fade
    else {
      // Safety: clearing a mask has no requirement.
      unsafe { self.clip.setMask(None) };
      return;
    };
    let length = if vertical { area.height } else { area.width }.max(1.0);
    self.fade.setFrame(rect(
      area.x - clip_x,
      area.y - clip_y,
      area.width,
      area.height,
    ));
    let (from, to) = if vertical {
      (NSPoint::new(0.5, 0.0), NSPoint::new(0.5, 1.0))
    } else {
      (NSPoint::new(0.0, 0.5), NSPoint::new(1.0, 0.5))
    };
    self.fade.setStartPoint(from);
    self.fade.setEndPoint(to);
    let locations = [0.0, start / length, 1.0 - end / length, 1.0].map(NSNumber::new_f64);
    self
      .fade
      .setLocations(Some(&NSArray::from_retained_slice(&locations)));
    // Safety: the mask is the slot's own and in no layer tree of its own; a
    // mask may not also be a sublayer.
    unsafe { self.clip.setMask(Some(&self.fade)) };
  }
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
  NSRect::new(
    NSPoint::new(x, y),
    NSSize::new(width.max(0.0), height.max(0.0)),
  )
}

/// Runs `change` with implicit animations off, so a move lands at once.
fn transaction(change: impl FnOnce()) {
  CATransaction::begin();
  CATransaction::setDisableActions(true);
  change();
  CATransaction::commit();
}
