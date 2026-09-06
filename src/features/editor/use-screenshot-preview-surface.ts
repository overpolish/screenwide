// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { RefObject, useEffect, useRef } from "react";

import {
  layoutScreenshotPreviewSurface,
  refreshScreenshotPreviewSources,
  setScreenshotPreviewZoom,
  startScreenshotPreview,
  stopScreenshotPreview,
} from "./api";
import { fitPreviewPane, PreviewPaneFit } from "./components/preview-transform";
import {
  screenshotOutputDimensions,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";
import {
  applyBackdropMask,
  clearBackdropMasks,
  effectiveBackdrop,
  Hole,
} from "./use-recording-preview-surface";

let sessionSequence = 0;

export type ScreenshotSelectionGestureEvent = {
  deltaX: number;
  deltaY: number;
  edges: number;
  operation:
    | "cropMove"
    | "cropResize"
    | "frameRadius"
    | "frameResize"
    | "move"
    | "recenterAction"
    | "radius"
    | "resize";
  paneIndex: number;
  phase: "begin" | "update" | "end" | "cancel";
  scale: number;
};

/**
 * The native screenshot editing preview: the composed output renders on the
 * pane surface below the webview (the canvas is only a geometry marker), so
 * every settings change is a single GPU pass with no pixels crossing IPC.
 */
export function useScreenshotPreviewSurface({
  artifactId,
  canvasRef,
  interactionOutput,
  isEditorSuspended = false,
  isEnabled,
  onPaneFitChange,
  onSelectionChange,
  onSelectionGesture,
  onZoomChange,
  output,
  paneCount = 1,
  selection,
  selectionTargets,
  sourceKey,
  zoomPercent,
}: {
  artifactId: number;
  canvasRef: RefObject<HTMLElement | null>;
  isEnabled: boolean;
  interactionOutput?: ScreenshotWorkspaceOutputSettings;
  /**
   * Temporarily hands input back to the webview without giving up the native
   * composition: the interaction view sits above the webview, so while it is
   * showing, DOM controls painted over the viewport (the save overlay's Cancel
   * button) never see the click. The next layout turns the native editor off,
   * and the one after the suspension clears turns it back on - along with the
   * workspace zoom, which the native side resets while the editor is inactive.
   */
  isEditorSuspended?: boolean;
  /** How small the workspace was drawn to fit the pane, which the toolbar
   * turns into its zoom ceiling. */
  onPaneFitChange?: (fit: PreviewPaneFit) => void;
  onSelectionChange?: (paneIndex: number | null) => void;
  onSelectionGesture?: (event: ScreenshotSelectionGestureEvent) => void;
  onZoomChange?: (zoomPercent: number) => void;
  output?: ScreenshotWorkspaceOutputSettings;
  paneCount?: number;
  selection?: Parameters<typeof layoutScreenshotPreviewSurface>[0]["selection"];
  selectionTargets?: Parameters<
    typeof layoutScreenshotPreviewSurface
  >[0]["selectionTargets"];
  sourceKey?: string;
  zoomPercent?: number;
}) {
  const sessionIdRef = useRef(0);
  const startedRef = useRef(false);
  const outputRef = useRef(output);
  outputRef.current = output;
  const interactionOutputRef = useRef(interactionOutput ?? output);
  interactionOutputRef.current = interactionOutput ?? output;
  const paneCountRef = useRef(paneCount);
  paneCountRef.current = paneCount;
  const lastNativeZoomRef = useRef<number | undefined>(undefined);
  const zoomPercentRef = useRef(zoomPercent);
  zoomPercentRef.current = zoomPercent;
  const onZoomChangeRef = useRef(onZoomChange);
  onZoomChangeRef.current = onZoomChange;
  const onPaneFitChangeRef = useRef(onPaneFitChange);
  onPaneFitChangeRef.current = onPaneFitChange;
  const onSelectionGestureRef = useRef(onSelectionGesture);
  onSelectionGestureRef.current = onSelectionGesture;
  const onSelectionChangeRef = useRef(onSelectionChange);
  onSelectionChangeRef.current = onSelectionChange;
  const selectionRef = useRef(selection);
  selectionRef.current = selection;
  const selectionTargetsRef = useRef(selectionTargets);
  selectionTargetsRef.current = selectionTargets;
  const editorSuspendedRef = useRef(isEditorSuspended);
  const pendingZoomRestoreRef = useRef(false);
  const measureRef = useRef<() => void>(() => undefined);
  const outputKey = JSON.stringify(output);

  useEffect(() => {
    if (!isEnabled) return;
    let disposed = false;
    const sessionId = Date.now() * 1_000 + (++sessionSequence % 1_000);
    sessionIdRef.current = sessionId;
    void startScreenshotPreview(artifactId, sessionId)
      .then(() => {
        if (disposed) return;
        startedRef.current = true;
        measureRef.current();
        return refreshScreenshotPreviewSources(artifactId, sessionId);
      })
      .catch(() => undefined);
    return () => {
      disposed = true;
      startedRef.current = false;
      void stopScreenshotPreview(sessionId).catch(() => undefined);
    };
  }, [artifactId, isEnabled]);

  useEffect(() => {
    if (!isEnabled || !startedRef.current) return;
    void refreshScreenshotPreviewSources(
      artifactId,
      sessionIdRef.current,
    ).catch(() => undefined);
  }, [artifactId, isEnabled, sourceKey]);

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
          lastNativeZoomRef.current = roundedZoom;
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
  }, [isEnabled]);

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
  }, [isEnabled]);

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
          "recenterAction",
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
  }, [isEnabled]);

  useEffect(() => {
    if (
      !isEnabled ||
      isEditorSuspended ||
      !startedRef.current ||
      zoomPercent === undefined ||
      sessionIdRef.current === 0
    )
      return;
    // React only ever holds native's zoom rounded to a whole percent. A value
    // equal to the last one native reported is that echo, not a new request;
    // sending it back would replace native's exact zoom with the rounded one.
    if (zoomPercent === lastNativeZoomRef.current) return;
    void setScreenshotPreviewZoom(sessionIdRef.current, zoomPercent).catch(
      () => undefined,
    );
  }, [isEditorSuspended, isEnabled, zoomPercent]);

  useEffect(() => {
    if (!isEnabled) return;
    let disposed = false;
    let inFlight = false;
    let lastLayout = "";
    const nativeEditorActive = !isEditorSuspended;
    // The native editor keeps no transform while it is inactive, so the zoom
    // React still shows has to be pushed again once this layout has turned the
    // editor back on. Riding the layout's completion rather than a second
    // effect keeps the two in order: the zoom command is dropped outright if
    // it reaches the surface first.
    if (nativeEditorActive && editorSuspendedRef.current && startedRef.current)
      pendingZoomRestoreRef.current = true;
    editorSuspendedRef.current = isEditorSuspended;
    let pendingLayout:
      Parameters<typeof layoutScreenshotPreviewSurface>[0] | null = null;
    const flush = () => {
      if (disposed || inFlight || !pendingLayout) return;
      const next = pendingLayout;
      pendingLayout = null;
      inFlight = true;
      void layoutScreenshotPreviewSurface(next)
        .catch(() => undefined)
        .finally(() => {
          if (
            !disposed &&
            nativeEditorActive &&
            pendingZoomRestoreRef.current
          ) {
            pendingZoomRestoreRef.current = false;
            const zoom = zoomPercentRef.current;
            if (zoom !== undefined) {
              void setScreenshotPreviewZoom(sessionIdRef.current, zoom).catch(
                () => undefined,
              );
            }
          }
          inFlight = false;
          flush();
        });
    };
    const measure = () => {
      const marker = canvasRef.current;
      const currentOutput = outputRef.current;
      if (
        startedRef.current &&
        currentOutput &&
        marker?.isConnected &&
        marker.getBoundingClientRect().width > 0
      ) {
        const viewport = marker.matches("[data-recording-preview-viewport]")
          ? marker
          : marker.closest<HTMLElement>("[data-recording-preview-viewport]");
        if (viewport) {
          const viewportRect = viewport.getBoundingClientRect();
          const scale = window.devicePixelRatio || 1;
          const fit = fitPreviewPane({
            natural: screenshotOutputDimensions(currentOutput),
            pixelRatio: scale,
            viewport: viewportRect,
          });
          const pane = fit.pane;
          onPaneFitChangeRef.current?.(fit);
          for (const element of document.querySelectorAll<HTMLElement>(
            "[data-preview-backdrop]",
          )) {
            const elementRect = element.getBoundingClientRect();
            const holes: Hole[] =
              viewportRect.width >= 1 && viewportRect.height >= 1
                ? [
                    {
                      height: Math.round(viewportRect.height * 100) / 100,
                      width: Math.round(viewportRect.width * 100) / 100,
                      x:
                        Math.round(
                          (viewportRect.left - elementRect.left) * 100,
                        ) / 100,
                      y:
                        Math.round((viewportRect.top - elementRect.top) * 100) /
                        100,
                    },
                  ]
                : [];
            applyBackdropMask(element, holes);
          }
          const viewportSurface = {
            height: viewportRect.height,
            width: viewportRect.width,
            x: viewportRect.left,
            y: viewportRect.top,
          };
          const backdrop = effectiveBackdrop();
          // A fresh capture with the same dimensions and default layout
          // produces the same geometry as the last one sent, but the native
          // session behind it is new and holds no selection yet. Keying the
          // dedupe on the session makes the first layout of every session
          // reach the surface.
          const nextLayout = JSON.stringify({
            backdrop,
            interactionOutput: interactionOutputRef.current,
            nativeEditor: nativeEditorActive,
            output: currentOutput,
            pane,
            scale,
            selection: selectionRef.current,
            selectionTargets: selectionTargetsRef.current,
            sessionId: sessionIdRef.current,
            viewportSurface,
          });
          if (nextLayout !== lastLayout) {
            lastLayout = nextLayout;
            pendingLayout = {
              backdrop,
              interactionOutput: interactionOutputRef.current ?? currentOutput,
              nativeEditor: nativeEditorActive,
              output: currentOutput,
              panes: Array.from(
                { length: paneCountRef.current },
                (_, index) => ({
                  index,
                  rect: pane,
                }),
              ),
              scale,
              selection: selectionRef.current,
              selectionTargets: selectionTargetsRef.current,
              sessionId: sessionIdRef.current,
              viewport: viewportSurface,
            };
            flush();
          }
        }
      }
    };
    measureRef.current = measure;
    const observer = new ResizeObserver(measure);
    const marker = canvasRef.current;
    if (marker) observer.observe(marker);
    measure();
    return () => {
      disposed = true;
      observer.disconnect();
      measureRef.current = () => undefined;
    };
  }, [canvasRef, isEditorSuspended, isEnabled]);

  useEffect(() => {
    measureRef.current();
  }, [outputKey, paneCount, selection, selectionTargets]);

  useEffect(() => {
    if (!isEnabled) return;
    let animation = 0;
    const updateAppearance = () => {
      cancelAnimationFrame(animation);
      animation = requestAnimationFrame(() => {
        measureRef.current();
      });
    };
    window.addEventListener("screenwide-theme-changed", updateAppearance);
    return () => {
      cancelAnimationFrame(animation);
      window.removeEventListener("screenwide-theme-changed", updateAppearance);
    };
  }, [isEnabled]);

  useEffect(() => {
    if (!isEnabled) return;
    return clearBackdropMasks;
  }, [isEnabled]);
}
