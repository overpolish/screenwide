// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The bindings Screenwide ships with, and how a saved file is laid over them.

use super::*;

impl Default for ShortcutSettings {
  fn default() -> Self {
    Self {
      bindings: vec![
        ShortcutBinding {
          action: ShortcutAction::ToggleRecordingBar,
          shortcut: Some("CommandOrControl+Shift+Digit6".to_owned()),
        },
        ShortcutBinding {
          action: ShortcutAction::StartStopRecording,
          shortcut: None,
        },
        ShortcutBinding {
          action: ShortcutAction::PauseResumeRecording,
          shortcut: None,
        },
        ShortcutBinding {
          action: ShortcutAction::SaveReplay,
          shortcut: Some("CommandOrControl+Shift+Digit9".to_owned()),
        },
        ShortcutBinding {
          action: ShortcutAction::TakeScreenshot,
          shortcut: Some("CommandOrControl+Shift+Digit8".to_owned()),
        },
        ShortcutBinding {
          action: ShortcutAction::TakeScreenshotToClipboard,
          shortcut: None,
        },
        ShortcutBinding {
          action: ShortcutAction::RecognizeText,
          shortcut: Some("CommandOrControl+Shift+KeyT".to_owned()),
        },
        ShortcutBinding {
          action: ShortcutAction::RulerOverlay,
          shortcut: Some("CommandOrControl+Shift+KeyR".to_owned()),
        },
        ShortcutBinding {
          action: ShortcutAction::AnnotateOverlay,
          shortcut: Some("CommandOrControl+Shift+KeyA".to_owned()),
        },
        ShortcutBinding {
          action: ShortcutAction::AnnotateClear,
          shortcut: Some("CommandOrControl+Shift+Backspace".to_owned()),
        },
      ],
    }
  }
}

/// The saved bindings over the defaults. An action the saved file has never
/// heard of keeps its default, so a shortcut introduced by an update reaches
/// an existing install; one the user cleared is stored as null and stays
/// cleared. A new default the user has already put to another use is left
/// unbound rather than taking the keys from under them.
pub(super) fn merge(stored: Option<ShortcutSettings>) -> ShortcutSettings {
  let mut settings = ShortcutSettings::default();
  let Some(stored) = stored else {
    return settings;
  };
  let mut introduced = Vec::new();
  for binding in &mut settings.bindings {
    match stored
      .bindings
      .iter()
      .find(|candidate| candidate.action == binding.action)
    {
      Some(saved) => binding.shortcut = saved.shortcut.clone(),
      None => introduced.push(binding.action),
    }
  }
  let keys = |shortcut: &Option<String>| {
    shortcut
      .as_deref()
      .and_then(|shortcut| shortcut.parse::<Shortcut>().ok())
  };
  for action in introduced {
    let default = settings
      .bindings
      .iter()
      .find(|binding| binding.action == action)
      .and_then(|binding| keys(&binding.shortcut));
    let taken = default.is_some_and(|default| {
      settings
        .bindings
        .iter()
        .any(|binding| binding.action != action && keys(&binding.shortcut) == Some(default))
    });
    if taken {
      if let Some(binding) = settings
        .bindings
        .iter_mut()
        .find(|binding| binding.action == action)
      {
        binding.shortcut = None;
      }
    }
  }
  settings
}
