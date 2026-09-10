// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ref } from "react";

import { Button } from "../../components/base/button/button";
import { Text } from "../../components/base/text/text";
import { WindowShell } from "../../components/shared/window-shell/window-shell";

export type ConfirmSheetProps = {
  cancelLabel: string;
  confirmLabel: string;
  message: string;
  onCancel: () => void;
  onConfirm: () => void;
  title: string;
  /** Measured by the window so it can fit itself to the words. */
  contentRef?: Ref<HTMLElement>;
};

/**
 * The confirmation sheet as it looks: the only place its shell exists, so the
 * real window and the stories cannot drift apart.
 *
 * A sheet has no title bar. It is put on screen by the window that asked,
 * belongs where that window puts it, and the two buttons are the only way out,
 * so there is nothing for a caption to offer.
 */
export function ConfirmSheet({
  cancelLabel,
  confirmLabel,
  contentRef,
  message,
  onCancel,
  onConfirm,
  title,
}: ConfirmSheetProps) {
  return (
    <WindowShell height="content" ref={contentRef}>
      <div className="flex flex-col gap-section p-window-inset">
        <Text variant="headline">{title}</Text>
        <Text>{message}</Text>
        <div className="flex justify-end gap-control-inset">
          <Button onPress={onCancel}>{cancelLabel}</Button>
          {/* The sheet opens on the confirming button, so Return answers the
              question the sheet was put up to ask. */}
          <Button color="primary" onPress={onConfirm}>
            {confirmLabel}
          </Button>
        </div>
      </div>
    </WindowShell>
  );
}
