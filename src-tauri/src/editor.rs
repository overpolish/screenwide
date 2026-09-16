// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

pub(crate) mod annotations;
mod artifact;
mod artifact_snapshot;
mod audio_save;
mod camera_save;
pub(crate) mod commands;
pub(crate) mod cursor_effects;
mod cursor_export;
mod directory;
pub(crate) mod effect_animation;
pub(crate) mod export_window;
pub(crate) mod keyboard_effects;
mod media_preview;
mod naming;
mod preferences;
pub(crate) mod preview;
pub(crate) mod preview_platform;
mod preview_workspace_model;
pub(crate) mod recording_preview;
pub(crate) mod recording_preview_player;
mod recording_sidecar;
mod recovery;
pub(crate) mod save;
pub(crate) mod screenshot_preview;
mod timeline_edit;
mod track_selection;
mod validation;
mod workspace;

pub use artifact::{discard, present_recording, present_screenshot};
#[path = "editor/recording_model.rs"]
mod recording_model;
#[path = "editor/screenshot_composition.rs"]
mod screenshot_composition;
pub use recording_model::{
  AudioTrackKind, AudioTrackVolume, CameraOverlaySettings, RecordingAudioTrack, RecordingCamera,
  RecordingExportOptions, RecordingOutputSettings,
};

#[path = "editor/screenshot_model.rs"]
mod screenshot_model;
pub use screenshot_model::{ScreenshotItem, ScreenshotWorkspaceOutputSettings};
#[path = "editor/workspace_kind.rs"]
mod workspace_kind;
use workspace_kind::kind_of_window;
pub use workspace_kind::EditorKind;

use screenshot_composition::compose_screenshot_workspace;

