// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy, Pilcrow, RotateCcw, Trash2, X } from "lucide-react";

import { Button } from "../../components/base/button/button";
import { IconButton } from "../../components/base/button/icon-button";
import { CanvasToolbar } from "../../components/shared/canvas-tools/canvas-toolbar";
import { ConfirmActionButton } from "../../components/shared/confirm-action-button/confirm-action-button";
import { cn } from "../../lib/styling";

export function TextRecognitionCloseAction({
  isMac,
  onClose,
}: {
  isMac: boolean;
  onClose: () => void;
}) {
  return (
    <CanvasToolbar
      className={cn(
        "absolute left-1/2 -translate-x-1/2",
        isMac ? "top-12" : "top-2",
      )}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
    >
      <ConfirmActionButton
        armedIcon={<Trash2 className="text-error" size={18} />}
        armedLabel="Confirm closing text recognition"
        idleIcon={<X size={18} />}
        idleLabel="Close text recognition"
        onConfirm={onClose}
        size="compact"
      />
    </CanvasToolbar>
  );
}

export function TextRecognitionActions({
  onClose,
  onCopyAll,
  onCopyAsParagraph,
  onReset,
}: {
  onClose: () => void;
  onCopyAll: () => void;
  onCopyAsParagraph: () => void;
  onReset: () => void;
}) {
  return (
    <>
      <Button
        className="shrink-0 whitespace-nowrap"
        onPress={onCopyAll}
        size="compact"
        variant="ghost"
      >
        <Copy size={15} />
        Copy all
      </Button>
      <Button
        className="shrink-0 whitespace-nowrap"
        onPress={onCopyAsParagraph}
        size="compact"
        variant="ghost"
      >
        <Pilcrow size={15} />
        Copy as paragraph
      </Button>
      <IconButton
        aria-label="Recognize another area"
        onPress={onReset}
        size="compact"
      >
        <RotateCcw size={15} />
      </IconButton>
      <ConfirmActionButton
        armedIcon={<Trash2 className="text-error" size={15} />}
        armedLabel="Confirm closing text recognition"
        idleIcon={<X size={15} />}
        idleLabel="Close text recognition"
        onConfirm={onClose}
        size="compact"
      />
    </>
  );
}
