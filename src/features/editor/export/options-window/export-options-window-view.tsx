// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, Ref } from "react";

import { WindowHeader } from "../../../../composites/window-header/window-header";
import { WindowShell } from "../../../../composites/window-shell/window-shell";
import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { EditorExportForm } from "../editor-export-form";
import { EditorExportFormProps } from "../editor-export-form-props";

import { ExportProgress, ExportProgressProps } from "./export-progress";

export type ExportOptionsWindowViewProps = {
  /** The export form. It stays in place, disabled, while a save runs. */
  form: EditorExportFormProps;
  /** Covers the form with the running save and takes the window's way out. */
  isSaving: boolean;
  onClose: () => void;
  /** Measured by the window so it can fit itself to the form. */
  contentRef?: Ref<HTMLElement>;
  /** Set beside the form, on its start side: the recording's preview. */
  preview?: ReactNode;
  /** The running save. Required in practice whenever `isSaving`. */
  progress?: ExportProgressProps;
};

/**
 * The export options window as it looks: the only place its shell exists.
 *
 * It owns nothing but the arrangement, so the real window and the stories
 * present the same thing and cannot drift apart.
 */
export function ExportOptionsWindowView({
  contentRef,
  form,
  isSaving,
  onClose,
  preview,
  progress,
}: ExportOptionsWindowViewProps) {
  return (
    <WindowShell
      className="relative"
      header={
        <WindowHeader
          isDraggable={false}
          // The save is watched from here, so there is no way out until it ends.
          onClose={isSaving ? undefined : onClose}
          title={t("editor-export")}
        />
      }
      height="content"
      ref={contentRef}
    >
      {/* While saving, the sheet shows its progress in place of the form, as
          a native sheet doing work does. The form stays mounted but hidden so
          the window keeps its size and cancelling lands back on it. */}
      <div className="relative">
        <div
          className={cn(
            "flex gap-layout px-window-inset pb-window-inset",
            isSaving && progress !== undefined && "invisible",
          )}
        >
          {preview ? (
            <div className="flex min-w-0 flex-1">{preview}</div>
          ) : null}
          <div
            className={cn(
              "flex min-w-0 flex-1 flex-col",
              preview !== undefined && "w-export-form flex-none",
            )}
          >
            <EditorExportForm {...form} isSaving={isSaving} />
          </div>
        </div>
        {isSaving && progress ? (
          <div className="absolute inset-0 flex items-center px-window-inset pb-window-inset">
            <ExportProgress {...progress} />
          </div>
        ) : null}
      </div>
    </WindowShell>
  );
}
