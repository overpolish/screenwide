// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";
import { pointerAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";

const MENU_PREFIX = "pin-keyframe:";
const MENU_WIDTH = 200;

/**
 * A right click on a pinned clip's keyframe, answered with the app's own menu:
 * one action, taking that keyframe away. A pin's only keyframe is the pin
 * itself, so it offers nothing; Unpin lets it go.
 */
export function usePinKeyframeMenu(
  onDelete: (annotationId: string, ms: number) => void,
) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: "Keyframe actions",
    mode: "menu",
    onSelect: (itemId, context) => {
      if (itemId !== "delete") return;
      const split = context.lastIndexOf(":");
      const ms = Number(context.slice(split + 1));
      if (split > 0 && Number.isFinite(ms))
        onDelete(context.slice(0, split), ms);
    },
    width: MENU_WIDTH,
  });

  return ({
    annotationId,
    isPinnedFrame,
    ms,
    point,
  }: {
    annotationId: string;
    isPinnedFrame: boolean;
    ms: number;
    point: { x: number; y: number };
  }) => {
    const items: PopupPanelItem[] = [
      {
        icon: "trash",
        id: "delete",
        label: isPinnedFrame ? "Delete Pinned Frame" : "Delete Correction",
      },
    ];
    return openMenu({
      anchor: pointerAnchor(point.x, point.y),
      context: `${annotationId}:${ms.toString()}`,
      items,
    });
  };
}
