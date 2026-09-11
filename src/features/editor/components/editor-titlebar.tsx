// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ClipboardCopy, Upload } from "lucide-react";
import { ReactNode, useCallback, useLayoutEffect, useRef } from "react";

import logoUrl from "../../../assets/screenwide-mark.svg";
import { Button } from "../../../components/base/button/button";
import { WindowHeader } from "../../../components/shared/window-header/window-header";
import { EditorArtifact } from "../types";
import { useEditorWindowShortcuts } from "../use-editor-window-shortcuts";

import { useEditorToolbarTools } from "./editor-toolbar-context";

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
  tools,
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
  /** The workspace's tools, carried in the bar. Defaults to what the visible
   * section is offering through `EditorToolbarContext`. */
  tools?: ReactNode;
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

  const sectionTools = useEditorToolbarTools();

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
      center={tools ?? sectionTools}
      leadingSection={
        <img
          alt="Screenwide"
          className="brightness-0 dark:invert"
          draggable={false}
          src={logoUrl}
        />
      }
      onClose={onClose}
      onMinimize={onMinimize}
      onTitleChange={artifact && !isSaving ? onFileStemChange : undefined}
      onToggleMaximize={onToggleMaximize}
      title={fileStem}
    />
  );
}
