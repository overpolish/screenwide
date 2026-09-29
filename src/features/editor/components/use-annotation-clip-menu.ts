// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";
import {
  PopupMenuAnchor,
  usePopupMenu,
} from "../../popup-panel/use-popup-menu";
import { availableArrangements, arranged } from "../annotation-order";
import { isPinnable, pinCorrections } from "../recording-annotation-pins";
import {
  RecordingAnnotationClip,
  recordingAnnotationClipsMeet,
} from "../recording-annotations";

import {
  annotationArrangeItems,
  annotationArrangementPicked,
} from "./annotation-arrange-items";

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
 * menu key, answered with the app's own menu. Under an Arrange heading it
 * moves the annotation through the drawing order, past the clips showing at
 * the same time as it; under a Tracking heading it pins the annotation to the
 * content or lets it go, says its content is out of view from here or back in
 * view, and takes its corrections away when it has any. The panel draws no
 * disabled rows, so an action is left out until it applies, and a menu with
 * nothing in it does not open.
 *
 * Every mounted menu hears every pick under its prefix, so each place that
 * opens this menu names it with a prefix of its own.
 */
export function useAnnotationClipMenu({
  clips,
  idPrefix,
  onClipsChange,
  pinning,
}: {
  clips: RecordingAnnotationClip[];
  idPrefix: string;
  onClipsChange: (clips: RecordingAnnotationClip[]) => void;
  pinning: AnnotationClipPinning | undefined;
}) {
  const openMenu = usePopupMenu({
    idPrefix,
    label: "Annotation actions",
    mode: "menu",
    onSelect: (itemId, annotationId) => {
      const arrangement = annotationArrangementPicked(itemId);
      if (arrangement) {
        const index = clips.findIndex(
          (clip) => clip.annotation.id === annotationId,
        );
        if (index >= 0)
          onClipsChange(
            arranged(clips, index, {
              arrangement,
              meets: recordingAnnotationClipsMeet,
            }),
          );
      } else if (itemId === "pin") pinning?.onPinnedChange(annotationId, true);
      else if (itemId === "unpin") pinning?.onPinnedChange(annotationId, false);
      else if (itemId === "hide") pinning?.onHideHere(annotationId);
      else if (itemId === "show") pinning?.onShowHere(annotationId);
      else if (itemId === "clear") pinning?.onClearCorrections(annotationId);
    },
    width: MENU_WIDTH,
  });

  return (anchor: PopupMenuAnchor, clip: RecordingAnnotationClip) => {
    const id = clip.annotation.id;
    const index = clips.findIndex((item) => item.annotation.id === id);
    const tracking: PopupPanelItem[] =
      pinning && isPinnable(clip)
        ? clip.pin
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
        : [];
    const items = [
      ...(index >= 0
        ? annotationArrangeItems(
            availableArrangements(clips, index, recordingAnnotationClipsMeet),
          )
        : []),
      ...tracking.map((item) => ({ ...item, section: SECTION })),
    ];
    if (items.length === 0) return;
    return openMenu({ anchor, context: id, items });
  };
}
