// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ClipboardCopy,
  Folder,
  Minus,
  Square,
  Trash2,
  Upload,
  X,
} from "lucide-react";
import { Input, TextField } from "react-aria-components";

import logoUrl from "../../../assets/screenwide-mark.svg";
import { Button } from "../../../components/base/button/button";
import { IconButton } from "../../../components/base/button/icon-button";
import { inputFieldVariants } from "../../../components/base/input-fields/input-field";
import { ConfirmActionButton } from "../../../components/shared/confirm-action-button/confirm-action-button";
import { truncateDirectoryPath } from "../directory-path";
import { ExportArtifact } from "../types";
import { useExportWindowShortcuts } from "../use-export-window-shortcuts";

const directoryLabel = (directory: string | null) =>
  directory ? truncateDirectoryPath(directory) : "Choose folder";

export function ExportTitlebar({
  artifact,
  directory,
  extension,
  fileStem,
  hasExportableContent,
  isExportPreparationPending,
  isSaving,
  onBrowse,
  onClose,
  onCopy,
  onExport,
  onFileStemChange,
  onMinimize,
  onToggleMaximize,
}: {
  artifact: ExportArtifact | null;
  directory: string | null;
  fileStem: string;
  hasExportableContent: boolean;
  extension?: string;
  isExportPreparationPending?: boolean;
  isSaving?: boolean;
  onBrowse?: () => void;
  onClose?: () => void;
  onCopy?: () => void;
  onExport?: () => void;
  onFileStemChange?: (fileStem: string) => void;
  onMinimize?: () => void;
  onToggleMaximize?: () => void;
}) {
  const styles = inputFieldVariants({ size: "md", variant: "ghost" });
  const canExport =
    Boolean(artifact) &&
    hasExportableContent &&
    fileStem.trim().length > 0 &&
    !isExportPreparationPending &&
    !isSaving;
  useExportWindowShortcuts({
    onCopy: artifact?.kind === "screenshot" && !isSaving ? onCopy : undefined,
    onExport: canExport ? onExport : undefined,
  });

  return (
    <header
      className="flex h-12 min-w-0 shrink-0 items-center gap-2 border-b border-muted/15 px-3"
      data-tauri-drag-region="deep"
    >
      <img
        alt="Screenwide"
        className="pointer-events-none size-5 shrink-0 brightness-0 dark:invert"
        data-tauri-drag-region
        draggable={false}
        src={logoUrl}
      />

      <TextField
        aria-label="File name"
        className="min-w-40 max-w-88 grow"
        isDisabled={!artifact || isSaving}
        onChange={onFileStemChange}
        value={fileStem}
      >
        <div className={styles.field()}>
          <div className={styles.inputWrapper()}>
            <Input
              className={styles.input()}
              onKeyDown={(event) => {
                if (event.key !== "Enter" && event.key !== "Escape") return;
                event.preventDefault();
                event.stopPropagation();
                event.currentTarget.blur();
              }}
            />
            <span className="shrink-0 text-xs text-muted">
              .{extension ?? artifact?.extension ?? "png"}
            </span>
          </div>
        </div>
      </TextField>

      <Button
        aria-label={
          directory ? `Choose export folder, currently ${directory}` : undefined
        }
        className="min-w-0 max-w-56 shrink"
        isDisabled={isSaving}
        onPress={onBrowse}
        size="compact"
      >
        <Folder className="shrink-0" size={15} />
        <span className="truncate">{directoryLabel(directory)}</span>
      </Button>

      <div className="min-w-4 grow" data-tauri-drag-region />

      {artifact?.kind === "screenshot" ? (
        <Button
          isDisabled={isSaving}
          onPress={onCopy}
          size="compact"
          variant="ghost"
        >
          <ClipboardCopy size={15} />
          Copy
        </Button>
      ) : null}
      <Button
        color="primary"
        isDisabled={!canExport}
        onPress={onExport}
        size="compact"
      >
        <Upload size={15} />
        Export
      </Button>
      <IconButton
        aria-label="Minimize"
        className="group"
        onPress={onMinimize}
        size="compact"
      >
        <Minus
          className="transform-gpu text-muted transition-[color,transform,scale] group-data-[hovered]:scale-110 group-data-[hovered]:text-content-fg"
          size={18}
        />
      </IconButton>
      <IconButton
        aria-label="Maximize or restore"
        className="group"
        onPress={onToggleMaximize}
        size="compact"
      >
        <Square
          className="transform-gpu text-muted transition-[color,transform,scale] group-data-[hovered]:scale-110 group-data-[hovered]:text-content-fg"
          size={14}
        />
      </IconButton>
      <ConfirmActionButton
        armedIcon={<Trash2 className="text-error" size={18} />}
        armedLabel="Confirm deleting capture"
        idleIcon={<X size={18} />}
        idleLabel="Close"
        key={artifact?.id ?? "empty"}
        onConfirm={onClose}
        size="compact"
      />
    </header>
  );
}
