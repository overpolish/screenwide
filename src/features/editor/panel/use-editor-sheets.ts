// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback } from "react";

import { hideExportOptions } from "../export/options-window/api";
import { useExportOptionsOpen } from "../export/options-window/use-export-options-open";
import { useSheetEscape } from "../shortcuts/sheet-escape";
import { EditorKind } from "../types";

/**
 * The sheet that stands over the editor: its export options. It is modal to
 * the editor, and Escape closes it from the editor window or its tool panel
 * as well as from the sheet itself.
 */
export function useEditorSheets(workspace: EditorKind, isSaving: boolean) {
  const { isExportOpen, open: openExportOptions } =
    useExportOptionsOpen(workspace);
  const dismiss = useCallback(() => {
    // A running save keeps the sheet up, as it does for Escape pressed
    // inside the sheet.
    if (isSaving) return;
    hideExportOptions().catch((cause: unknown) => {
      console.error("Could not close the sheet", cause);
    });
  }, [isSaving]);
  useSheetEscape(isExportOpen ? dismiss : null);
  return { isSheetOpen: isExportOpen, openExportOptions };
}
