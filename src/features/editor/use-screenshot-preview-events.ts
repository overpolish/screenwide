// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { RefObject, useEffect } from "react";

import {
  ScreenshotAnnotationChangeEvent,
  screenshotAnnotationChange,
  ScreenshotSelectionGestureEvent,
} from "./screenshot-preview-events";

/**
 * Everything the native screenshot preview reports back: the view transform,
 * which layer it selected, a pointer gesture in flight, and a finished
 * annotation edit.
 *
 * The handlers are taken as refs rather than values: each subscription lives
 * for the whole session, and re-subscribing on every render would drop events
 * between the unlisten and the next listen.
 */
export function useScreenshotPreviewEvents({
  isEnabled,
  onAnnotationChangeRef,
  onSelectionChangeRef,
  onSelectionGestureRef,
  onZoomChangeRef,
  sessionIdRef,
}: {
  isEnabled: boolean;
  onAnnotationChangeRef: RefObject<
    ((event: ScreenshotAnnotationChangeEvent) => void) | undefined
  >;
  onSelectionChangeRef: RefObject<
    ((paneIndex: number | null) => void) | undefined
  >;
  onSelectionGestureRef: RefObject<
    ((event: ScreenshotSelectionGestureEvent) => void) | undefined
  >;
  onZoomChangeRef: RefObject<((zoomPercent: number) => void) | undefined>;
  sessionIdRef: RefObject<number>;
}) {
  useEffect(() => {
    if (!isEnabled) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<{ sessionId: number; zoomPercent: number }>(
      "screenshot-preview://transform",
      (event) => {
        if (
          !disposed &&
          event.payload.sessionId === sessionIdRef.current &&
          Number.isFinite(event.payload.zoomPercent)
        ) {
          const roundedZoom = Math.round(event.payload.zoomPercent);
          onZoomChangeRef.current?.(roundedZoom);
        }
      },
    ).then((dispose) => {
      if (disposed) dispose();
      else unlisten = dispose;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [isEnabled, onZoomChangeRef, sessionIdRef]);

  useEffect(() => {
    if (!isEnabled) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<{ paneIndex: number | null; sessionId: number }>(
      "screenshot-preview://selection-change",
      (event) => {
        const payload = event.payload;
        if (
          disposed ||
          payload.sessionId !== sessionIdRef.current ||
          (payload.paneIndex !== null && !Number.isInteger(payload.paneIndex))
        )
          return;
        onSelectionChangeRef.current?.(payload.paneIndex);
      },
    ).then((dispose) => {
      if (disposed) dispose();
      else unlisten = dispose;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [isEnabled, onSelectionChangeRef, sessionIdRef]);

  useEffect(() => {
    if (!isEnabled) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<unknown>("screenshot-preview://annotation-change", (event) => {
      if (disposed) return;
      const change = screenshotAnnotationChange(
        event.payload,
        sessionIdRef.current,
      );
      if (change) onAnnotationChangeRef.current?.(change);
    }).then((dispose) => {
      if (disposed) dispose();
      else unlisten = dispose;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [isEnabled, onAnnotationChangeRef, sessionIdRef]);

  useEffect(() => {
    if (!isEnabled) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<
      Omit<ScreenshotSelectionGestureEvent, "operation"> & {
        operation: number;
        sessionId: number;
      }
    >("screenshot-preview://selection-gesture", (event) => {
      const payload = event.payload;
      if (
        disposed ||
        payload.sessionId !== sessionIdRef.current ||
        !Number.isFinite(payload.deltaX) ||
        !Number.isFinite(payload.deltaY) ||
        !Number.isInteger(payload.edges) ||
        ![0, 1, 2, 3, 4, 5, 6, 7].includes(payload.operation) ||
        !Number.isInteger(payload.paneIndex) ||
        !Number.isFinite(payload.scale) ||
        !["begin", "update", "end", "cancel"].includes(payload.phase)
      )
        return;
      onSelectionGestureRef.current?.({
        deltaX: payload.deltaX,
        deltaY: payload.deltaY,
        edges: payload.edges,
        operation: [
          "move",
          "resize",
          "radius",
          "frameResize",
          "frameRadius",
          "cropMove",
          "cropResize",
        ][payload.operation] as ScreenshotSelectionGestureEvent["operation"],
        paneIndex: payload.paneIndex,
        phase: payload.phase,
        scale: payload.scale,
      });
    }).then((dispose) => {
      if (disposed) dispose();
      else unlisten = dispose;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [isEnabled, onSelectionGestureRef, sessionIdRef]);
}
