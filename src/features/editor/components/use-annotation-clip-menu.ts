// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";
import {
  PopupMenuAnchor,
  usePopupMenu,
} from "../../popup-panel/use-popup-menu";
import { pinCorrections } from "../recording-annotation-pins";
import { RecordingAnnotationClip } from "../recording-annotations";

const MENU_WIDTH = 200;

/** What a pinned annotation's menu can do, by annotation id. */
export type AnnotationClipPinning = {
  /** Whether the playhead is where the annotation shows and its content can
   * be said to have gone out of view. */
  canHideHere: (annotationId: string) => boolean;
  /** Whether the playhead is where the annotation is hidden because its
   * content went out of sight or was said to be out of view. */
  canShowHere: (annotationId: string) => boolean;
  onClearCorrections: (annotationId: string) => void;
  /** Says the content is out of view from the playhead: the annotation is
   * hidden, and nothing is followed, until the next keyframe. */
  onHideHere: (annotationId: string) => void;
  onPinnedChange: (annotationId: string, pinned: boolean) => void;
  /** Puts the annotation back at the playhead where it was last shown. */
  onShowHere: (annotationId: string) => void;
};

/** The heading the pin's actions sit under, the way the speed menu heads its
 * rates. */
const SECTION = "Tracking";

/**
 * A right click on an annotation, on its clip or on the picture, or its clip's
 * menu key, answered with the app's own menu, under a Tracking heading: pin
 * it to the content or let it go, say its content is out of view from here or
 * back in view, and take its corrections away when it has any. The panel draws
 * no disabled rows, so an action is left out until it applies.
 *
 * Every mounted menu hears every pick under its prefix, so each place that
 * opens this menu names it with a prefix of its own.
 */
export function useAnnotationClipMenu({
  idPrefix,
  pinning,
}: {
  idPrefix: string;
  pinning: AnnotationClipPinning | undefined;
}) {
  const openMenu = usePopupMenu({
    idPrefix,
    label: "Annotation actions",
    mode: "menu",
    onSelect: (itemId, annotationId) => {
      if (itemId === "pin") pinning?.onPinnedChange(annotationId, true);
      else if (itemId === "unpin") pinning?.onPinnedChange(annotationId, false);
      else if (itemId === "hide") pinning?.onHideHere(annotationId);
      else if (itemId === "show") pinning?.onShowHere(annotationId);
      else if (itemId === "clear") pinning?.onClearCorrections(annotationId);
    },
    width: MENU_WIDTH,
  });

  return (anchor: PopupMenuAnchor, clip: RecordingAnnotationClip) => {
    if (!pinning) return;
    const id = clip.annotation.id;
    const items: PopupPanelItem[] = (
      clip.pin
        ? [
            ...(pinning.canHideHere(id)
              ? [{ id: "hide", label: "Content Out of View" }]
              : []),
            ...(pinning.canShowHere(id)
              ? [{ id: "show", label: "Content Back in View" }]
              : []),
            ...(pinCorrections(clip.pin) > 0
              ? [{ id: "clear", label: "Clear Corrections" }]
              : []),
            { id: "unpin", label: "Unpin" },
          ]
        : [{ id: "pin", label: "Pin to Content" }]
    ).map((item) => ({ ...item, section: SECTION }));
    return openMenu({ anchor, context: id, items });
  };
}
