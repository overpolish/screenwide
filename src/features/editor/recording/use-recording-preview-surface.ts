// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { RefObject, useEffect, useRef } from "react";

import { setRecordingPreviewEditorSuspended } from "../api";
import {
  clearBackdropMasks,
  effectiveBackdrop,
  updatePreviewBackdropMasks,
  viewportCorners,
} from "../preview/preview-backdrop";
import { RecordingOutputSettings } from "../screenshot/screenshot-output";
import { CameraOverlaySettings } from "../types";

import { createRecordingLayoutQueue } from "./recording-preview-layout";

import type { RecordingPreviewChrome } from "./recording-preview-layout";
import type { SceneRotation } from "./scenes/recording-scenes";

export {
  clearBackdropMasks,
  effectiveBackdrop,
} from "../preview/preview-backdrop";

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
    | "cropDraw"
    | "cropMove"
    | "cropResize"
    | "frameResize"
    | "move"
    | "resize"
    /** A ring of the rotation gizmo: every sample carries the turn it
     * reaches, in degrees, about X in `deltaX`, Y in `deltaY` and Z in
     * `scale`. */
    | "rotate";
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
  /** The crop window keeps its shape as its handles are dragged. */
  cropKeepsShape?: boolean;
  cropMode?: boolean;
  /** A scene fixes where this pane sits: a move pans its picture and a
   * resize zooms it, and the outline stays put. */
  framed?: boolean;
  image?: { height: number; width: number; x: number; y: number };
  /** A scene places this pane but leaves it free to move and resize, as a
   * custom scene's box: it moves like any layer but never grows the canvas. */
  inScene?: boolean;
  layerId?: number;
  maximumScale?: number;
  minimumScale?: number;
  recenterBounds?: { height: number; width: number; x: number; y: number };
  /** Where a scene turns this pane, which the select tool turns with the
   * rotation gizmo: how it is turned now. */
  rotation?: SceneRotation;
};

export function useRecordingPreviewSurface({
  bakeCamera,
  cameraCanvasRef,
  cameraOverlay,
  chrome,
  isEditorSuspended = false,
  isEnabled,
  isPlaying = false,
  nativeEditorOwnsLayout = false,
  nativeLayoutHasPanes,
  nativeLayoutKey,
  onError,
  onSelectionChange,
  onSelectionGesture,
  recordingOutput,
  screenCanvasRef,
  selection,
  selectionTargets,
  sessionIdRef,
  startedRef,
}: {
  bakeCamera: boolean;
  cameraCanvasRef: RefObject<HTMLCanvasElement | null>;
  cameraOverlay: CameraOverlaySettings;
  chrome: RecordingPreviewChrome;
  isEnabled: boolean;
  nativeLayoutHasPanes: boolean;
  nativeLayoutKey: string;
  onError: (message: string) => void;
  recordingOutput: RecordingOutputSettings;
  screenCanvasRef: RefObject<HTMLCanvasElement | null>;
  sessionIdRef: RefObject<number>;
  startedRef: RefObject<boolean>;
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
  selection?: RecordingPreviewSelection | null;
  selectionTargets?: RecordingPreviewSelection[] | null;
}) {
  const compositionRef = useRef({ bakeCamera, cameraOverlay, recordingOutput });
  const selectionRef = useRef(selection);
  selectionRef.current = isPlaying ? null : selection;
  const selectionTargetsRef = useRef(selectionTargets);
  selectionTargetsRef.current = isPlaying ? null : selectionTargets;
  const chromeRef = useRef(chrome);
  chromeRef.current = isPlaying ? {} : chrome;
  const onSelectionChangeRef = useRef(onSelectionChange);
  onSelectionChangeRef.current = onSelectionChange;
  const onSelectionGestureRef = useRef(onSelectionGesture);
  onSelectionGestureRef.current = onSelectionGesture;
  const selectionGestureActiveRef = useRef(false);
  const layoutRequestIdRef = useRef(0);
  const measureRef = useRef<() => void>(() => undefined);
  compositionRef.current = { bakeCamera, cameraOverlay, recordingOutput };

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
        ![0, 1, 2, 3, 4, 5, 6].includes(payload.operation) ||
        !Number.isInteger(payload.paneIndex) ||
        !Number.isFinite(payload.scale) ||
        !["begin", "update", "end", "cancel"].includes(payload.phase)
      )
        return;
      // Crop is a React-mirrored display mode. Its uncropped composition must
      // be allowed to follow a layer selection immediately; freezing layout
      // here leaves the native OSC on the new layer over the previous layer's
      // pixels until mouse-up.
      // A turn, like a move or a resize, is drawn natively at pointer rate.
      if (payload.phase === "begin")
        selectionGestureActiveRef.current =
          payload.operation < 3 || payload.operation === 6;
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
          "frameResize",
          "cropMove",
          "cropResize",
          "cropDraw",
          "rotate",
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
    const layouts = createRecordingLayoutQueue({
      nextRequestId: () => ++layoutRequestIdRef.current,
      onError,
    });
    const queueLayout = layouts.queue;
    // Enabling is ownership, not suspension: a suspended editor stays enabled
    // and keeps its transform, and only its input and chrome go away.
    const nativeEditorActive = nativeEditorOwnsLayout;
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
            ...chromeRef.current,
            backdrop: effectiveBackdrop(),
            ...compositionRef.current,
            nativeEditor: nativeEditorActive,
            panes: [],
            requestId: 0,
            scale: window.devicePixelRatio || 1,
            selection: null,
            sessionId: sessionIdRef.current,
            viewport: { height: 0, width: 0, x: 0, y: 0 },
            viewportCorners: {
              bottomLeft: 0,
              bottomRight: 0,
              topLeft: 0,
              topRight: 0,
            },
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
          const corners = viewportCorners(viewport);
          updatePreviewBackdropMasks(viewportRect, corners);
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
            ...chromeRef.current,
            backdrop: effectiveBackdrop(),
            ...compositionRef.current,
            nativeEditor: nativeEditorActive,
            panes,
            requestId: 0,
            scale,
            selection: selectionRef.current,
            selectionTargets: selectionTargetsRef.current,
            sessionId: sessionIdRef.current,
            viewport: viewportSurface,
            viewportCorners: corners,
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
        layouts.dispose();
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
      layouts.dispose();
      cancelAnimationFrame(animation);
      mutationObserver?.disconnect();
      observer.disconnect();
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

  // The holes go with the preview, not with a layout change: the effect above
  // re-runs when One video is switched, and holes cleared there would let the
  // window backdrop cover the native panes until the next measure.
  useEffect(() => {
    if (!isEnabled) return;
    return clearBackdropMasks;
  }, [isEnabled]);

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
    bakeCamera,
    cameraOverlay,
    chrome.annotationTool,
    chrome.selectHeld,
    isPlaying,
    recordingOutput,
    selection,
    selectionTargets,
  ]);
}
