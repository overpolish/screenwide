// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures arriving for images: dropped on a workspace or chosen in the
//! import dialog. The store they go into is `screenwide_images`.

/// Reading a picture out of what a drag carries on Windows, for the
/// workspace and for references.
#[cfg(target_os = "windows")]
pub(crate) mod dragged_windows;
/// A picture dropped on the workspace, on its way to becoming an image.
pub(crate) mod dropped;
/// Bringing a picture in from a file or the clipboard.
pub mod import;
