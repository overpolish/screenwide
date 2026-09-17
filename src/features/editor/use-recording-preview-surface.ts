// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { RefObject, useEffect, useRef } from "react";

import {
  layoutRecordingPreviewSurface,
  setRecordingPreviewEditorSuspended,
  setRecordingPreviewZoom,
} from "./api";
import {
  applyBackdropMask,
  clearBackdropMasks,
  effectiveBackdrop,
  Hole,
} from "./preview-backdrop";
import {
  resetRecordingPreviewView,
  setRecordingPreviewFitBasis,
} from "./preview-view-api";
import { PreviewZoomRequest } from "./preview-zoom-state";
import { RecordingOutputSettings } from "./screenshot-output";
import { CameraOverlaySettings } from "./types";
import {
  ResizeFit,
  resizeFitWidth,
  useNativePreviewFit,
} from "./use-native-preview-fit";

export {
  applyBackdropMask,
  clearBackdropMasks,
  effectiveBackdrop,
} from "./preview-backdrop";
export type { Hole } from "./preview-backdrop";

/**
 * The audio-only preview's viewport. It has no pane canvases of its own, so
 * the native ribbon is laid out from this element directly.
 */
const audioRibbonViewport = () =>
  document.querySelector<HTMLElement>("[data-audio-ribbon-viewport]");

export type RecordingSelectionGestureEvent = {
  deltaX: number;
  deltaY: number;
  edges: number;
  operation:
    | "cropMove"
    | "cropResize"
    | "frameRadius"
    | "frameResize"
    | "move"
    | "radius"
    | "resize";
  paneIndex: number;
  phase: "begin" | "update" | "end" | "cancel";
  scale: number;
  cameraOverlay?: CameraOverlaySettings;
  recordingOutput?: RecordingOutputSettings;
};

export type RecordingPreviewSelection = {
  paneIndex: number;
  radiusPercent: number;
  rect: { height: number; width: number; x: number; y: number };
  cropMode?: boolean;
  image?: { height: number; width: number; x: number; y: number };
  layerId?: number;
  maximumScale?: number;
  minimumScale?: number;
  recenterBounds?: { height: number; width: number; x: number; y: number };
};

