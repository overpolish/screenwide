// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The desktop behind window picking: each display frozen as picking starts,
//! and shown slightly blurred round the window being picked, which stays
//! live. The overlay keeps every display's picture for the surface covering
//! that display, however its surfaces are rebuilt as the pick moves between
//! displays.

use std::collections::HashMap;
use std::sync::Arc;

use screenwide_imaging::CapturedImage;
use tauri::{AppHandle, Manager, WebviewWindow};

use super::super::MonitorDetails;
use crate::app_windows::WindowLabel;

#[derive(Default)]
pub(super) struct Backdrops {
  images: HashMap<u32, Arc<CapturedImage>>,
}

impl Backdrops {
  /// Freezes every display. One that cannot be captured is picked over
  /// without a backdrop.
  pub(super) async fn capture(monitors: &[MonitorDetails]) -> Self {
    use crate::screenshots::{freeze_monitors, FrozenDesktop};
    let ids = monitors
      .iter()
      .map(|monitor| monitor.id)
      .collect::<Vec<_>>();
    let stills = freeze_monitors(&ids, FrozenDesktop::WithoutOwnWindows).await;
    let mut images = HashMap::new();
    for (id, still) in ids.into_iter().zip(stills) {
      match still {
        Ok(image) => {
          images.insert(id, Arc::new(image));
        }
        Err(error) => {
          eprintln!("Could not freeze display {id} behind window picking: {error}");
        }
      }
    }
    Self { images }
  }

  /// Every display's picture, to hand to the overlay.
  pub(super) fn images(&self) -> Vec<(u32, Arc<CapturedImage>)> {
    self
      .images
      .iter()
      .map(|(&id, image)| (id, Arc::clone(image)))
      .collect()
  }
}

/// Puts each display's picture behind the overlay, or with none lets them go,
/// saying whether the overlay took them.
pub(super) fn install(window: &WebviewWindow, images: &[(u32, Arc<CapturedImage>)]) -> bool {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
    let images = images
      .iter()
      .map(|(id, image)| (*id, image.rgba.as_slice(), image.width, image.height))
      .collect::<Vec<_>>();
    crate::app_windows::screenshot_region::region_osc::set_backdrops(window, &images)
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = (window, images);
    false
  }
}

/// Lets the picture go once the overlay has faded, unless picking has begun
/// again meanwhile.
pub(super) fn release(app: &AppHandle) {
  let handle = app.clone();
  let _ = app.run_on_main_thread(move || {
    if super::is_active() {
      return;
    }
    if let Some(window) = handle.get_webview_window(WindowLabel::RegionSelector.as_str()) {
      let _ = install(&window, &[]);
    }
  });
}
