// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";

import { Arrangement } from "./annotation-order";

/** The heading the moves through the drawing order sit under. */
const SECTION = "Arrange";
const PREFIX = "arrange:";

/**
 * The rows an annotation's menu offers for its place in the drawing order,
 * worded as a video layer's are, with the bracket keys that do the same. The
 * panel draws no disabled rows, so a move that passes nothing is left out.
 * Moving to the front or back is offered with the step that way unless the
 * caller says otherwise: a still's annotation can step into the next layer
 * while already at the end of its own.
 */
export const annotationArrangeItems = ({
  canBringForward,
  canMoveToBack,
  canMoveToFront,
  canSendBackward,
}: {
  canBringForward: boolean;
  canSendBackward: boolean;
  canMoveToBack?: boolean;
  canMoveToFront?: boolean;
}): PopupPanelItem[] =>
  [
    ...(canBringForward
      ? [{ id: `${PREFIX}forward`, label: "Move Forward", shortcut: "]" }]
      : []),
    ...((canMoveToFront ?? canBringForward)
      ? [{ id: `${PREFIX}front`, label: "Move to Front", shortcut: "Shift+]" }]
      : []),
    ...(canSendBackward
      ? [{ id: `${PREFIX}backward`, label: "Move Backward", shortcut: "[" }]
      : []),
    ...((canMoveToBack ?? canSendBackward)
      ? [{ id: `${PREFIX}back`, label: "Move to Back", shortcut: "Shift+[" }]
      : []),
  ].map((item) => ({ ...item, section: SECTION }));

/** The move a picked row names, or null for a row that is not one. */
export const annotationArrangementPicked = (
  itemId: string,
): Arrangement | null => {
  const move = itemId.startsWith(PREFIX) ? itemId.slice(PREFIX.length) : null;
  return move === "back" ||
    move === "backward" ||
    move === "forward" ||
    move === "front"
    ? move
    : null;
};
