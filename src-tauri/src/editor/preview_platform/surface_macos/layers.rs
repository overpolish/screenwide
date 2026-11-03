// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Core Animation layers the surface draws into. Below the webview: the
//! backstop, the workspace and the audio ribbon, on a flipped root layer so
//! frames are laid out from the top left as the engine gives them. Over it,
//! on the input view: the selection chrome.
//!
//! Layer properties are safe to set from any thread inside a transaction,
//! which is how the engine's off-main-thread draws move them.

use std::sync::Mutex;

use objc2::rc::Retained;
use objc2_app_kit::NSColor;
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::{CACornerMask, CALayer, CAMetalLayer, CATransaction};
use screenwide_compositor::PreviewSurfaceCorners;

pub(super) struct Layers {
  /// The root of the view below the webview.
  pub(super) root: Retained<CALayer>,
  pub(super) backdrop: Retained<CALayer>,
  pub(super) workspace: Retained<CAMetalLayer>,
  pub(super) ribbon: Retained<CAMetalLayer>,
  /// The root of the input view over the webview.
  pub(super) chrome_root: Retained<CALayer>,
  pub(super) chrome: Retained<CAMetalLayer>,
  /// The corners the layers were last rounded to.
  corners: Mutex<PreviewSurfaceCorners>,
}

// Core Animation layers may be changed from any thread inside a transaction,
// and nothing here reaches AppKit.
unsafe impl Send for Layers {}
unsafe impl Sync for Layers {}

/// Off screen, where a layer waits until it has something to show.
const AWAY: f64 = -100_000.0;

fn metal_layer() -> Retained<CAMetalLayer> {
  let layer = CAMetalLayer::new();
  layer.setOpaque(false);
  layer.setPresentsWithTransaction(false);
  layer.setAllowsNextDrawableTimeout(false);
  layer
}

impl Layers {
  pub(super) fn new() -> Self {
    let root = CALayer::new();
    root.setGeometryFlipped(true);
    let backdrop = CALayer::new();
    let workspace = metal_layer();
    let ribbon = metal_layer();
    root.addSublayer(&backdrop);
    root.addSublayer(&workspace);
    root.addSublayer(&ribbon);
    let chrome_root = CALayer::new();
    chrome_root.setGeometryFlipped(true);
    let chrome = metal_layer();
    chrome_root.addSublayer(&chrome);
    let layers = Self {
      root,
      backdrop,
      workspace,
      ribbon,
      chrome_root,
      chrome,
      corners: Mutex::new(PreviewSurfaceCorners::default()),
    };
    transaction(|| {
      let all: [&CALayer; 4] = [
        &layers.backdrop,
        &layers.workspace,
        &layers.ribbon,
        &layers.chrome,
      ];
      for layer in all {
        layer.setFrame(rect(AWAY, AWAY, 2.0, 2.0));
      }
    });
    layers
  }

  /// Sets the screen's scale on every layer that draws.
  pub(super) fn set_scale(&self, scale: f64) {
    transaction(|| {
      let all: [&CALayer; 4] = [&self.backdrop, &self.workspace, &self.ribbon, &self.chrome];
      for layer in all {
        layer.setContentsScale(scale);
      }
    });
  }

  /// Fills the backstop with `color`, straight alpha.
  pub(super) fn paint_backdrop(&self, color: [f64; 4]) {
    let color = NSColor::colorWithSRGBRed_green_blue_alpha(
      color[0].clamp(0.0, 1.0),
      color[1].clamp(0.0, 1.0),
      color[2].clamp(0.0, 1.0),
      color[3].clamp(0.0, 1.0),
    );
    transaction(|| self.backdrop.setBackgroundColor(Some(&color.CGColor())));
  }

  /// Rounds the corners of every layer laid over the viewport - the
  /// backstop, the workspace with its checkerboard, the ribbon and the
  /// chrome - so the picture meets the window's chrome on the curve the
  /// webview's hole has. Core Animation takes one radius, which every
  /// rounded corner shares.
  pub(super) fn round(&self, corners: PreviewSurfaceCorners) {
    match self.corners.lock() {
      Ok(mut last) if *last != corners => *last = corners,
      _ => return,
    }
    // The roots are flipped, so a layer's least y is its top.
    let mut mask = CACornerMask::empty();
    let mut radius: f64 = 0.0;
    for (corner, flag) in [
      (corners.top_left, CACornerMask::LayerMinXMinYCorner),
      (corners.top_right, CACornerMask::LayerMaxXMinYCorner),
      (corners.bottom_right, CACornerMask::LayerMaxXMaxYCorner),
      (corners.bottom_left, CACornerMask::LayerMinXMaxYCorner),
    ] {
      if corner > 0.0 {
        mask |= flag;
        radius = radius.max(corner);
      }
    }
    transaction(|| {
      let all: [&CALayer; 4] = [&self.backdrop, &self.workspace, &self.ribbon, &self.chrome];
      for layer in all {
        layer.setCornerRadius(radius);
        layer.setMaskedCorners(mask);
        layer.setMasksToBounds(radius > 0.0);
      }
    });
  }
}

/// Puts `layer` at `frame`, in points from the root's top left, or off
/// screen for `None`.
pub(super) fn place(layer: &CALayer, frame: Option<(f64, f64, f64, f64)>) {
  let (x, y, width, height) = frame.unwrap_or((AWAY, AWAY, 2.0, 2.0));
  transaction(|| layer.setFrame(rect(x, y, width.max(1.0), height.max(1.0))));
}

pub(super) fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
  NSRect::new(NSPoint::new(x, y), NSSize::new(width, height))
}

/// Runs `change` in a transaction of its own with implicit animations off,
/// so a move lands at once from any thread. Off the main thread the change
/// is flushed too: a Metal present there opens an implicit transaction that
/// nothing on that thread ever commits, and a commit nested in it waits for
/// it, so a layer moved off the main thread stayed where it was until
/// something on the main thread committed (the audio ribbon not showing,
/// or staying over the picture until a click).
pub(super) fn transaction(change: impl FnOnce()) {
  CATransaction::begin();
  CATransaction::setDisableActions(true);
  change();
  CATransaction::commit();
  if objc2::MainThreadMarker::new().is_none() {
    CATransaction::flush();
  }
}