use annotations::Annotation;
use artifact::{emit_snapshot, snapshots, take_artifact};
use artifact_snapshot::snapshot;
pub use artifact_snapshot::EditorArtifactSnapshot;
use camera_save::validate_camera_overlay;
use commands::store_export_directory;
use directory::current_directory;
pub use export_window::hide as hide_export_options_for;
#[cfg(target_os = "windows")]
pub(crate) use media_preview::ffmpeg_path;
use naming::sanitize_file_stem;
use preferences::{
  load_cursor_effects, load_recording_output, load_screenshot_background_radius,
  load_screenshot_output, load_screenshot_radius, remember_completed_export,
  remember_screenshot_background_radius, remember_screenshot_output, remember_screenshot_radius,
};
use recording_sidecar::{RecordingCursor, RecordingKeyboard};
pub use recovery::initialize;
#[cfg(test)]
use recovery::orphan_plan;
use save::{delivered_extension, scale_percent};
use validation::{validate_camera_resolution_scale, validate_primary_resolution_scale};
pub use workspace::has_pending_kind as has_pending_workspace_kind;
pub use workspace::{
  focus_if_pending as focus_pending_workspace,
  focus_if_screenshot_blocked as focus_if_screenshot_workspace_blocked,
  has_pending as has_pending_workspace, release_recording as release_recording_workspace,
  release_screenshot as release_screenshot_workspace,
  reserve_recording as reserve_recording_workspace,
  reserve_screenshot as reserve_screenshot_workspace,
};
mod window;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use tauri::{image::Image, AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::recording::{FinalizeInfo, PrimaryRecordingKind};
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use crate::screenshots::compose_screenshot;
use crate::screenshots::{
  encode_png, screenshot_directory, unique_path, CapturedImage, ScreenshotOutputSettings,
};

#[cfg(target_os = "macos")]
pub(crate) fn initialize_cursor_artwork() {
  cursor_effects::initialize_artwork();
}

const EDITOR_CHANGED_EVENT: &str = "editor://artifact";
const EXPORT_PROGRESS_EVENT: &str = "export://progress";
const EDITOR_PREFERENCES_FILE: &str = "editor-preferences.json";
const SCREENSHOT_EXTENSION: &str = "png";
/// What a saved recording is delivered as when it can be, which is whenever
/// FFmpeg is on the machine. See [`save_recording`] for the other case.
const RECORDING_EXTENSION: &str = "mp4";
const AUDIO_EXTENSION: &str = "m4a";
/// The container a recording is written to while it runs. macOS writes a
/// fragmented QuickTime movie; Windows writes fragmented MP4. Both retain
/// completed fragments if the app dies mid-recording.
const WORKING_RECORDING_EXTENSION: &str = if cfg!(windows) { "mp4" } else { "mov" };
/// Every extension a working recording can be found under in the recordings
/// directory. `.mp4` is there for the files an earlier version of the app left
/// behind: an upgrade must not walk past someone's unsaved recording.
const WORKING_RECORDING_EXTENSIONS: &[&str] = &["mov", "mp4"];
/// How long an unclaimed recording is kept before it is swept away. Long
/// enough that a crash is recoverable, short enough that a forgotten one does
/// not sit in the app's data directory forever.
const ORPHAN_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const MAX_FILE_STEM: usize = 200;

/// A capture waiting to be saved.
///
/// The window renders itself by artifact kind rather than assuming a
/// screenshot, because a recording is a file on disk rather than pixels in
/// memory and almost nothing about handling it is the same.
pub enum EditorArtifact {
  Screenshot {
    /// Unique per capture. Two consecutive fullscreen captures are identical
    /// in every other respect, so the window needs this to tell them apart
    /// and start the new one at fit rather than inheriting the old zoom.
    id: u64,
    /// Ordered back-to-front. The first slice keeps the existing single-item
    /// compositor contract while the native scene renderer is introduced.
    items: Vec<ScreenshotItem>,
    suggested_file_stem: String,
  },
  Recording {
    audio_tracks: Vec<RecordingAudioTrack>,
    camera: Option<RecordingCamera>,
    cursor: Option<RecordingCursor>,
    keyboard: Option<RecordingKeyboard>,
    id: u64,
    duration_ms: u64,
    height: u32,
    /// The working file. Saving moves it or derives the requested compressed
    /// copy; discarding deletes it.
    path: PathBuf,
    primary_kind: PrimaryRecordingKind,
    source_scale_percent: u16,
    suggested_file_stem: String,
    width: u32,
  },
}

fn recording_audio_tracks(
  has_system_audio: bool,
  has_microphone: bool,
) -> Vec<RecordingAudioTrack> {
  let mut tracks = Vec::with_capacity(usize::from(has_system_audio) + usize::from(has_microphone));
  if has_system_audio {
    tracks.push(RecordingAudioTrack {
      kind: AudioTrackKind::SystemAudio,
      label: "System audio".to_owned(),
      stream_index: tracks.len(),
    });
  }
  if has_microphone {
    tracks.push(RecordingAudioTrack {
      kind: AudioTrackKind::Microphone,
      label: "Microphone".to_owned(),
      stream_index: tracks.len(),
    });
  }
  tracks
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSnapshot {
  pub artifact: Option<EditorArtifactSnapshot>,
  pub cursor_effects: cursor_effects::CursorEffectSettings,
  pub directory: Option<PathBuf>,
  pub recording_output: Option<RecordingOutputSettings>,
  pub screenshot_radius_percent: f64,
  pub screenshot_background_radius_percent: f64,
  pub screenshot_output: Option<ScreenshotOutputSettings>,
  /// Which workspace this describes. The change event is app-wide because the
  /// recording bar listens to it too, so every receiver needs to know which of
  /// its snapshots the payload replaces.
  pub workspace: EditorKind,
}

/// Every workspace at once, for a webview that has just come up and has no
/// event history to reconstruct them from.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSnapshots {
  pub recording: EditorSnapshot,
  pub screenshot: EditorSnapshot,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportProgress {
  artifact_id: u64,
  phase: &'static str,
  progress_percent: f64,
}

#[derive(Clone)]
struct ActiveExportJob {
  artifact_id: u64,
  cancelled: Arc<AtomicBool>,
}

/// One editor workspace's own state: what is waiting in it, where it will be
/// saved, and the save running from it. Kept as separate mutexes per field, as
/// the rest of this state is, so a long-running save never blocks a snapshot.
#[derive(Default)]
struct EditorWorkspaceSlot {
  active_export: Mutex<Option<ActiveExportJob>>,
  artifact: Mutex<Option<EditorArtifact>>,
  directory: Mutex<Option<PathBuf>>,
}

#[derive(Default)]
pub struct EditorState {
  recording: EditorWorkspaceSlot,
  screenshot: EditorWorkspaceSlot,
  /// A capture being set up. Any in-flight capture blocks any other, of either
  /// kind, because the machine can only point its camera at one thing at once.
  capture_reservation: Mutex<Option<EditorKind>>,
  generation: AtomicU64,
  cursor_effects: Mutex<cursor_effects::CursorEffectSettings>,
  recording_output: Mutex<Option<RecordingOutputSettings>>,
  screenshot_radius_percent: Mutex<f64>,
  screenshot_background_radius_percent: Mutex<f64>,
  screenshot_output: Mutex<Option<ScreenshotOutputSettings>>,
  recording_preview: Mutex<Option<media_preview::RecordingPreview>>,
  recording_preview_preparation: Mutex<()>,
  /// Cached by artifact, stream kind (screen/camera/baked), quality and scale.
  compression_estimates: Mutex<HashMap<(u64, u8, u8, u16), u64>>,
  compression_estimate_preparation: Mutex<()>,
}

impl EditorState {
  fn slot(&self, kind: EditorKind) -> &EditorWorkspaceSlot {
    match kind {
      EditorKind::Recording => &self.recording,
      EditorKind::Screenshot => &self.screenshot,
    }
  }
}

#[cfg(test)]
mod tests;
