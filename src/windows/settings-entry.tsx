// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingInputSync } from "../features/recording-inputs/recording-input-sync";
import { SettingsWindow } from "../features/settings/settings-window";

/** The window, with the stores it reads kept in step with Rust and the other windows. */
export function SettingsEntry() {
  return (
    <>
      <RecordingInputSync />
      <SettingsWindow />
    </>
  );
}
