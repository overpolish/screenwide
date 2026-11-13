// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isWindows } from "../../../../lib/platform";
import { libraryIsland } from "../library/library-island";

import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type { EditorKind } from "../../types";
import type { EditorIslandContent } from "../editor-island";
import type { LibraryKind } from "../library/library-kind";

// The Library for each editor: a recording's lays songs and sound effects on
// its timeline and pictures on its picture, a screenshot's only pictures.

const KINDS: Record<EditorKind, readonly LibraryKind[]> = {
  recording: ["music", "effect", "image"],
  screenshot: ["image"],
};

export function libraryIslandContent({
  change,
  snapshot,
  workspace,
}: {
  change: (values: ToolPanelPatch) => void;
  snapshot: ToolPanelSnapshot;
  workspace: EditorKind;
}): EditorIslandContent {
  return libraryIsland({
    // Only the Mac lets a drag start from the island's window.
    canDrag: !isWindows(),
    hasSongRoom: snapshot.hasSongRoom ?? true,
    kinds: KINDS[workspace],
    onAddPicture: (libraryImage) => {
      change({ libraryImage });
    },
    onAddSound: (soundAddition) => {
      change({ soundAddition });
    },
  });
}
