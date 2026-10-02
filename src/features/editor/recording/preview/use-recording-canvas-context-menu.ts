// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { pointerAnchor } from "../../../popup-panel/use-popup-menu";
import { RecordingAnnotationClip } from "../annotations/recording-annotations";
import {
  AnnotationClipPinning,
  useAnnotationClipMenu,
} from "../annotations/use-annotation-clip-menu";

/** What a right press on an annotation needs: the clips to find it among and
 * to reorder, the pin actions its menu offers, and the choice it may be one
 * of. */
type CanvasMenuAnnotations = {
  clips: RecordingAnnotationClip[];
  onClipsChange: (clips: RecordingAnnotationClip[]) => void;
  pinning: AnnotationClipPinning;
  selectedIds: ReadonlySet<string>;
};

/**
 * A right press on an annotation in the native canvas opens the menu its
 * timeline clip opens. The native side has already chosen it, or kept the
 * group it is one of, and names it here; a right press on a bare pane has no
 * menu.
 */
export function useRecordingCanvasContextMenu(
  annotations: CanvasMenuAnnotations,
) {
  const openAnnotationMenu = useAnnotationClipMenu({
    clips: annotations.clips,
    idPrefix: "annotation-canvas:",
    onClipsChange: annotations.onClipsChange,
    pinning: annotations.pinning,
    selectedIds: annotations.selectedIds,
  });
  const openCanvasAnnotationMenuRef = useRef<
    (annotationId: string, x: number, y: number) => void
  >(() => undefined);
  openCanvasAnnotationMenuRef.current = (annotationId, x, y) => {
    const clip = annotations.clips.find(
      (item) => item.annotation.id === annotationId,
    );
    if (clip) void openAnnotationMenu(pointerAnchor(x, y), clip);
  };
  useEffect(() => {
    // The subscription lands after a hop. A cleanup that runs before it
    // lands, as React's development double-mount does, must still let go of
    // it, or the window hears every click twice and the menu opens and
    // closes in one go.
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void getCurrentWindow()
      .listen<{
        annotationId: string | null;
        x: number;
        y: number;
      }>("preview://context-menu", ({ payload }) => {
        if (payload.annotationId !== null)
          openCanvasAnnotationMenuRef.current(
            payload.annotationId,
            payload.x,
            payload.y,
          );
      })
      .then((dispose) => {
        if (disposed) dispose();
        else unlisten = dispose;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
