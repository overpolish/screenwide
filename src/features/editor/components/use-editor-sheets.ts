// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback } from "react";

import { resolveConfirmSheet } from "../../confirm-sheet/api";
import { useConfirmSheetModal } from "../../confirm-sheet/use-confirm-sheet-modal";
import { hideExportOptions } from "../export-options/api";
import { useExportOptionsOpen } from "../export-options/use-export-options-open";
import { useSheetEscape } from "../sheet-escape";
import { EditorKind } from "../types";

/**
 * The sheets that stand over the editor: its export options, and a confirm
 * sheet such as the one closing an unsaved capture asks. Either is modal to
 * the editor, and Escape closes whichever is up from the editor window or its
 * tool panel as well as from the sheet itself.
 */
export function useEditorSheets(workspace: EditorKind, isSaving: boolean) {
  const { isExportOpen, open: openExportOptions } =
    useExportOptionsOpen(workspace);
  const isConfirmOpen = useConfirmSheetModal();
  const dismiss = useCallback(() => {
    // Escape answers a confirm sheet with its way out, never the action it
    // asks about. A running save keeps the export sheet up, as it does for
    // Escape pressed inside the sheet.
    const closing = isConfirmOpen
      ? resolveConfirmSheet(false)
      : isSaving
        ? null
        : hideExportOptions();
    closing?.catch((cause: unknown) => {
      console.error("Could not close the sheet", cause);
    });
  }, [isConfirmOpen, isSaving]);
  const isSheetOpen = isExportOpen || isConfirmOpen;
  useSheetEscape(isSheetOpen ? dismiss : null);
  return { isSheetOpen, openExportOptions };
}
