// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { type EditorIslandKind } from "../island/host/editor-island-kind";
import { useEditorIslandWindow } from "../island/host/use-editor-island-window";
import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";

// The one island the screenshot editor shows over its picture: the column's
// or title bar's (`island`), or the Library its title bar's button brings
// out. Whichever was taken up last is out: the Library puts the tool in
// hand down, and a tool, a mode or annotations taken up put the Library
// away, so Escape only ever reaches the one in view.

export function useScreenshotLibrary({
  backToSelect,
  island,
}: {
  /** Select in hand with nothing chosen and no mode open. */
  backToSelect: () => void;
  /** The island of the tool, mode or annotations in hand. */
  island: EditorIslandKind | null;
}) {
  const [isOpen, setIsOpen] = useState(false);
  // An island newly taken up puts the Library away, in the same render.
  const [seen, setSeen] = useState(island);
  if (seen !== island) {
    setSeen(island);
    if (island !== null) setIsOpen(false);
  }
  const open = (next: boolean) => {
    if (next) backToSelect();
    setIsOpen(next);
  };
  const putAway = isOpen
    ? () => {
        setIsOpen(false);
      }
    : undefined;
  useEditorWindowShortcuts({
    onDeselect: putAway,
    onDeselectAll: putAway,
    onToggleLibrary: () => {
      open(!isOpen);
    },
    // Out in front of everything else, the Library is what Escape puts away.
    ownsEscape: isOpen,
  });
  useEditorIslandWindow({
    island: isOpen ? "library" : island,
    workspace: "screenshot",
  });
  /** The title bar's Library button. */
  return { isOpen, onOpenChange: open };
}
