// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { pointerAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";
import {
  annotationArrangeItems,
  annotationArrangementPicked,
} from "../annotations/annotation-arrange-items";
import { Arrangement } from "../annotations/annotation-order";
import { annotationMenuTargets } from "../annotations/annotation-selection";
import { Annotation } from "../annotations/annotations";

import { screenshotAnnotationArrangements } from "./screenshot-annotation-layers";
import { ScreenshotWorkspaceOutputSettings } from "./screenshot-output";

const MENU_WIDTH = 200;

/**
 * A right press on an annotation in the screenshot preview, answered with the
 * app's own menu at the pointer: it moves the annotation through the
 * stacking, or every one chosen with it when it is one of several, and a step
 * past the end of its layer carries it into the next one. The native chrome
 * picks annotations on every layer, so the report names the layer the
 * annotation is drawn on; a report naming one that is not there opens
 * nothing.
 */
export function useScreenshotAnnotationMenu({
  arrange,
  selectedIds,
  workspace,
}: {
  arrange: (
    arrangement: Arrangement,
    itemId: number,
    isMember: (annotation: Annotation) => boolean,
  ) => void;
  selectedIds: ReadonlySet<string>;
  workspace: ScreenshotWorkspaceOutputSettings | undefined;
}) {
  // The layer the menu was opened on, held while it is open: the pick
  // arrives as a separate event, after the workspace may have moved on.
  const layerRef = useRef<number | null>(null);
  const openMenu = usePopupMenu({
    idPrefix: "screenshot-annotation:",
    label: "Annotation actions",
    mode: "menu",
    onSelect: (itemId, annotationId) => {
      const arrangement = annotationArrangementPicked(itemId);
      const layerId = layerRef.current;
      if (!arrangement || layerId === null) return;
      const targets = annotationMenuTargets(annotationId, selectedIds);
      arrange(arrangement, layerId, (annotation) => targets.has(annotation.id));
    },
    width: MENU_WIDTH,
  });
  const openRef = useRef(
    (_: {
      annotationId: string;
      paneIndex: number;
      x: number;
      y: number;
    }) => {},
  );
  openRef.current = ({ annotationId, paneIndex, x, y }) => {
    const items = workspace?.items ?? [];
    if (!workspace || paneIndex < 0 || paneIndex >= items.length) return;
    const layerId = items[paneIndex].id;
    const targets = annotationMenuTargets(annotationId, selectedIds);
    const rows = annotationArrangeItems(
      screenshotAnnotationArrangements(workspace, layerId, (annotation) =>
        targets.has(annotation.id),
      ),
    );
    if (rows.length === 0) return;
    layerRef.current = layerId;
    void openMenu({
      anchor: pointerAnchor(x, y),
      context: annotationId,
      items: rows,
    });
  };
  useEffect(() => {
    // The subscription lands after a hop; a cleanup that runs first must
    // still let go of it.
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<unknown>("screenshot-preview://annotation-menu", (event) => {
      const payload = event.payload;
      if (disposed || typeof payload !== "object" || payload === null) return;
      const { annotationId, paneIndex, x, y } = payload as Record<
        string,
        unknown
      >;
      if (
        typeof annotationId === "string" &&
        typeof paneIndex === "number" &&
        Number.isInteger(paneIndex) &&
        typeof x === "number" &&
        typeof y === "number" &&
        Number.isFinite(x) &&
        Number.isFinite(y)
      )
        openRef.current({ annotationId, paneIndex, x, y });
    }).then((dispose) => {
      if (disposed) dispose();
      else unlisten = dispose;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
