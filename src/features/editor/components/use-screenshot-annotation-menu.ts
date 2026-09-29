// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { pointerAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";
import { availableArrangements, arranged } from "../annotation-order";
import { Annotation } from "../annotations";

import {
  annotationArrangeItems,
  annotationArrangementPicked,
} from "./annotation-arrange-items";

const MENU_WIDTH = 200;

// A still draws every annotation on a layer together, so any two meet.
const meet = () => true;

/**
 * A right press on an annotation in the screenshot preview, answered with the
 * app's own menu at the pointer: it moves the annotation through its layer's
 * drawing order. The native chrome only hit-tests the selected layer, so the
 * annotation is looked for among `annotations`, that layer's; a report naming
 * one that is not there opens nothing.
 */
export function useScreenshotAnnotationMenu({
  annotations,
  onCommit,
}: {
  annotations: Annotation[];
  onCommit: (annotations: Annotation[]) => void;
}) {
  const openMenu = usePopupMenu({
    idPrefix: "screenshot-annotation:",
    label: "Annotation actions",
    mode: "menu",
    onSelect: (itemId, annotationId) => {
      const arrangement = annotationArrangementPicked(itemId);
      const index = annotations.findIndex(
        (annotation) => annotation.id === annotationId,
      );
      if (arrangement && index >= 0)
        onCommit(
          arranged(annotations, index, {
            arrangement,
            meets: meet,
          }),
        );
    },
    width: MENU_WIDTH,
  });
  const openRef = useRef(
    (_: { annotationId: string; x: number; y: number }) => {},
  );
  openRef.current = ({ annotationId, x, y }) => {
    const index = annotations.findIndex(
      (annotation) => annotation.id === annotationId,
    );
    if (index < 0) return;
    const items = annotationArrangeItems(
      availableArrangements(annotations, index, meet),
    );
    if (items.length > 0)
      void openMenu({
        anchor: pointerAnchor(x, y),
        context: annotationId,
        items,
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
      const { annotationId, x, y } = payload as Record<string, unknown>;
      if (
        typeof annotationId === "string" &&
        typeof x === "number" &&
        typeof y === "number" &&
        Number.isFinite(x) &&
        Number.isFinite(y)
      )
        openRef.current({ annotationId, x, y });
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
