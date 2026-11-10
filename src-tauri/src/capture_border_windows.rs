// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows draws a yellow border round every display or window a Graphics
//! Capture session reads, and every capture here asks for none
//! (`SetIsBorderRequired(false)`, in the capture crate and in `xcap`). Windows
//! honours that only once the user has allowed borderless capture, which the
//! package declares (`graphicsCaptureWithoutBorder` in `msix/AppxManifest.xml`)
//! and the app asks for here.

use windows::Graphics::Capture::{GraphicsCaptureAccess, GraphicsCaptureAccessKind};

/// Asks for borderless capture, on a thread of its own so launch never waits
/// on it. Windows asks the user the first time and answers from their choice
/// after that. Whatever the answer, capture goes on: declined, the border
/// shows. Windows without the API (Windows 10) fails the call, and so does a
/// build with no package to declare the capability; both keep the border.
pub fn request_borderless() {
  std::thread::spawn(|| {
    let _ = GraphicsCaptureAccess::RequestAccessAsync(GraphicsCaptureAccessKind::Borderless)
      .and_then(|request| request.join());
  });
}
