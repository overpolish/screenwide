// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { t } from "../../../../i18n/i18n";
import {
  pointerAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";
import {
  annotationArrangeItems,
  annotationArrangementPicked,
} from "../../annotations/annotation-arrange-items";
import { Arrangement } from "../../annotations/annotation-order";
import { baseOf } from "../../timeline/strip/strip-part";
import { RecordingCameraPlacement } from "../annotations/recording-annotation-layers";
import { RecordingAnnotationClip } from "../annotations/recording-annotations";
import {
  AnnotationClipPinning,
  useAnnotationClipMenu,
} from "../annotations/use-annotation-clip-menu";
import { formatSyncMenu, pickedFormatSync } from "../formats/format-item-menu";
import { useFrameFormats } from "../formats/frame-formats-context";
import {
  ScenePane,
  scenePaneArrangements,
} from "../scenes/recording-scene-order";
import { RecordingSceneClip } from "../scenes/recording-scenes";

const MENU_WIDTH = 200;

/** What a right press on an annotation needs: the clips to find it among and
 * to reorder, where One video draws the pictures they move between, the pin
 * actions its menu offers, and the choice it may be one of. */
type CanvasMenuAnnotations = {
  cameraPlacement: RecordingCameraPlacement | null;
  clips: RecordingAnnotationClip[];
  onClipsChange: (clips: RecordingAnnotationClip[]) => void;
  pinning: AnnotationClipPinning;
  selectedIds: ReadonlySet<string>;
};

/** What a right press on a pane needs: the scene under the playhead, null
 * where none plays, and the move that reorders its panes. */
type CanvasMenuScene = {
  arrange: (pane: ScenePane, move: Arrangement) => void;
  clip: RecordingSceneClip | null;
};

/** The pane a bare press names: the layer the native side picked. */
const paneOfLayer = (layer: number): ScenePane | null =>
  layer === 0 ? "screen" : layer === 1 ? "camera" : null;

/**
 * A right press on an annotation in the native canvas opens the menu its
 * timeline clip opens. The native side has already chosen it, or kept the
 * group it is one of, and names it here. A right press on a pane of a custom
 * scene opens the moves through the scene's order, as a screenshot layer's
 * does; on any other pane it opens nothing.
 */
export function useRecordingCanvasContextMenu(
  annotations: CanvasMenuAnnotations,
  scene: CanvasMenuScene,
) {
  const openAnnotationMenu = useAnnotationClipMenu({
    cameraPlacement: annotations.cameraPlacement,
    clips: annotations.clips,
    idPrefix: "annotation-canvas:",
    onClipsChange: annotations.onClipsChange,
    pinning: annotations.pinning,
    selectedIds: annotations.selectedIds,
  });
  const sceneRef = useRef(scene);
  sceneRef.current = scene;
  const formats = useFrameFormats();
  const formatsRef = useRef(formats);
  formatsRef.current = formats;
  const openPaneMenu = usePopupMenu({
    idPrefix: "scene-pane:",
    label: t("editor-layer-actions"),
    mode: "menu",
    onSelect: (itemId, pane) => {
      const sceneId = sceneRef.current.clip?.id;
      if (
        sceneId &&
        pickedFormatSync(formatsRef.current, itemId, baseOf(sceneId))
      )
        return;
      const move = annotationArrangementPicked(itemId);
      if (move && (pane === "camera" || pane === "screen"))
        sceneRef.current.arrange(pane, move);
    },
    width: MENU_WIDTH,
  });
  const openCanvasMenuRef = useRef<
    (
      press: { annotationId: string | null; paneIndex: number },
      x: number,
      y: number,
    ) => void
  >(() => undefined);
  openCanvasMenuRef.current = ({ annotationId, paneIndex }, x, y) => {
    if (annotationId !== null) {
      const clip = annotations.clips.find(
        (item) => item.annotation.id === annotationId,
      );
      if (clip) void openAnnotationMenu(pointerAnchor(x, y), clip);
      return;
    }
    const pane = paneOfLayer(paneIndex);
    if (!pane || !scene.clip) return;
    const formatMenu = formatSyncMenu(formats, baseOf(scene.clip.id));
    const items = [
      ...annotationArrangeItems(scenePaneArrangements(scene.clip, pane)),
      ...formatMenu.items,
    ];
    if (items.length === 0) return;
    void openPaneMenu({
      anchor: pointerAnchor(x, y),
      context: pane,
      items,
      selectedIds: formatMenu.selectedIds,
    });
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
        paneIndex: number;
        x: number;
        y: number;
      }>("preview://context-menu", ({ payload }) => {
        openCanvasMenuRef.current(payload, payload.x, payload.y);
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
