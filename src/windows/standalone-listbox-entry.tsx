// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToolPanelSync } from "../features/editor/tool-panels/tool-panel-sync";
import { PopupPanelSync } from "../features/popup-panel/popup-panel-sync";
import { PopupPanelWindow } from "../features/popup-panel/popup-panel-window";

/** The window, with the stores it reads kept in step with Rust and the other windows. */
export function StandaloneListboxEntry() {
  return (
    <>
      <ToolPanelSync />
      <PopupPanelSync />
      <PopupPanelWindow />
    </>
  );
}
