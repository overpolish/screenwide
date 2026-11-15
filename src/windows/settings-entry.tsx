// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PermissionSync } from "../features/permissions/permission-sync";
import { PopupPanelSync } from "../features/popup-panel/popup-panel-sync";
import { RecordingInputSync } from "../features/recording-inputs/recording-input-sync";
import { type SettingsSection } from "../features/settings/settings-sections";
import { SettingsWindow } from "../features/settings/settings-window";

/** Panes something outside Settings opens it on: Licences for a paid tool
 * that was turned away, Models for dictation that cannot start. */
const OPENED_ON: readonly SettingsSection[] = ["licences", "transcription"];

/** The window, with the stores it reads kept in step with Rust and the other
 * windows. Built for another window, it opens on the pane named in its
 * address. */
export function SettingsEntry() {
  const query = new URLSearchParams(window.location.search);
  return (
    <>
      <PermissionSync />
      <PopupPanelSync />
      <RecordingInputSync />
      <SettingsWindow
        initialSection={OPENED_ON.find((section) => query.has(section))}
      />
    </>
  );
}
