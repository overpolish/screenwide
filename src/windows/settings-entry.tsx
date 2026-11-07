// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelSync } from "../features/popup-panel/popup-panel-sync";
import { RecordingInputSync } from "../features/recording-inputs/recording-input-sync";
import { SettingsWindow } from "../features/settings/settings-window";

/** The window, with the stores it reads kept in step with Rust and the other
 * windows. Built for a paid tool that was turned away, it opens on Licences. */
export function SettingsEntry() {
  const opensOnLicences = new URLSearchParams(window.location.search).has(
    "licences",
  );
  return (
    <>
      <PopupPanelSync />
      <RecordingInputSync />
      <SettingsWindow
        initialSection={opensOnLicences ? "licences" : undefined}
      />
    </>
  );
}