export function useRecordingPreviewSurface({
  annotationTool,
  bakeCamera,
  cameraCanvasRef,
  cameraOverlay,
  isEditorSuspended = false,
  isEnabled,
  isPlaying = false,
  nativeEditorOwnsLayout = false,
  nativeLayoutHasPanes,
  nativeLayoutKey,
  onError,
  onSelectionChange,
  onSelectionGesture,
  onZoomChange,
  recordingOutput,
  screenCanvasRef,
  selection,
  selectionTargets,
  sessionIdRef,
  startedRef,
  zoomRequest,
}: {
  bakeCamera: boolean;
  cameraCanvasRef: RefObject<HTMLCanvasElement | null>;
  cameraOverlay: CameraOverlaySettings;
  isEnabled: boolean;
  nativeLayoutHasPanes: boolean;
  nativeLayoutKey: string;
  onError: (message: string) => void;
  recordingOutput: RecordingOutputSettings;
  screenCanvasRef: RefObject<HTMLCanvasElement | null>;
  sessionIdRef: RefObject<number>;
  startedRef: RefObject<boolean>;
  /** The annotation tool in hand, when one is. It travels with the layout
   * beside the selection its chrome has to agree with. */
  annotationTool?: "arrow" | "counter" | "select" | null;
  /**
   * Temporarily hands input back to the webview without giving up ownership
   * of the layout: the native interaction view sits above the webview, so
   * while it is showing, DOM controls painted over the viewport (the save
   * overlay's Cancel button) never see the click. Suspending takes that view
   * and the native chrome away and nothing else - the workspace transform
   * keeps running and is still there, untouched, when the suspension clears.
   */
  isEditorSuspended?: boolean;
  isPlaying?: boolean;
  nativeEditorOwnsLayout?: boolean;
  onSelectionChange?: (paneIndex: number | null) => void;
  onSelectionGesture?: (event: RecordingSelectionGestureEvent) => void;
  onZoomChange?: (zoomPercent: number) => void;
  selection?: RecordingPreviewSelection | null;
  selectionTargets?: RecordingPreviewSelection[] | null;
  zoomRequest?: PreviewZoomRequest;
}) {
  const compositionRef = useRef({ bakeCamera, cameraOverlay, recordingOutput });
  const selectionRef = useRef(selection);
  selectionRef.current = isPlaying ? null : selection;
  const selectionTargetsRef = useRef(selectionTargets);
  selectionTargetsRef.current = isPlaying ? null : selectionTargets;
  const annotationToolRef = useRef(annotationTool);
  annotationToolRef.current = isPlaying ? null : annotationTool;
  const onSelectionChangeRef = useRef(onSelectionChange);
  onSelectionChangeRef.current = onSelectionChange;
  const onSelectionGestureRef = useRef(onSelectionGesture);
  onSelectionGestureRef.current = onSelectionGesture;
  const selectionGestureActiveRef = useRef(false);
  const layoutRequestIdRef = useRef(0);
  const measureRef = useRef<() => void>(() => undefined);
  const layoutRef = useRef<Promise<unknown>>(Promise.resolve());
  const resizeFitRef = useRef<ResizeFit | null>(null);
  const lastZoomRequestRef = useRef<PreviewZoomRequest | undefined>(undefined);
  compositionRef.current = { bakeCamera, cameraOverlay, recordingOutput };

  useEffect(() => {
    if (!isEnabled || !nativeEditorOwnsLayout) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<{ sessionId: number; zoomPercent: number }>(
      "recording-preview://transform",
      (event) => {
        if (
          !disposed &&
          event.payload.sessionId === sessionIdRef.current &&
          Number.isFinite(event.payload.zoomPercent)
        ) {
          const roundedZoom = Math.round(event.payload.zoomPercent);
          onZoomChange?.(roundedZoom);
        }
      },
    ).then((cleanup) => {
      if (disposed) cleanup();
      else unlisten = cleanup;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [isEnabled, nativeEditorOwnsLayout, onZoomChange, sessionIdRef]);

  useEffect(() => {
    if (!isEnabled || !nativeEditorOwnsLayout || !startedRef.current) return;
    void setRecordingPreviewEditorSuspended(
      sessionIdRef.current,
      isEditorSuspended,
    ).catch((cause: unknown) => {
      onError(String(cause));
    });
  }, [
    isEditorSuspended,
    isEnabled,
    nativeEditorOwnsLayout,
    onError,
    sessionIdRef,
    startedRef,
  ]);

  useEffect(() => {
    if (
      !isEnabled ||
      !nativeEditorOwnsLayout ||
      isEditorSuspended ||
      zoomRequest === undefined ||
      !startedRef.current
    )
      return;
    // Reports never create requests. Consume each explicit request once,
    // including across rerenders caused by newer native transform events.
    if (zoomRequest === lastZoomRequestRef.current) return;
    lastZoomRequestRef.current = zoomRequest;
    void setRecordingPreviewZoom(
      sessionIdRef.current,
      zoomRequest.percent,
    ).catch((cause: unknown) => {
      onError(String(cause));
    });
  }, [
    isEditorSuspended,
    isEnabled,
    nativeEditorOwnsLayout,
    onError,
    sessionIdRef,
    startedRef,
    zoomRequest,
  ]);

  useEffect(() => {
    if (!isEnabled || !nativeEditorOwnsLayout) return;
    let disposed = false;
    const disposers: (() => void)[] = [];
    void listen<{ paneIndex: number | null; sessionId: number }>(
      "recording-preview://selection-change",
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
      else disposers.push(dispose);
    });
    void listen<
      Omit<RecordingSelectionGestureEvent, "operation"> & {
        operation: number;
        sessionId: number;
      }
    >("recording-preview://selection-gesture", (event) => {
      const payload = event.payload;
      if (
        disposed ||
        payload.sessionId !== sessionIdRef.current ||
        !Number.isFinite(payload.deltaX) ||
        !Number.isFinite(payload.deltaY) ||
        !Number.isInteger(payload.edges) ||
        ![0, 1, 2, 3, 4, 5, 6, 7, 8, 9].includes(payload.operation) ||
        !Number.isInteger(payload.paneIndex) ||
        !Number.isFinite(payload.scale) ||
        !["begin", "update", "end", "cancel"].includes(payload.phase)
      )
        return;
      // Crop is a React-mirrored display mode. Its uncropped composition must
      // be allowed to follow a layer selection immediately; freezing layout
      // here leaves the native OSC on the new layer over the previous layer's
      // pixels until mouse-up.
      if (payload.phase === "begin")
        selectionGestureActiveRef.current = payload.operation < 5;
      else if (payload.phase === "end" || payload.phase === "cancel")
        selectionGestureActiveRef.current = false;
      onSelectionGestureRef.current?.({
        cameraOverlay: payload.cameraOverlay,
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
        ][payload.operation] as RecordingSelectionGestureEvent["operation"],
        paneIndex: payload.paneIndex,
        phase: payload.phase,
        recordingOutput: payload.recordingOutput,
        scale: payload.scale,
      });
      if (payload.phase === "end" || payload.phase === "cancel") {
        // Pointer-rate composition is native, so React-driven layout sync is
        // suppressed during the gesture. Reconcile once React has committed
        // its final semantic frame; otherwise the native state can remain
        // newer than React until some unrelated workarea interaction.
        requestAnimationFrame(() => {
          if (!disposed) measureRef.current();
        });
      }
    }).then((dispose) => {
      if (disposed) dispose();
      else disposers.push(dispose);
    });
    return () => {
      disposed = true;
      for (const dispose of disposers) dispose();
    };
  }, [isEnabled, nativeEditorOwnsLayout, sessionIdRef]);

  useEffect(() => {
    if (!isEnabled) return;
    let animation = 0;
    let disposed = false;
    let inFlight = false;
    let lastLayout = "";
    // Enabling is ownership, not suspension: a suspended editor stays enabled
    // and keeps its transform, and only its input and chrome go away.
    const nativeEditorActive = nativeEditorOwnsLayout;
    let pendingLayout:
      Parameters<typeof layoutRecordingPreviewSurface>[0] | null = null;
    const queueLayout = (
      value: Parameters<typeof layoutRecordingPreviewSurface>[0],
    ) => {
      const nextLayout = JSON.stringify([value, resizeFitRef.current]);
      if (nextLayout === lastLayout) return;
      lastLayout = nextLayout;
      const requestId = ++layoutRequestIdRef.current;
      pendingLayout = { ...value, requestId };
      flush();
    };
    const flush = () => {
      if (disposed || inFlight || !pendingLayout) return;
      const next = pendingLayout;
      pendingLayout = null;
      inFlight = true;
      layoutRef.current = layoutRecordingPreviewSurface(next)
        .catch((cause: unknown) => {
          if (!disposed) onError(String(cause));
        })
        .finally(() => {
          inFlight = false;
          flush();
        });
    };
    const measure = () => {
      // Pointer-rate recording edits are rendered directly by the retained
      // native workspace. ResizeObserver still sees React's live semantic
      // mirror changing the invisible geometry markers; feeding those bounds
      // back into native during the same gesture makes the two owners fight
      // and visibly flicker. Reconcile exactly once from the final React state
      // after end/cancel (scheduled by the gesture listener above).
      if (selectionGestureActiveRef.current) return;
      if (startedRef.current) {
        const connected = [screenCanvasRef.current, cameraCanvasRef.current]
          .map((canvas, index) => ({ canvas, index }))
          .filter(
            ({ canvas }) =>
              canvas?.isConnected && canvas.getBoundingClientRect().width > 0,
          );
        // A recording with no visible video still has a viewport: the native
        // surface fills it with the audio ribbon. It carries no panes, so the
        // marker canvases the video path measures do not exist.
        const ribbonViewport =
          connected.length === 0 ? audioRibbonViewport() : null;
        if (connected.length === 0 && !ribbonViewport) {
          clearBackdropMasks();
          queueLayout({
            annotationTool: annotationToolRef.current,
            backdrop: effectiveBackdrop(),
            ...compositionRef.current,
            nativeEditor: nativeEditorActive,
            panes: [],
            requestId: 0,
            scale: window.devicePixelRatio || 1,
            selection: null,
            sessionId: sessionIdRef.current,
            viewport: { height: 0, width: 0, x: 0, y: 0 },
          });
          return;
        }
        const viewport =
          ribbonViewport ??
          connected[0]?.canvas?.closest<HTMLElement>(
            "[data-recording-preview-viewport]",
          );
        if (viewport) {
          const viewportRect = viewport.getBoundingClientRect();
          const panes = connected.map(({ canvas, index }) => {
            const rect = canvas?.getBoundingClientRect() ?? new DOMRect();
            return {
              index,
              rect: {
                height: rect.height,
                width: rect.width,
                x: rect.left - viewportRect.left,
                y: rect.top - viewportRect.top,
              },
            };
          });
          // Punch the whole viewport, not the pane rects. The native container
          // behind the panes paints the same composited backdrop, so the result
          // looks identical - but a per-pane hole would have to move in lockstep
          // with the panes, and the webview commits its layer tree from another
          // process on its own schedule. During a canvas resize that hole would
          // land a display tick before or after the native pane and shimmer
          // along its edge.
          const holes: Hole[] =
            viewportRect.width >= 1 && viewportRect.height >= 1
              ? [
                  {
                    height: Math.round(viewportRect.height * 100) / 100,
                    width: Math.round(viewportRect.width * 100) / 100,
                    x: 0,
                    y: 0,
                  },
                ]
              : [];
          for (const element of document.querySelectorAll<HTMLElement>(
            "[data-preview-backdrop]",
          )) {
            const elementRect = element.getBoundingClientRect();
            applyBackdropMask(
              element,
              holes.map((hole) => ({
                ...hole,
                x:
                  Math.round((viewportRect.left - elementRect.left) * 100) /
                  100,
                y: Math.round((viewportRect.top - elementRect.top) * 100) / 100,
              })),
            );
          }
          const viewportSurface = {
            height: viewportRect.height,
            width: viewportRect.width,
            x: viewportRect.left,
            y: viewportRect.top,
          };
          const scale = window.devicePixelRatio || 1;
          // One native layout may be in flight at a time. Intermediate DOM
          // positions are replaced by the newest one, and the Rust side also
          // rejects an older request if IPC completion order ever differs.
          queueLayout({
            annotationTool: annotationToolRef.current,
            backdrop: effectiveBackdrop(),
            ...compositionRef.current,
            fitWidth: resizeFitWidth(
              resizeFitRef.current,
              viewportRect.width,
              sessionIdRef.current,
            ),
            nativeEditor: nativeEditorActive,
            panes,
            requestId: 0,
            scale,
            selection: selectionRef.current,
            selectionTargets: selectionTargetsRef.current,
            sessionId: sessionIdRef.current,
            viewport: viewportSurface,
          });
        }
      }
    };
    measureRef.current = measure;
    if (!nativeEditorOwnsLayout) {
      // A fixed Storybook layout: one measure is enough to hand the native
      // side the empty viewport.
      measure();
      return () => {
        disposed = true;
        clearBackdropMasks();
        measureRef.current = () => undefined;
      };
    }
    const observer = new ResizeObserver(() => {
      measure();
    });
    let mutationObserver: MutationObserver | undefined;
    const observeMarkers = () => {
      if (disposed) return;
      const canvases = [
        screenCanvasRef.current,
        cameraCanvasRef.current,
      ].filter(
        (canvas): canvas is HTMLCanvasElement => canvas?.isConnected === true,
      );
      // The audio-only viewport is the one case with a surface but no marker
      // canvases; its own resizes are what the ribbon has to follow.
      const ribbonViewport =
        canvases.length === 0 ? audioRibbonViewport() : null;
      if (!startedRef.current || (canvases.length === 0 && !ribbonViewport)) {
        animation = requestAnimationFrame(observeMarkers);
        return;
      }
      for (const canvas of canvases) observer.observe(canvas);
      const viewport =
        ribbonViewport ??
        canvases[0]?.closest<HTMLElement>("[data-recording-preview-viewport]");
      if (viewport) {
        observer.observe(viewport);
        mutationObserver = new MutationObserver((records) => {
          // measure() writes the backdrop mask's style inside this same
          // subtree; reacting to that write would loop forever.
          const relevant = records.some(
            (record) =>
              !(record.target instanceof HTMLElement) ||
              !record.target.closest("[data-preview-backdrop]"),
          );
          if (!relevant) return;
          for (const canvas of [
            screenCanvasRef.current,
            cameraCanvasRef.current,
          ]) {
            if (canvas?.isConnected) observer.observe(canvas);
          }
          measure();
        });
        // `attributes` matters as much as `childList`: when the workspace is
        // width-constrained, a viewport height change (the timeline loading
        // in) moves the centred marker without resizing it. ResizeObserver
        // only reports size changes, and the viewport-resize measure runs
        // before React has repositioned the marker - so the corrected
        // position only ever reaches native if the marker's style write
        // itself re-triggers a measure.
        mutationObserver.observe(viewport, {
          attributeFilter: ["style"],
          attributes: true,
          childList: true,
          subtree: true,
        });
      }
      measure();
    };
    animation = requestAnimationFrame(observeMarkers);
    return () => {
      disposed = true;
      cancelAnimationFrame(animation);
      mutationObserver?.disconnect();
      observer.disconnect();
      clearBackdropMasks();
      measureRef.current = () => undefined;
    };
  }, [
    cameraCanvasRef,
    isEnabled,
    nativeEditorOwnsLayout,
    nativeLayoutHasPanes,
    nativeLayoutKey,
    onError,
    screenCanvasRef,
    sessionIdRef,
    startedRef,
  ]);

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
    if (!selectionGestureActiveRef.current) measureRef.current();
  }, [
    annotationTool,
    bakeCamera,
    cameraOverlay,
    isPlaying,
    recordingOutput,
    selection,
    selectionTargets,
  ]);

  const { fitDuringResize, fitPreview, setFitBasis } = useNativePreviewFit({
    layoutRef,
    measureRef,
    onError,
    reset: resetRecordingPreviewView,
    resizeFitRef,
    sessionIdRef,
    setBasis: setRecordingPreviewFitBasis,
    startedRef,
  });

  return { fitDuringResize, fitPreview, setFitBasis };
}
