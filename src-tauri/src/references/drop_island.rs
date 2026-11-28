// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The island while a picture is dragged with Option held: it comes out at
//! the top of the screen to say that letting go keeps the picture as a
//! reference (`drop`), and goes back to whatever else holds it as the drag
//! ends or Option is let go. The hint says the same every time, so the
//! claim is all there is to it.

use tauri::AppHandle;

use crate::app_windows::island::{self, IslandClaim};

/// A picture is dragged with Option held, so letting go keeps it.
pub(crate) fn show(app: &AppHandle) {
  island::claim(app, IslandClaim::ReferenceDrop);
}

/// The drag ended or Option was let go.
pub(crate) fn withdraw(app: &AppHandle) {
  island::release(app, IslandClaim::ReferenceDrop);
}
