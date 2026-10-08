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
pub(crate) mod library;
mod listing;
mod manifest;
mod relocate;

pub(crate) use deleted::{delete, expire, listed as recently_deleted, restore, KEEP_MS};
pub(crate) use folder::{
  create, ensure_default_directory, is_cancelled, move_to_trash, preview_path, projects_directory,
  rename, scrub_strip_path, sweep_cancelled, NewProject, SCRUB_FRAME_WIDTH,
};
pub(crate) use listing::{folder_size, in_folder, summarize, ProjectSummary};
pub(crate) use manifest::{
  read, resolve, update, write, Manifest, ProjectKind, RecordingManifest, RecordingMedia,
  RecordingOrigin, ScreenshotLayer, ScreenshotManifest,
};
pub(crate) use relocate::{duplicate, move_into};

/// The extension of the manifest, and the file type the app opens.
pub(crate) const EXTENSION: &str = "screenwide";

#[cfg(test)]
mod relocate_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use tests::scratch_project;
