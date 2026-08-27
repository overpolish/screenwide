// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  state::VisualKey, state::EXIT_US, KeyboardCompositor, KeyboardOverlay, Shortcut, HOLD_US,
};
use crate::exports::timeline_edit::{DeletedKeyboardShortcutRange, KeyboardShortcutPositionRange};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeyboardTimelineItem {
  pub id: u64,
  pub start_ms: u64,
  pub end_ms: u64,
  pub label: String,
}

impl KeyboardCompositor {
  pub(crate) fn set_deleted_shortcuts(&self, ids: &[u64], ranges: &[DeletedKeyboardShortcutRange]) {
    {
      let mut deleted = self
        .deleted_shortcut_ids
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      deleted.clear();
      deleted.extend(ids.iter().copied());
      *self
        .deleted_shortcut_ranges
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = ranges.to_vec();
    }
    self.rebake();
  }

  pub(crate) fn set_shortcut_positions(&self, positions: &[KeyboardShortcutPositionRange]) {
    {
      *self
        .shortcut_positions
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = positions.to_vec();
    }
    self.rebake();
  }

  pub(super) fn apply_shortcut_position(
    &self,
    overlay: &mut KeyboardOverlay,
    visible: &[&VisualKey],
    position_ms: u64,
  ) {
    let position = visible
      .iter()
      .max_by_key(|visual| visual.enter_us)
      .map(|visual| visual.source_shortcut as u64)
      .and_then(|shortcut_id| {
        self
          .shortcut_positions
          .read()
          .unwrap_or_else(|poisoned| poisoned.into_inner())
          .iter()
          .find(|position| {
            position.shortcut_id == shortcut_id
              && position_ms >= position.start_ms
              && position_ms < position.end_ms
          })
          .map(|position| {
            (
              position.center_x as f32,
              position.center_y as f32,
              position.size_percent.map(|size| (size / 100.0) as f32),
            )
          })
      });
    if let Some((center_x, center_y, scale)) = position {
      overlay.center_x = center_x;
      overlay.center_y = center_y;
      if let Some(scale) = scale {
        overlay.scale = scale;
        overlay.requested_scale = scale;
      }
    }
  }

  /// Returns captured shortcut groups in reconstruction order, using the same
  /// parsed data consumed by preview and export.
  pub(crate) fn timeline_items(&self) -> Vec<KeyboardTimelineItem> {
    let baked = self
      .baked
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    self
      .shortcuts
      .iter()
      .enumerate()
      .filter(|(index, _)| {
        !self
          .deleted_shortcut_ids
          .read()
          .unwrap_or_else(|poisoned| poisoned.into_inner())
          .contains(&(*index as u64))
      })
      .filter_map(|(index, shortcut)| {
        let start_us = shortcut.keys.iter().map(|key| key.down_us).min()?;
        let last_key_us = shortcut
          .keys
          .iter()
          .map(|key| key.up_us.unwrap_or(key.down_us))
          .max()
          .unwrap_or(start_us);
        // Match VisualKey::visible_at: released keys remain visible through
        // the compositor's exit lifetime, so the lane ends when the artwork
        // is actually gone rather than at the physical key-up event.
        let end_us = baked
          .timeline
          .visuals
          .iter()
          .filter(|visual| visual.source_shortcut == index)
          .filter_map(|visual| {
            let artwork_end = visual
              .exit
              .map(|(exit_us, _)| exit_us.saturating_add(EXIT_US));
            artwork_end
              .into_iter()
              .chain(visual.layout_anchor_until_us)
              .max()
          })
          .max()
          .unwrap_or_else(|| last_key_us.saturating_add(HOLD_US).saturating_add(EXIT_US));
        Some(KeyboardTimelineItem {
          id: index as u64,
          start_ms: start_us / 1_000,
          end_ms: end_us / 1_000,
          label: shortcut_label(shortcut, self.legacy_modifier_expansion),
        })
      })
      .collect()
  }
}

fn shortcut_label(shortcut: &Shortcut, legacy_modifier_expansion: bool) -> String {
  let mut labels: Vec<String> = Vec::new();
  if legacy_modifier_expansion && shortcut.keys.len() == 1 {
    let key = &shortcut.keys[0];
    if !is_modifier_key(key.key_code) {
      for (bit, label) in [
        (1, "Command"),
        (2, "Control"),
        (4, "Option"),
        (8, "Shift"),
        (16, "fn"),
      ] {
        if key.modifier_mask & bit != 0 {
          labels.push(label.to_owned());
        }
      }
    }
  }
  labels.extend(shortcut.keys.iter().map(|key| key_label(key.key_code)));
  labels.join(" ")
}

fn is_modifier_key(code: u16) -> bool {
  matches!(code, 54..=56 | 58..=63)
}

/// Labels for macOS-normalized virtual keycodes used by both capture platforms.
fn key_label(code: u16) -> String {
  let label = match code {
    0 => "A",
    1 => "S",
    2 => "D",
    3 => "F",
    4 => "H",
    5 => "G",
    6 => "Z",
    7 => "X",
    8 => "C",
    9 => "V",
    11 => "B",
    12 => "Q",
    13 => "W",
    14 => "E",
    15 => "R",
    16 => "Y",
    17 => "T",
    18 => "1",
    19 => "2",
    20 => "3",
    21 => "4",
    22 => "6",
    23 => "5",
    24 => "=",
    25 => "9",
    26 => "7",
    27 => "−",
    28 => "8",
    29 => "0",
    30 => "]",
    31 => "O",
    32 => "U",
    33 => "[",
    34 => "I",
    35 => "P",
    36 | 76 => "Enter",
    37 => "L",
    38 => "J",
    39 => "'",
    40 => "K",
    41 => ";",
    42 => "\\",
    43 => ",",
    44 => "/",
    45 => "N",
    46 => "M",
    47 => ".",
    48 => "Tab",
    49 => "Space",
    50 => "`",
    51 => "Backspace",
    53 => "Esc",
    54 | 55 => "Command",
    56 | 60 => "Shift",
    57 => "Caps Lock",
    58 | 61 => "Option",
    59 | 62 => "Control",
    63 => "fn",
    65 => ".",
    67 => "*",
    69 => "+",
    71 => "Clear",
    75 => "/",
    78 => "−",
    81 => "=",
    82 => "0",
    83 => "1",
    84 => "2",
    85 => "3",
    86 => "4",
    87 => "5",
    88 => "6",
    89 => "7",
    91 => "8",
    92 => "9",
    96 => "F5",
    97 => "F6",
    98 => "F7",
    99 => "F3",
    100 => "F8",
    101 => "F9",
    103 => "F11",
    105 => "F13",
    106 => "F16",
    107 => "F14",
    109 => "F10",
    111 => "F12",
    113 => "F15",
    114 => "Insert",
    115 => "Home",
    116 => "Page Up",
    117 => "Delete",
    118 => "F4",
    119 => "End",
    120 => "F2",
    121 => "Page Down",
    122 => "F1",
    123 => "←",
    124 => "→",
    125 => "↓",
    126 => "↑",
    _ => return format!("Key {code}"),
  };
  label.to_owned()
}
