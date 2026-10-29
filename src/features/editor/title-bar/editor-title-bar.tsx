// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ClipboardCopy, Upload, Trash2 } from "lucide-react";
import { useCallback, useLayoutEffect, useRef } from "react";

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { WindowHeader } from "../../../composites/window-header/window-header";
import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";
import { EditorArtifact } from "../types";

export function EditorTitleBar({
  artifact,
  canExport,
  isSaving,
  onClose,
  onCopy,
  onDelete,
  onExport,
  onMinimize,
  onRename,
  onToggleMaximize,
}: {
  artifact: EditorArtifact | null;
  canExport: boolean;
  isSaving?: boolean;
  onClose?: () => void;
  onCopy?: () => void;
  /** Closes the editor and sets the project aside in Recently Deleted, for
   * a take that is not worth exporting. */
  onDelete?: () => void;
  onExport?: () => void;
  onMinimize?: () => void;
  /** Renames the project, which the title names. */
  onRename?: (title: string) => void;
  onToggleMaximize?: () => void;
}) {
  const canExportRef = useRef(canExport);
  const onExportRef = useRef(onExport);
  useLayoutEffect(() => {
    canExportRef.current = canExport;
    onExportRef.current = onExport;
  }, [canExport, onExport]);
  /** Opening the dialog from the shortcut still commits an in-progress rename. */
  const onShortcutExport = useCallback(() => {
    const activeElement = document.activeElement;
    if (
      activeElement instanceof HTMLElement &&
      activeElement.isContentEditable
    ) {
      activeElement.blur();
      requestAnimationFrame(() => {
        if (canExportRef.current) onExportRef.current?.();
      });
      return;
    }
    if (canExportRef.current) onExportRef.current?.();
  }, []);
  useEditorWindowShortcuts({
    onCopy: artifact?.kind === "screenshot" && !isSaving ? onCopy : undefined,
    onExport: canExport ? onShortcutExport : undefined,
  });

  return (
    <WindowHeader
      actions={
        <div className="flex shrink-0 items-center gap-inline">
          {onDelete ? (
            // Two presses: it sits beside Export, and the editor closes
            // behind it.
            <ConfirmActionButton
              confirmLabel={t("editor-confirm-delete")}
              icon={<Trash2 />}
              isDisabled={!artifact || isSaving}
              label={t("editor-delete")}
              onConfirm={onDelete}
              variant="ghost"
            />
          ) : null}
          {artifact?.kind === "screenshot" ? (
            <Button isDisabled={isSaving} onPress={onCopy} variant="ghost">
              <ClipboardCopy />
              {t("editor-copy")}
            </Button>
          ) : null}
          <Button color="primary" isDisabled={!canExport} onPress={onExport}>
            <Upload />
            {t("editor-export")}
          </Button>
        </div>
      }
      onClose={onClose}
      onMinimize={onMinimize}
      onTitleChange={artifact && !isSaving ? onRename : undefined}
      onToggleMaximize={onToggleMaximize}
      title={artifact?.suggestedFileStem ?? ""}
    />
  );
}
