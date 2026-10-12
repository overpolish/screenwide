// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A Screenwide project: one folder holding a recording and everything made
//! from it, so it can be moved, kept on another drive or sent to someone and
//! still open.
//!
//! ```text
//! <title>/
//!   <title>.screenwide   the manifest a user opens: what was recorded, where
//!                        its media is, the edit, and the look: canvas,
//!                        cursor, keyboard overlay, camera bubble and tracks
//!   preview.png          the picture the project browser shows: the edit as
//!                        the editor composed it, or else a frame of the
//!                        movie or the audio's ribbon
//!   media/               the movie, camera, cursor and keyboard tracks
//!     pictures/          the background picture and the images' pictures,
//!                        each named by its contents
//! ```
//!
//! Media is named relative to the folder, so renaming or moving the folder
//! never breaks it. Capture writes straight into `media/`, and the manifest is
//! written as soon as the capture is running, so a project survives the app
//! dying mid-recording and opens like any other.

mod deleted;
#[cfg(test)]
mod deleted_tests;
mod folder;
pub mod library;
mod listing;
mod manifest;
mod relocate;

pub use deleted::{delete, expire, listed as recently_deleted, restore, KEEP_MS};
pub use folder::{
  create_in, is_cancelled, move_to_trash, preview_path, rename, scrub_strip_path, sweep_cancelled,
  NewProject, SCRUB_FRAME_WIDTH,
};
pub use listing::{folder_size, in_folder, summarize, ProjectSummary};
pub use manifest::{
  read, resolve, update, write, Manifest, PrimaryRecordingKind, ProjectKind, RecordingManifest,
  RecordingMedia, RecordingOrigin, ScreenshotLayer, ScreenshotManifest,
};
pub use relocate::{duplicate, move_into};

/// The extension of the manifest, and the file type the app opens.
pub const EXTENSION: &str = "screenwide";

#[cfg(test)]
mod relocate_tests;
#[cfg(any(test, feature = "test-support"))]
pub mod scratch;
#[cfg(test)]
mod tests;
