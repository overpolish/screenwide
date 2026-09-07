// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ref } from "react";

import logoUrl from "../../../assets/screenwide-mark.svg";
import { Overlay } from "../../../components/base/overlay/overlay";
import { WindowHeader } from "../../../components/shared/window-header/window-header";
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
    <main
      className="window-surface gap-section relative flex w-full flex-col overflow-hidden rounded-window text-content-fg"
      // Windows cannot backdrop-blur over a transparent page (Mica lives
      // outside the webview), so the content blurs itself while this is set.
      data-overlay-open={isSaving && progress !== undefined ? "" : undefined}
      ref={contentRef}
    >
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
      <div className="px-window-inset pb-window-inset flex flex-col">
        <EditorExportForm {...form} isSaving={isSaving} />
      </div>
      {/* The form keeps the window's size, so cancelling lands back on it
          without the window moving; the save floats over it. */}
      <Overlay blur="lg" contained isOpen={isSaving && progress !== undefined}>
        {progress ? <ExportProgress {...progress} /> : null}
      </Overlay>
    </main>
  );
}
