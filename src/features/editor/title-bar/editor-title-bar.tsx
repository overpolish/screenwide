// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ClipboardCopy, ScrollText, Upload, Trash2 } from "lucide-react";
import { type ReactNode, useCallback, useLayoutEffect, useRef } from "react";

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { WindowHeader } from "../../../composites/window-header/window-header";
import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { useAddCueShortcut } from "../../teleprompter/cues/editor/use-add-cue-shortcut";
import { addEditorCue } from "../../teleprompter/cues/editor/use-editor-cue";
import { useAnnotationSelectionCount } from "../annotations/annotation-channel";
import { useRecordingFrameCopy } from "../recording/use-copy-recording-frame";
import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";
import { EditorArtifact } from "../types";

export function EditorTitleBar({
  artifact,
  canExport,
  exportProgress,
  isExportOpen = false,
  isSaving,
  modes,
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
  /** The save's whole percent while it runs (null while it reports none),
   * drawn as a small ring on Export so it shows with the island away. */
  exportProgress?: number | null;
  /** The Export island is out: the button stays pressed. */
  isExportOpen?: boolean;
  isSaving?: boolean;
  /** The editor's modes, centred in the bar (`EditorModeBar`). */
  modes?: ReactNode;
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
  // A screenshot copies itself; a recording, the frame under the playhead,
  // which only its preview can compose (`useCopyRecordingFrame`).
  const copyFrame = useRecordingFrameCopy((state) => state.copy);
  const copy =
    artifact?.kind === "screenshot"
      ? onCopy
      : artifact?.kind === "recording" && copyFrame
        ? () => void copyFrame()
        : undefined;
  // With annotations chosen, ⌘C copies them instead (`annotation-clipboard`).
  const chosenAnnotations = useAnnotationSelectionCount(
    artifact?.kind === "recording" ? "recording" : "screenshot",
  );
  useEditorWindowShortcuts({
    onCopy: isSaving || chosenAnnotations > 0 ? undefined : copy,
    onExport: canExport ? onShortcutExport : undefined,
  });
  // The open project, added to a script as a cue: a recording's frame under
  // the playhead, or the screenshot (`use-editor-cue.ts`).
  const addCue =
    artifact && !isSaving
      ? () => {
          addEditorCue(artifact.kind, artifact.id).catch((error: unknown) => {
            console.error("Could not add the cue", error);
          });
        }
      : undefined;
  useAddCueShortcut(addCue);

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
          {copy ? (
            <Button isDisabled={isSaving} onPress={copy} variant="ghost">
              <ClipboardCopy />
              {artifact?.kind === "recording"
                ? t("editor-copy-frame")
                : t("editor-copy")}
            </Button>
          ) : null}
          {artifact?.kind === "screenshot" ? (
            <Button isDisabled={isSaving} onPress={addCue} variant="ghost">
              <ScrollText />
              {t("editor-add-to-script")}
            </Button>
          ) : null}
          <Button
            aria-pressed={isExportOpen}
            className={isExportOpen ? "primary-raised-pressed" : undefined}
            color="primary"
            // A press on it toggles the Export island, rather than being
            // a press off the island that puts it away.
            data-export-toggle
            isDisabled={!canExport}
            onPress={onExport}
          >
            {exportProgress === undefined ? (
              <Upload />
            ) : (
              <CircularProgress
                aria-label={t("editor-export-progress")}
                color="on-primary"
                isIndeterminate={exportProgress === null}
                size="small"
                value={exportProgress ?? 0}
              />
            )}
            {t("editor-export")}
          </Button>
        </div>
      }
      center={modes}
      onClose={onClose}
      onMinimize={onMinimize}
      onTitleChange={artifact && !isSaving ? onRename : undefined}
      onToggleMaximize={onToggleMaximize}
      title={artifact?.suggestedFileStem ?? ""}
    />
  );
}
