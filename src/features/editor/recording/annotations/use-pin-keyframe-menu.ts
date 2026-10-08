// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { PopupPanelItem } from "../../../popup-panel/store";
import {
  pointerAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";

const MENU_PREFIX = "pin-keyframe:";
const MENU_WIDTH = 200;

/** What a keyframe on a pinned clip is: the frame it was pinned on, a
 * correction made by hand, or where its content was said to go out of view. */
export type PinKeyframeKind = "correction" | "outOfView" | "pinned";

const deleteLabel = (kind: PinKeyframeKind) => {
  switch (kind) {
    case "correction":
      return t("editor-pin-delete-correction");
    case "outOfView":
      return t("editor-pin-delete-out-of-view");
    case "pinned":
      return t("editor-pin-delete-pinned-frame");
  }
};

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
    label: t("editor-pin-keyframe-actions"),
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
    kind,
    ms,
    point,
  }: {
    annotationId: string;
    kind: PinKeyframeKind;
    ms: number;
    point: { x: number; y: number };
  }) => {
    const items: PopupPanelItem[] = [
      { icon: "trash", id: "delete", label: deleteLabel(kind) },
    ];
    return openMenu({
      anchor: pointerAnchor(point.x, point.y),
      context: `${annotationId}:${ms.toString()}`,
      items,
    });
  };
}
