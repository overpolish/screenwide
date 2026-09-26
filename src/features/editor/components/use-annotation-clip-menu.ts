// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";
import {
  PopupMenuAnchor,
  usePopupMenu,
} from "../../popup-panel/use-popup-menu";
import { pinCorrections } from "../recording-annotation-pins";
import { RecordingAnnotationClip } from "../recording-annotations";

const MENU_PREFIX = "annotation-clip:";
const MENU_WIDTH = 200;

/**
 * A right click on an annotation's clip, or its menu key, answered with the
 * app's own menu: pin it to the content or let it go, and take its corrections
 * away when it has any. The panel draws no disabled rows, so Clear
 * Corrections is left out until there is something to clear.
 */
export function useAnnotationClipMenu({
  onClearCorrections,
  onPinnedChange,
}: {
  onClearCorrections: (annotationId: string) => void;
  onPinnedChange: (annotationId: string, pinned: boolean) => void;
}) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: "Annotation actions",
    mode: "menu",
    onSelect: (itemId, annotationId) => {
      if (itemId === "pin") onPinnedChange(annotationId, true);
      else if (itemId === "unpin") onPinnedChange(annotationId, false);
      else if (itemId === "clear") onClearCorrections(annotationId);
    },
    width: MENU_WIDTH,
  });

  return (anchor: PopupMenuAnchor, clip: RecordingAnnotationClip) => {
    const items: PopupPanelItem[] = clip.pin
      ? [
          { id: "unpin", label: "Unpin" },
          ...(pinCorrections(clip.pin) > 0
            ? [{ id: "clear", label: "Clear Corrections" }]
            : []),
        ]
      : [{ id: "pin", label: "Pin to Content" }];
    return openMenu({
      anchor,
      context: clip.annotation.id,
      items,
    });
  };
}
