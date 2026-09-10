// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ref } from "react";

import logoUrl from "../../../assets/screenwide-mark.svg";
import { WindowHeader } from "../../../components/shared/window-header/window-header";
import { WindowShell } from "../../../components/shared/window-shell/window-shell";
import { cn } from "../../../lib/styling";
import {
  EditorExportForm,
  EditorExportFormProps,
} from "../components/editor-export-form";

import { ExportProgress, ExportProgressProps } from "./export-progress";

export type ExportOptionsWindowViewProps = {
  /** The export form. It stays in place, disabled, while a save runs. */
  form: EditorExportFormProps;
  /** Covers the form with the running save and takes the window's way out. */
  isSaving: boolean;
  onClose: () => void;
  /** Measured by the window so it can fit itself to the form. */
  contentRef?: Ref<HTMLElement>;
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
  progress,
}: ExportOptionsWindowViewProps) {
  return (
    <WindowShell
      className="relative"
      header={
        <WindowHeader
          isDraggable={false}
          leadingSection={
            <img
              alt="Screenwide"
              className="brightness-0 dark:invert"
              draggable={false}
              src={logoUrl}
            />
          }
          // The save is watched from here, so there is no way out until it ends.
          onClose={isSaving ? undefined : onClose}
          title="Export"
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
            "flex flex-col px-window-inset pb-window-inset",
            isSaving && progress !== undefined && "invisible",
          )}
        >
          <EditorExportForm {...form} isSaving={isSaving} />
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
