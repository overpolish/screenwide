// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The bindings Screenwide ships with, and how a saved file is laid over them.

use super::*;

impl Default for ShortcutSettings {
  fn default() -> Self {
    Self {
      bindings: vec![
        ShortcutBinding {
          action: ShortcutAction::ToggleRecordingControls,
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
          shortcut: Some(SCREENSHOT_DEFAULT.to_owned()),
        },
        ShortcutBinding {
          action: ShortcutAction::TakeReferenceScreenshot,
          shortcut: Some("CommandOrControl+Shift+KeyD".to_owned()),
        },
        ShortcutBinding {
          action: ShortcutAction::RecognizeText,
          shortcut: Some("CommandOrControl+Shift+KeyE".to_owned()),
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
        ShortcutBinding {
          action: ShortcutAction::ToggleKeepAwake,
          shortcut: None,
        },
        ShortcutBinding {
          action: ShortcutAction::ToggleKeepScreenOn,
          shortcut: None,
        },
        ShortcutBinding {
          action: ShortcutAction::Dictate,
          shortcut: Some("CommandOrControl+Shift+Space".to_owned()),
        },
        ShortcutBinding {
          action: ShortcutAction::TakeNotes,
          shortcut: Some("CommandOrControl+Shift+Digit1".to_owned()),
        },
        // Left unset: calls own the keys their apps mute with, and a global
        // shortcut on one would take it from them.
        ShortcutBinding {
          action: ShortcutAction::ToggleMicrophoneMute,
          shortcut: None,
        },
      ],
      replace_system_keys: true,
      system_keys_announced: false,
    }
  }
}

/// Defaults an action shipped with before its current one. The settings
/// window saves the keys it records with their modifiers spelled out, so a
/// saved binding still holding one of these verbatim was never changed.
const RETIRED_DEFAULTS: &[(ShortcutAction, &str)] = &[
  (
    ShortcutAction::TakeScreenshot,
    "CommandOrControl+Shift+Digit8",
  ),
  (
    ShortcutAction::TakeReferenceScreenshot,
    "CommandOrControl+Shift+Digit7",
  ),
  (ShortcutAction::RecognizeText, "CommandOrControl+Shift+KeyT"),
];

fn is_retired_default(action: ShortcutAction, shortcut: Option<&str>) -> bool {
  RETIRED_DEFAULTS
    .iter()
    .any(|&(retired, keys)| retired == action && shortcut == Some(keys))
}

const SCREENSHOT_DEFAULT: &str = "CommandOrControl+Shift+Digit2";
/// Copy a screenshot's default, from before copying became part of Take a
/// screenshot.
const COPY_SCREENSHOT_DEFAULT: &str = "CommandOrControl+Shift+Digit1";

/// An action as a saved file names it, including one since merged into
/// another.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum SavedAction {
  /// Copy a screenshot, now part of Take a screenshot.
  TakeScreenshotToClipboard,
  #[serde(untagged)]
  Current(ShortcutAction),
}

#[derive(Deserialize)]
struct SavedBinding {
  action: SavedAction,
  shortcut: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedSettings {
  bindings: Vec<SavedBinding>,
  #[serde(default = "replaces_system_keys")]
  replace_system_keys: bool,
  #[serde(default)]
  system_keys_announced: bool,
}

/// Reads a saved file, which may still hold Copy a screenshot. Its keys
/// pass to Take a screenshot, which now copies too, when they were the ones
/// in use: Take a screenshot cleared, or left on its default while Copy a
/// screenshot's was changed. Keys the user gave Take a screenshot stay.
pub(super) fn saved_settings(contents: &[u8]) -> Option<ShortcutSettings> {
  let saved: SavedSettings = serde_json::from_slice(contents).ok()?;
  let mut copy = None;
  let mut bindings = Vec::with_capacity(saved.bindings.len());
  for binding in saved.bindings {
    match binding.action {
      SavedAction::Current(action) => bindings.push(ShortcutBinding {
        action,
        shortcut: binding.shortcut,
      }),
      SavedAction::TakeScreenshotToClipboard => copy = Some(binding.shortcut),
    }
  }
  if let Some(copy) = copy {
    let take = bindings
      .iter()
      .position(|binding| binding.action == ShortcutAction::TakeScreenshot);
    let take_keys = take.map(|index| bindings[index].shortcut.as_deref());
    let take_cleared = take_keys == Some(None);
    let take_left = take_keys.is_none_or(|keys| {
      keys == Some(SCREENSHOT_DEFAULT) || is_retired_default(ShortcutAction::TakeScreenshot, keys)
    });
    let copy_changed = copy
      .as_deref()
      .is_some_and(|keys| keys != COPY_SCREENSHOT_DEFAULT);
    if take_cleared || (take_left && copy_changed) {
      let inherited = ShortcutBinding {
        action: ShortcutAction::TakeScreenshot,
        shortcut: copy,
      };
      match take {
        Some(index) => bindings[index] = inherited,
        None => bindings.push(inherited),
      }
    }
  }
  Some(ShortcutSettings {
    bindings,
    replace_system_keys: saved.replace_system_keys,
    system_keys_announced: saved.system_keys_announced,
  })
}

/// The saved bindings over the defaults. An action the saved file has never
/// heard of keeps its default, so a shortcut introduced by an update reaches
/// an existing install, and one still on a default since retired moves to
/// the current one. One the user changed stays, and one they cleared is
/// stored as null and stays cleared. A default the user has already put to
/// another use does not take the keys from under them: the action keeps
/// what it had, or stays unbound if it is new.
pub(super) fn merge(stored: Option<ShortcutSettings>) -> ShortcutSettings {
  let mut settings = ShortcutSettings::default();
  let Some(stored) = stored else {
    return settings;
  };
  settings.replace_system_keys = stored.replace_system_keys;
  settings.system_keys_announced = stored.system_keys_announced;
  let mut arriving = Vec::new();
  for binding in &mut settings.bindings {
    match stored
      .bindings
      .iter()
      .find(|candidate| candidate.action == binding.action)
    {
      Some(saved) if is_retired_default(binding.action, saved.shortcut.as_deref()) => {
        arriving.push((binding.action, saved.shortcut.clone()));
      }
      Some(saved) => binding.shortcut = saved.shortcut.clone(),
      None => arriving.push((binding.action, None)),
    }
  }
  let keys = |shortcut: &Option<String>| {
    shortcut
      .as_deref()
      .and_then(|shortcut| shortcut.parse::<Shortcut>().ok())
  };
  for (action, kept) in arriving {
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
        binding.shortcut = kept;
      }
    }
  }
  settings
}
