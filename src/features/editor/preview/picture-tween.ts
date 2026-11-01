// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorKind } from "../types";

// A setting chosen on an island, such as the screen's padding, its corners,
// its shadow, its turn or the frame's size, is drawn as the picture moving
// to it rather than jumping: the compositor is handed the settings in
// between, frame by frame (`picture-between.ts`). Only a change asked for
// that way moves; a drag on the picture already moves under the pointer,
// and an undo puts things back at once. The settings written to the project
// are the new ones from the start; only what is drawn passes through the
// ones in between, so an export draws the new ones.

/** A part of a workspace's picture handed to the preview on its own: the
 * picture's settings, or a recording's scenes. */
export type PicturePart = "picture" | "scenes";

/** When each workspace last asked for its next change to be moved to, in
 * ms since the page loaded, and the parts that have taken the ask; a change
 * arriving long after is not that one. One ask reaches every part, as a
 * setting chosen in a scene changes the scenes and one outside it the
 * picture's settings. */
const asked = new Map<EditorKind, { at: number; taken: Set<PicturePart> }>();
/** How long an ask waits for the change it announces, in ms. */
const ASK_LIFETIME_MS = 600;

/** The next change to `workspace`'s picture is moved to, not jumped to. */
export const askPictureTween = (workspace: EditorKind) => {
  asked.set(workspace, { at: performance.now(), taken: new Set() });
};

/** Whether a change to `part` arriving now was asked to be moved to, without
 * using the ask up: a render reads it, and a render may run twice. */
export const pictureTweenAsked = (workspace: EditorKind, part: PicturePart) => {
  const ask = asked.get(workspace);
  return (
    ask !== undefined &&
    !ask.taken.has(part) &&
    performance.now() - ask.at < ASK_LIFETIME_MS
  );
};

/** Whether a change to `part` arriving now was asked to be moved to, which
 * uses the ask up for that part. */
export const takePictureTween = (workspace: EditorKind, part: PicturePart) => {
  const isAsked = pictureTweenAsked(workspace, part);
  asked.get(workspace)?.taken.add(part);
  return isAsked;
};

/** The settings whose change is moved to, by the asks that carry them. */
export const TWEENED_PATCH_KEYS = [
  "background",
  "cropAspect",
  "frameRadius",
  "frameSize",
  "recenterSelection",
  "resetCrop",
  "resetFrame",
  "resetPlacement",
  "sceneInset",
  "scenePlacementReset",
  "sceneRadius",
  "sceneRecenter",
  "sceneRotation",
  "selectionDropShadow",
  "selectionInset",
  "selectionRadius",
  "selectionRotation",
] as const;

/** Eases out, quickly away and slowly into place. */
export const easeOut = (t: number) => 1 - (1 - t) ** 3;
