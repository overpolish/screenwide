// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ImageArt } from "../annotations/annotations";
import type { EditorKind } from "../types";

/** The workspaces that can place a picture asked for from the panel, each by
 * the placer its editor has registered. */
const placers = new Map<EditorKind, (art: ImageArt) => void>();

/** Let `workspace` place pictures asked for from the panel; returns what
 * takes the placer away again. */
export const registerImagePlacer = (
  workspace: EditorKind,
  placer: (art: ImageArt) => void,
) => {
  placers.set(workspace, placer);
  return () => {
    if (placers.get(workspace) === placer) placers.delete(workspace);
  };
};

/** Place `art` in the middle of the layer in hand on `workspace`, as a
 * pasted picture is. Nothing happens where that workspace cannot place one. */
export const requestImagePlacement = (workspace: EditorKind, art: ImageArt) => {
  placers.get(workspace)?.(art);
};
