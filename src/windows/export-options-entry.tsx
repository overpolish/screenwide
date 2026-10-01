// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorSync } from "../features/editor/editor-sync";
import { ExportOptionsSync } from "../features/editor/export/options-window/export-options-sync";
import { ExportOptionsWindow } from "../features/editor/export/options-window/export-options-window";
import { PopupPanelSync } from "../features/popup-panel/popup-panel-sync";

/** The window, with the stores it reads kept in step with Rust and the other windows. */
export function ExportOptionsEntry() {
  return (
    <>
      <EditorSync />
      <ExportOptionsSync />
      <PopupPanelSync />
      <ExportOptionsWindow />
    </>
  );
}
