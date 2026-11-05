// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A reference's DirectComposition tree on Windows: the picture's visual and
//! the chrome's over it, each drawn by wgpu, in a topmost target on the
//! reference's window, so they draw over the webview without being a window
//! that takes the pointer. The picture's visual takes the reference's
//! opacity through an effect group of its own; the whole tree fades through
//! another as the reference closes, which DirectComposition animates by
//! itself.

use screenwide_gpu::surface::Surface;
use screenwide_gpu::Gpu;
use windows::Win32::{
  Foundation::HWND,
  Graphics::{
    DirectComposition::{
      DCompositionCreateDevice, IDCompositionDevice, IDCompositionEffectGroup, IDCompositionTarget,
      IDCompositionVisual, DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
    },
    Dxgi::IDXGIDevice,
  },
};

pub(super) struct Composition {
  device: IDCompositionDevice,
  /// Held only to keep the tree alive.
  _target: IDCompositionTarget,
  /// Placed where the view stands in the window.
  root: IDCompositionVisual,
  _picture: IDCompositionVisual,
  _chrome: IDCompositionVisual,
  /// The picture's opacity.
  picture_opacity: IDCompositionEffectGroup,
  /// The whole reference's opacity, which falls as it closes.
  fade: IDCompositionEffectGroup,
}

fn failed(what: &str) -> impl FnOnce(windows::core::Error) -> String + '_ {
  move |error| format!("{what}: {error}")
}

impl Composition {
  /// The tree over `window`'s content and its webview, and the surfaces the
  /// picture and the chrome are drawn on.
  pub(super) fn new(gpu: &Gpu, window: HWND) -> Result<(Self, (Surface, Surface)), String> {
    // The swap chains are the visuals' only content, so the device needs no
    // Direct3D device of its own.
    let device: IDCompositionDevice = unsafe { DCompositionCreateDevice(None::<&IDXGIDevice>) }
      .map_err(failed("DirectComposition could not open"))?;
    let target = unsafe { device.CreateTargetForHwnd(window, true) }
      .map_err(failed("A reference's view could not attach"))?;
    let visual = || {
      let visual =
        unsafe { device.CreateVisual() }.map_err(failed("A visual could not be made"))?;
      // DirectComposition defaults to nearest-neighbour bitmap sampling.
      unsafe { visual.SetBitmapInterpolationMode(DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR) }
        .map_err(failed("A visual could not be set up"))?;
      Ok::<_, String>(visual)
    };
    let (root, picture, chrome) = (visual()?, visual()?, visual()?);
    let picture_opacity =
      unsafe { device.CreateEffectGroup() }.map_err(failed("An effect could not be made"))?;
    let fade =
      unsafe { device.CreateEffectGroup() }.map_err(failed("An effect could not be made"))?;
    unsafe {
      picture
        .SetEffect(&picture_opacity)
        .map_err(failed("The picture's opacity could not be set up"))?;
      root
        .SetEffect(&fade)
        .map_err(failed("The fade could not be set up"))?;
      target
        .SetRoot(&root)
        .map_err(failed("A reference's view could not attach"))?;
      // With no visual to add it next to, `false` puts each in front of
      // every visual before it: the chrome over the picture.
      for visual in [&picture, &chrome] {
        root
          .AddVisual(visual, false, None::<&IDCompositionVisual>)
          .map_err(failed("A visual could not be added"))?;
      }
    }
    let surfaces = (
      Surface::on_visual(gpu, &picture)?,
      Surface::on_visual(gpu, &chrome)?,
    );
    unsafe { device.Commit() }.map_err(failed("A reference's view could not be shown"))?;
    Ok((
      Self {
        device,
        _target: target,
        root,
        _picture: picture,
        _chrome: chrome,
        picture_opacity,
        fade,
      },
      surfaces,
    ))
  }

  /// Puts the view's corner at `x`, `y` in the window's pixels.
  pub(super) fn set_origin(&self, x: f32, y: f32) {
    let set = unsafe { self.root.SetOffsetX2(x) }
      .and_then(|()| unsafe { self.root.SetOffsetY2(y) })
      .and_then(|()| unsafe { self.device.Commit() });
    if let Err(error) = set {
      eprintln!("A reference's view could not be placed: {error}");
    }
  }

  pub(super) fn set_picture_opacity(&self, opacity: f32) {
    let set = unsafe { self.picture_opacity.SetOpacity2(opacity) }
      .and_then(|()| unsafe { self.device.Commit() });
    if let Err(error) = set {
      eprintln!("A reference's opacity could not be set: {error}");
    }
  }

  /// Fades the whole reference out over `seconds`, evenly.
  pub(super) fn fade_out(&self, seconds: f64) {
    let faded = unsafe { self.device.CreateAnimation() }.and_then(|animation| unsafe {
      animation.AddCubic(0.0, 1.0, -(1.0 / seconds) as f32, 0.0, 0.0)?;
      animation.End(seconds, 0.0)?;
      self.fade.SetOpacity(&animation)?;
      self.device.Commit()
    });
    if let Err(error) = faded {
      eprintln!("A reference could not fade: {error}");
    }
  }
}
