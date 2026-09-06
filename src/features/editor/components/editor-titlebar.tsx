// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, ClipboardCopy, Upload, X } from "lucide-react";
import { useCallback, useLayoutEffect, useRef } from "react";

import logoUrl from "../../../assets/screenwide-mark.svg";
import { Button } from "../../../components/base/button/button";
import { ConfirmActionButton } from "../../../components/shared/confirm-action-button/confirm-action-button";
import { WindowHeader } from "../../../components/shared/window-header/window-header";
import { EditorArtifact } from "../types";
import { useEditorWindowShortcuts } from "../use-editor-window-shortcuts";

export function EditorTitlebar({
  artifact,
  canExport,
  fileStem,
  isSaving,
  onClose,
  onCopy,
  onExport,
  onFileStemChange,
  onMinimize,
  onToggleMaximize,
}: {
  artifact: EditorArtifact | null;
  canExport: boolean;
  fileStem: string;
  isSaving?: boolean;
  onClose?: () => void;
  onCopy?: () => void;
  onExport?: () => void;
  onFileStemChange?: (fileStem: string) => void;
  onMinimize?: () => void;
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
        <div className="gap-control flex shrink-0 items-center">
          {artifact?.kind === "screenshot" ? (
            <Button isDisabled={isSaving} onPress={onCopy} variant="ghost">
              <ClipboardCopy />
              Copy
            </Button>
          ) : null}
          <Button color="primary" isDisabled={!canExport} onPress={onExport}>
            <Upload />
            Export
          </Button>
        </div>
      }
      closeAction={
        <ConfirmActionButton
          armedClassName="bg-error-surface text-error data-[hovered]:bg-error-surface-hover data-[pressed]:bg-error-surface-pressed"
          armedIcon={<Check />}
          armedLabel="Confirm deleting capture"
          idleIcon={<X />}
          idleLabel="Close"
          key={artifact?.id ?? "empty"}
          onConfirm={onClose}
          size="compact"
        />
      }
      leadingSection={
        <img
          alt="Screenwide"
          className="brightness-0 dark:invert"
          draggable={false}
          src={logoUrl}
        />
      }
      onMinimize={onMinimize}
      onTitleChange={artifact && !isSaving ? onFileStemChange : undefined}
      onToggleMaximize={onToggleMaximize}
      title={fileStem}
    />
  );
}
