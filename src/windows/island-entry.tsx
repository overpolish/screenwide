// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandWindow } from "../features/island/island-window";
import { PermissionSync } from "../features/permissions/permission-sync";
import { PopupPanelSync } from "../features/popup-panel/popup-panel-sync";
import { RecordingIslandControls } from "../features/recording-controls/island/recording-island-controls";
import { RecordingStateSync } from "../features/recording-controls/recording-state-sync";
import { RecordingInputSync } from "../features/recording-inputs/recording-input-sync";
import { RecordingSourceSync } from "../features/recording-sources/recording-source-sync";

/** The island's window, showing the recording controls, with the stores they
 * read kept in step with Rust and the other windows. */
export function IslandEntry() {
  return (
    <>
      <PermissionSync />
      <PopupPanelSync />
      <RecordingInputSync />
      <RecordingSourceSync />
      <RecordingStateSync />
      <IslandWindow>
        {(island) => <RecordingIslandControls island={island} />}
      </IslandWindow>
    </>
  );
}
