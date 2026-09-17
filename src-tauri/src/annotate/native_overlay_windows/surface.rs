// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One host window's overlay child and the swap chain its pixels arrive
//! through.
//!
//! The child is where the annotations are drawn: the Tauri host owns a WebView2
//! sibling, and only a `WS_EX_NOREDIRECTIONBITMAP` child under a topmost
//! composition target draws above it. `SetWindowDisplayAffinity` is never
//! called here - it only accepts top-level windows, and the child is already
//! covered by the host's own exclusion.

use super::*;

/// Registered once per process, on first attach.
static CLASS: OnceLock<u16> = OnceLock::new();

pub(super) struct Surface {
  /// The Tauri host this draws in, which is how a surface is found again.
  pub(super) host: HWND,
  /// Which [`Display`] the annotations are mapped into.
  pub(super) display: u32,
  pub(super) child: HWND,
  pub(super) chain: overlay_surface::CompositionSwapChain,
  /// The child's physical size, so a frame that has not changed size neither
  /// moves the window nor reallocates the buffers.
  size: (u32, u32),
}

impl Surface {
  pub(super) fn new(
    device: &overlay_surface::Device,
    host: HWND,
    display: u32,
  ) -> Result<Self, String> {
    let atom = *CLASS.get_or_init(|| {
      overlay_surface::register_class(
        w!("ScreenwideAnnotate"),
        Some(window_proc::window_proc),
        WNDCLASS_STYLES(0),
      )
    });
    if atom == 0 {
      return Err("The annotate overlay window class could not be registered".to_owned());
    }
    let child = overlay_surface::create_child(host, w!("ScreenwideAnnotate"))?;
    window_proc::set_state(child, display, false);
    let chain = device.create_swap_chain(child)?;
    let mut surface = Self {
      host,
      display,
      child,
      chain,
      size: (0, 0),
    };
    surface.fit()?;
    Ok(surface)
  }

  /// Matches the child and its buffers to the host's client area, and reports
  /// the physical size a frame draws at. Zero means there is nothing to draw:
  /// the host has no client area yet.
  pub(super) fn fit(&mut self) -> Result<(u32, u32), String> {
    let mut client = RECT::default();
    if unsafe { GetClientRect(self.host, &mut client) }.is_err() {
      return Ok((0, 0));
    }
    let size = (
      (client.right - client.left).max(0) as u32,
      (client.bottom - client.top).max(0) as u32,
    );
    if size.0 == 0 || size.1 == 0 {
      return Ok((0, 0));
    }
    if size != self.size {
      // Raised to the top of its siblings, not left where it was created: the
      // host's WebView2 child is a sibling, and a child window below it draws
      // behind it whatever its composition target says. This is what the
      // region OSC's `raise` does for the same reason.
      unsafe {
        SetWindowPos(
          self.child,
          Some(HWND_TOP),
          0,
          0,
          size.0 as i32,
          size.1 as i32,
          SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )
      }
      .map_err(|error| format!("The annotate overlay child could not be laid out: {error}"))?;
      let _ = unsafe { ShowWindowAsync(self.child, SW_SHOWNOACTIVATE) };
      self.size = size;
    }
    self.chain.resize(size)?;
    Ok(size)
  }

  /// The click-through flag lives on the child window: see
  /// [`window_proc::set_state`].
  pub(super) fn set_click_through(&self, through: bool) {
    window_proc::set_state(self.child, self.display, through);
  }
}

impl Drop for Surface {
  fn drop(&mut self) {
    let _ = unsafe { DestroyWindow(self.child) };
  }
}
