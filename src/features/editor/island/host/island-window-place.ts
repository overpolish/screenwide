// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type EditorKind } from "../../types";

import {
  type EditorIslandKind,
  isStripIsland,
  type PictureInset,
  type PopupPanelIslandContent,
} from "./editor-island-kind";

// Where the editor's island window stands: over the editor's whole content
// area under its title bar, the picture, the strip and the columns, for
// every island. It never changes as one island gives way to another, so
// the window's frame and the island's place inside it never have to move
// together, which no two windows can do on one frame. The islands stand
// against the picture's rect inside it, handed over as `inset`; the Export
// island, taller than most pictures, reaches down past the picture's foot.

/** Both workspaces mark their picture area with this. */
export const previewViewport = () =>
  document.querySelector<HTMLElement>("[data-recording-preview-viewport]");

/** The editor's content under its title bar (`editor-panel.tsx`). */
export const editorContent = () =>
  document.querySelector<HTMLElement>("[data-editor-content]");

export type IslandWindowPlace = {
  /** The window's bounds in the editor's page. */
  bounds: DOMRect;
  inset: PictureInset;
};

/** Where the island window stands, and the picture in it; null while there
 * is no picture. */
export const islandWindowPlace = (): IslandWindowPlace | null => {
  const picture = previewViewport()?.getBoundingClientRect();
  const bounds = editorContent()?.getBoundingClientRect();
  if (!picture || !bounds) return null;
  return {
    bounds,
    inset: {
      bottom: bounds.bottom - picture.bottom,
      left: picture.left - bounds.left,
      right: bounds.right - picture.right,
      top: picture.top - bounds.top,
    },
  };
};

/** How far along the window, in px from its left edge, the middle of the
 * chosen strip items stands, which their island stands over. Several
 * marked alike, as the strip's gathered items and the parts of one split
 * at a jump are, stand as one, from the first one's start to the last
 * one's end. */
const stripPlace = (bounds: DOMRect) => {
  const marked = [...document.querySelectorAll("[data-strip-island]")].map(
    (element) => element.getBoundingClientRect(),
  );
  if (marked.length === 0) return 0;
  const left = Math.min(...marked.map((item) => item.left));
  const right = Math.max(...marked.map((item) => item.right));
  return (left + right) / 2 - bounds.left;
};

export const islandContent = (
  island: EditorIslandKind | null,
  { bounds, inset }: IslandWindowPlace,
  workspace: EditorKind,
): PopupPanelIslandContent => ({
  inset,
  island,
  kind: "island",
  workspace,
  x: isStripIsland(island) ? stripPlace(bounds) : 0,
});
