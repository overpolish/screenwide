// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ClipboardCopy,
  FileImage,
  FileMusic,
  FilePlay,
  FolderSearch,
} from "lucide-react";
import { type PointerEvent, useRef } from "react";

import { platformArg, t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { IconButton } from "../../../../primitives/button/icon-button";
import { formatBytes } from "../../duration";

import type { ToolPanelExport } from "../../tools/tool-panel-export";

/** How far the pointer moves, pressed on the file, before it is a drag. */
const DRAG_THRESHOLD_PX = 3;

const QUIET = "text-fg-secondary enabled:hover:text-fg";

/** What is done with the written file. Copy and the drag are only offered
 * where the system takes them (macOS); without them, a handler is left out
 * and its part with it. */
export type ExportFinishHandlers = {
  onReveal: () => void;
  onCopy?: () => void;
  /** Takes the dragged file out to another app, from the press under way. */
  onDragOut?: () => void;
};

/** The finish's actions, at the row's end: each a glyph, named for
 * assistive technology alone. */
export function ExportFinishActions({
  onCopy,
  onReveal,
}: ExportFinishHandlers) {
  return (
    <>
      <IconButton
        aria-label={t("editor-export-reveal", { platform: platformArg() })}
        className={QUIET}
        onPress={onReveal}
      >
        <FolderSearch aria-hidden />
      </IconButton>
      {onCopy ? (
        <IconButton
          aria-label={t("editor-export-copy")}
          className={QUIET}
          onPress={onCopy}
        >
          <ClipboardCopy aria-hidden />
        </IconButton>
      ) : null}
    </>
  );
}

const FILE_GLYPHS = {
  audio: FileMusic,
  screenshot: FileImage,
  video: FilePlay,
};

/** The written file as a chip, its kind's glyph, name and size, which is
 * dragged out into another app as the file itself where the system lets a
 * drag start (`onDragOut`). The drag starts once the pressed pointer moves,
 * so a press alone does nothing. */
export function ExportFileChip({
  file,
  kind,
  onDragOut,
}: {
  file: NonNullable<ToolPanelExport["exported"]>;
  kind: ToolPanelExport["kind"];
  onDragOut?: () => void;
}) {
  const pressRef = useRef<{ x: number; y: number } | null>(null);
  const Glyph = FILE_GLYPHS[kind];
  const drag = onDragOut
    ? {
        "aria-label": t("editor-export-drag-file", { name: file.name }),
        onPointerDown: (event: PointerEvent) => {
          if (event.button === 0)
            pressRef.current = { x: event.clientX, y: event.clientY };
        },
        onPointerMove: (event: PointerEvent) => {
          const press = pressRef.current;
          if (!press || event.buttons !== 1) return;
          if (
            Math.hypot(event.clientX - press.x, event.clientY - press.y) <
            DRAG_THRESHOLD_PX
          )
            return;
          pressRef.current = null;
          onDragOut();
        },
        onPointerUp: () => {
          pressRef.current = null;
        },
        role: "img",
      }
    : {};
  return (
    <div
      {...drag}
      className={cn(
        "flex h-island-control min-w-0 items-center gap-inline rounded-full bg-fill ps-3 pe-2 text-label",
        onDragOut && "cursor-grab",
      )}
    >
      <Glyph aria-hidden className="size-icon shrink-0 text-fg-secondary" />
      <span className="max-w-56 min-w-0 truncate" dir="auto">
        {file.name}
      </span>
      <Badge>{formatBytes(file.bytes)}</Badge>
    </div>
  );
}
