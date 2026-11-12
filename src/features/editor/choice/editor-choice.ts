// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { drawingToolKind } from "../tools/tool-registry";

import type { AnnotationKind } from "../../annotations/types";

// The rules of the choice model (island/DECISIONS.md, "The choice model")
// that the editor decides, the same in both editors: when ⌘ may hold Select,
// which pen the choice calls for and when, and which pens let the choice go.
// The engine decides what each press on the picture is; these decide what the
// editor makes of it.

/**
 * Whether ⌘ (Ctrl on Windows) going down may hold Select over `tool`: a pen,
 * or Select with a layer in hand, whose chrome the hold stands down. Crop,
 * Frame and Scene draw their own chrome, which a hold would take away and
 * bring back on every ⌘; nothing holds while the Export island stands over
 * the editor.
 */
export const canHoldSelect = ({
  hasLayerInHand,
  isExportOpen,
  tool,
}: {
  hasLayerInHand: boolean;
  isExportOpen: boolean;
  tool: string;
}) =>
  !isExportOpen &&
  (drawingToolKind(tool) !== null || (tool === "select" && hasLayerInHand));

/**
 * The pen to take up for the annotation chosen, or null to leave the tool as
 * it is. One annotation chosen with Select or a pen takes up its own pen, so
 * the island dresses it and the next press draws its kind; several chosen
 * have no one kind and take up nothing. It waits for the gesture to end: not
 * while the picture is pressed, nor while ⌘ holds Select, so a band that
 * starts with one annotation chosen and ends with several never passes
 * through that one's pen on the way.
 */
export const followedTool = ({
  isHeld,
  isPicturePressed,
  kind,
  tool,
}: {
  isHeld: boolean;
  isPicturePressed: boolean;
  kind: AnnotationKind | null;
  tool: string;
}): AnnotationKind | null => {
  if (kind === null || isHeld || isPicturePressed) return null;
  if (tool !== "select" && drawingToolKind(tool) === null) return null;
  return tool === kind ? null : kind;
};

/**
 * Whether taking up `next` should let the annotation in hand go.
 *
 * Reaching for the tool that draws the other shape is a decision about what
 * comes next, so the annotation goes with it: otherwise the tool follows the
 * annotation straight back and the switch never lands. Taking up the tool an
 * annotation was drawn with keeps it, which is how choosing one adopts its
 * tool, the pen and a stroke included.
 */
export const toolDisagreesWithAnnotation = (
  next: string | null,
  kind: AnnotationKind | null,
) => drawingToolKind(next) !== null && kind !== null && kind !== next;
