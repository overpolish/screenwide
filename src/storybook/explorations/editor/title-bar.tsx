// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ClipboardCopy, Redo2, Share, Trash2, Undo2 } from "lucide-react";

import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { Toolbar } from "../../../primitives/toolbar/toolbar";
import { PROJECT_NAME } from "../shared/mock-data";

export type EditorKind = "audio" | "recording" | "screenshot";

// The window's own bar: the name after the window buttons, then history and
// what leaves the editor. A window toolbar, so its text commands carry a
// leading icon; Export is the one thing the window is for, so it alone is
// primary. A screenshot is copied as often as it is saved, so Copy sits
// beside it.
export function TitleBar({ kind }: { kind: EditorKind }) {
  return (
    <header className="flex h-12 shrink-0 items-center gap-separate ps-traffic-lights pe-inset">
      <span className="min-w-0 truncate text-heading" dir="auto">
        {kind === "screenshot"
          ? "Screenwide 2026-08-08 at 14.32.05"
          : PROJECT_NAME}
      </span>
      <Toolbar aria-label="History" className="ms-auto">
        <IconButton aria-label="Undo" variant="ghost">
          <Undo2 />
        </IconButton>
        <IconButton aria-label="Redo" isDisabled variant="ghost">
          <Redo2 />
        </IconButton>
        <IconButton aria-label="Delete" variant="ghost">
          <Trash2 />
        </IconButton>
      </Toolbar>
      <span className="flex items-center gap-inline">
        {kind === "screenshot" ? (
          <Button>
            <ClipboardCopy />
            Copy
          </Button>
        ) : null}
        <Button color="primary">
          <Share />
          Export
        </Button>
      </span>
    </header>
  );
}
