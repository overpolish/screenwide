// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";
import { pointerAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";
import { RecordingVideoTrackId } from "../types";

/** The labels carry their shortcut as a key hint at the trailing edge. */
const MENU_WIDTH = 180;

const FORWARD: PopupPanelItem = {
  id: "forward",
  label: "Move Forward",
  shortcut: "]",
};
const BACKWARD: PopupPanelItem = {
  id: "backward",
  label: "Move Backward",
  shortcut: "[",
};

/**
 * Which of the two moves apply to a layer, given the order it sits in.
 *
 * The panel draws no disabled rows, so the timeline row and the native canvas
 * both ask here rather than each working it out from the order themselves.
 */
export const recordingTrackMoves = (
  order: RecordingVideoTrackId[],
  track: RecordingVideoTrackId,
) => {
  const index = order.indexOf(track);
  return {
    canMoveBackward: index >= 0 && index < order.length - 1,
    canMoveForward: index > 0,
  };
};

/**
 * A right click on a video row, answered with the app's own menu at the
 * pointer, the way the speed menu opens on a clip.
 *
 * The panel draws no disabled rows, so a move that cannot apply is left out:
 * the topmost row offers only Move Backward, the bottom one only Move
 * Forward, and a lone row opens nothing at all.
 */
export const RECORDING_TRACK_MENU_PREFIX = "recording-track:";

export function useRecordingTrackMenu(
  onMove: (
    track: RecordingVideoTrackId,
    direction: "backward" | "forward",
  ) => void,
) {
  const openMenu = usePopupMenu({
    idPrefix: RECORDING_TRACK_MENU_PREFIX,
    label: "Video layer actions",
    mode: "menu",
    onSelect: (itemId, context) => {
      onMove(
        context === "camera" ? "camera" : "primary",
        itemId === "forward" ? "forward" : "backward",
      );
    },
    width: MENU_WIDTH,
  });

  return (
    point: { x: number; y: number },
    track: RecordingVideoTrackId,
    moves: { canMoveBackward: boolean; canMoveForward: boolean },
  ) =>
    openMenu({
      anchor: pointerAnchor(point.x, point.y),
      context: track,
      items: [
        ...(moves.canMoveForward ? [FORWARD] : []),
        ...(moves.canMoveBackward ? [BACKWARD] : []),
      ],
    });
}
