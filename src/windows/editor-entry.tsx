// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorSync } from "../features/editor/editor-sync";
import { EditorWindow } from "../features/editor/editor-window";
import { ExportOptionsSync } from "../features/editor/export/options-window/export-options-sync";
import { ToolPanelSync } from "../features/editor/tool-panels/tool-panel-sync";
import { PopupPanelSync } from "../features/popup-panel/popup-panel-sync";

/** The window, with the stores it reads kept in step with Rust and the other windows. */
export function EditorEntry() {
  return (
    <>
      <EditorSync />
      <ExportOptionsSync />
      <ToolPanelSync />
      <PopupPanelSync />
      <EditorWindow />
    </>
  );
}
