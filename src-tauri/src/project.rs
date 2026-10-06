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

mod folder;
pub(crate) mod library;
mod listing;
mod manifest;

pub(crate) use folder::{
  create, is_cancelled, move_to_trash, preview_path, projects_directory, rename, sweep_cancelled,
  NewProject,
};
pub(crate) use listing::{in_folder, summarize, ProjectSummary};
pub(crate) use manifest::{
  read, resolve, update, write, Manifest, ProjectKind, RecordingManifest, RecordingMedia,
  RecordingOrigin, ScreenshotLayer, ScreenshotManifest,
};

/// The extension of the manifest, and the file type the app opens.
pub(crate) const EXTENSION: &str = "screenwide";

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use tests::scratch_project;
