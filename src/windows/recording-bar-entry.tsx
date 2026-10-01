// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorSync } from "../features/editor/editor-sync";
import { PermissionSync } from "../features/permissions/permission-sync";
import { PopupPanelSync } from "../features/popup-panel/popup-panel-sync";
import { RecordingBarWindow } from "../features/recording-controls/recording-bar/recording-bar-window";
import { RecordingStateSync } from "../features/recording-controls/recording-state-sync";
import { RecordingInputSync } from "../features/recording-inputs/recording-input-sync";
import { RecordingSourceSync } from "../features/recording-sources/recording-source-sync";

/** The window, with the stores it reads kept in step with Rust and the other windows. */
export function RecordingBarEntry() {
  return (
    <>
      <EditorSync />
      <PermissionSync />
      <RecordingInputSync />
      <RecordingSourceSync />
      <RecordingStateSync />
      <PopupPanelSync />
      <RecordingBarWindow />
    </>
  );
}
