// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Which editor workspace something belongs to.
///
/// A recording and a screenshot are held apart, each in its own workspace with
/// its own window, so one can sit waiting for a decision while the other is
/// being made. The enum is what keys them; growing past two is a matter of
/// widening it and the slot lookup, not of unpicking the callers.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EditorKind {
  Recording,
  Screenshot,
}

impl EditorKind {
  pub const ALL: [Self; 2] = [Self::Recording, Self::Screenshot];

  pub const fn window_label(self) -> crate::windows::WindowLabel {
    match self {
      Self::Recording => crate::windows::WindowLabel::EditorRecording,
      Self::Screenshot => crate::windows::WindowLabel::EditorScreenshot,
    }
  }

  pub(super) fn from_window_label(label: &str) -> Option<Self> {
    Self::ALL
      .into_iter()
      .find(|kind| kind.window_label().as_str() == label)
  }

  pub(super) fn of(artifact: &EditorArtifact) -> Self {
    match artifact {
      EditorArtifact::Recording { .. } => Self::Recording,
      EditorArtifact::Screenshot { .. } => Self::Screenshot,
    }
  }
}

/// The workspace a command is addressed to, read off the window it came from.
///
/// Tauri injects the calling window, so the webview never has to name its own
/// workspace and no `invoke` carries an argument that could disagree with the
/// window it was sent from.
pub(super) fn kind_of_window(window: &tauri::WebviewWindow) -> Result<EditorKind, String> {
  EditorKind::from_window_label(window.label())
    .ok_or_else(|| "That window has no editor workspace".to_owned())
}
