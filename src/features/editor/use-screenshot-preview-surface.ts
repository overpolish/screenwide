// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useEffect, useRef } from "react";

import { AnnotationStyle } from "./annotations";
import {
  layoutScreenshotPreviewSurface,
  refreshScreenshotPreviewSources,
  setScreenshotPreviewEditorSuspended,
  setScreenshotPreviewZoom,
  startScreenshotPreview,
  stopScreenshotPreview,
} from "./api";
import { fitPreviewPane, PreviewPaneFit } from "./components/preview-transform";
import {
  updatePreviewBackdropMasks,
  updatePreviewFitMetadata,
} from "./preview-fit-metadata";
import {
  resetScreenshotPreviewView,
  setScreenshotPreviewFitBasis,
} from "./preview-view-api";
import { PreviewZoomRequest } from "./preview-zoom-state";
import {
  screenshotOutputDimensions,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";
import {
  ScreenshotAnnotationChangeEvent,
  ScreenshotSelectionGestureEvent,
} from "./screenshot-preview-events";
import {
  ResizeFit,
  resizeFitWidth,
  useNativePreviewFit,
} from "./use-native-preview-fit";
import {
  clearBackdropMasks,
  effectiveBackdrop,
} from "./use-recording-preview-surface";
import { useScreenshotPreviewEvents } from "./use-screenshot-preview-events";

export type {
  ScreenshotAnnotationChangeEvent,
  ScreenshotSelectionGestureEvent,
} from "./screenshot-preview-events";

let sessionSequence = 0;

/**
 * The native screenshot editing preview: the composed output renders on the
 * pane surface below the webview (the canvas is only a geometry marker), so
 * every settings change is a single GPU pass with no pixels crossing IPC.
 */
export function useScreenshotPreviewSurface({
  annotationCounterAngle,
  annotationDefaults,
  annotationTool,
  artifactId,
  canvasRef,
  interactionOutput,
  isEditorSuspended = false,
  isEnabled,
  onAnnotationChange,
  onAnnotationHover,
  onPaneFitChange,
  onSelectionChange,
  onSelectionGesture,
  onZoomChange,
  output,
  paneCount = 1,
  selectedAnnotationId,
  selection,
  selectionTargets,
  sourceKey,
  zoomRequest,
}: {
  artifactId: number;
  canvasRef: RefObject<HTMLElement | null>;
  isEnabled: boolean;
  /** The dress the next fresh arrow is drawn in, when the editor has settled
   * on one. It rides along with the layout so the native tool can draw a new
   * arrow in it without a round trip of its own. */
  /** Where a fresh counter's tail points, in radians clockwise from east. */
  annotationCounterAngle?: number | null;
  annotationDefaults?: AnnotationStyle | null;
  /** The annotation tool in hand, when one is. */
  annotationTool?: "arrow" | "counter" | "select";
  interactionOutput?: ScreenshotWorkspaceOutputSettings;
  /**
   * Temporarily hands input back to the webview without giving up the native
   * composition: the interaction view sits above the webview, so while it is
   * showing, DOM controls painted over the viewport (the save overlay's Cancel
   * button) never see the click. Suspending takes that view and the native
   * chrome away and nothing else - the workspace transform keeps running and
   * is still there, untouched, when the suspension clears.
   */
  isEditorSuspended?: boolean;
  /** A finished arrow gesture: the layer's whole list, and what is chosen. */
  onAnnotationChange?: (event: ScreenshotAnnotationChangeEvent) => void;
  /** Which mark the halo is on, or null for none. */
  onAnnotationHover?: (annotationId: string | null) => void;
  /** How small the workspace was drawn to fit the pane, which the toolbar
   * turns into its zoom ceiling. */
  onPaneFitChange?: (fit: PreviewPaneFit) => void;
  onSelectionChange?: (paneIndex: number | null) => void;
  onSelectionGesture?: (event: ScreenshotSelectionGestureEvent) => void;
  onZoomChange?: (zoomPercent: number) => void;
  output?: ScreenshotWorkspaceOutputSettings;
  paneCount?: number;
  selectedAnnotationId?: string | null;
  selection?: Parameters<typeof layoutScreenshotPreviewSurface>[0]["selection"];
  selectionTargets?: Parameters<
    typeof layoutScreenshotPreviewSurface
  >[0]["selectionTargets"];
  sourceKey?: string;
  zoomRequest?: PreviewZoomRequest;
}) {
  const sessionIdRef = useRef(0);
  const startedRef = useRef(false);
  const outputRef = useRef(output);
  outputRef.current = output;
  const interactionOutputRef = useRef(interactionOutput ?? output);
  interactionOutputRef.current = interactionOutput ?? output;
  const paneCountRef = useRef(paneCount);
  paneCountRef.current = paneCount;
  const lastZoomRequestRef = useRef<PreviewZoomRequest | undefined>(undefined);
  const onZoomChangeRef = useRef(onZoomChange);
  onZoomChangeRef.current = onZoomChange;
  const onPaneFitChangeRef = useRef(onPaneFitChange);
  onPaneFitChangeRef.current = onPaneFitChange;
  const onSelectionGestureRef = useRef(onSelectionGesture);
  onSelectionGestureRef.current = onSelectionGesture;
  const onSelectionChangeRef = useRef(onSelectionChange);
  onSelectionChangeRef.current = onSelectionChange;
  const onAnnotationChangeRef = useRef(onAnnotationChange);
  onAnnotationChangeRef.current = onAnnotationChange;
  const onAnnotationHoverRef = useRef(onAnnotationHover);
  onAnnotationHoverRef.current = onAnnotationHover;
  const annotationToolRef = useRef(annotationTool);
  annotationToolRef.current = annotationTool;
  const annotationDefaultsRef = useRef(annotationDefaults);
  annotationDefaultsRef.current = annotationDefaults;
  const annotationAngleRef = useRef(annotationCounterAngle);
  annotationAngleRef.current = annotationCounterAngle;
  const selectedAnnotationIdRef = useRef(selectedAnnotationId);
  selectedAnnotationIdRef.current = selectedAnnotationId;
  const selectionRef = useRef(selection);
  selectionRef.current = selection;
  const selectionTargetsRef = useRef(selectionTargets);
  selectionTargetsRef.current = selectionTargets;
  const editorSuspendedRef = useRef(isEditorSuspended);
  editorSuspendedRef.current = isEditorSuspended;
  const measureRef = useRef<() => void>(() => undefined);
  const layoutRef = useRef<Promise<unknown>>(Promise.resolve());
  const resizeFitRef = useRef<ResizeFit | null>(null);
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
        // Native starts interactive, so a session opened while React owns the
        // workarea has to be told about the suspension straight away.
        if (editorSuspendedRef.current)
          void setScreenshotPreviewEditorSuspended(sessionId, true).catch(
            () => undefined,
          );
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

  useScreenshotPreviewEvents({
    isEnabled,
    onAnnotationChangeRef,
    onAnnotationHoverRef,
    onSelectionChangeRef,
    onSelectionGestureRef,
    onZoomChangeRef,
    sessionIdRef,
  });

  useEffect(() => {
    if (!isEnabled || !startedRef.current || sessionIdRef.current === 0) return;
    void setScreenshotPreviewEditorSuspended(
      sessionIdRef.current,
      isEditorSuspended,
    ).catch(() => undefined);
  }, [isEditorSuspended, isEnabled]);

  useEffect(() => {
    if (
      !isEnabled ||
      isEditorSuspended ||
      !startedRef.current ||
      zoomRequest === undefined ||
      sessionIdRef.current === 0
    )
      return;
    // Reports never create requests. Consume each explicit request once,
    // including across rerenders caused by newer native transform events.
    if (zoomRequest === lastZoomRequestRef.current) return;
    lastZoomRequestRef.current = zoomRequest;
    void setScreenshotPreviewZoom(
      sessionIdRef.current,
      zoomRequest.percent,
    ).catch(() => undefined);
  }, [isEditorSuspended, isEnabled, zoomRequest]);

  useEffect(() => {
    if (!isEnabled) return;
    let disposed = false;
    let inFlight = false;
    let lastLayout = "";
    let pendingLayout:
      Parameters<typeof layoutScreenshotPreviewSurface>[0] | null = null;
    const flush = () => {
      if (disposed || inFlight || !pendingLayout) return;
      const next = pendingLayout;
      pendingLayout = null;
      inFlight = true;
      layoutRef.current = layoutScreenshotPreviewSurface(next)
        .catch(() => undefined)
        .finally(() => {
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
          updatePreviewFitMetadata(
            viewport,
            currentOutput.width,
            currentOutput.height,
          );
          const scale = window.devicePixelRatio || 1;
          const fit = fitPreviewPane({
            natural: screenshotOutputDimensions(currentOutput),
            pixelRatio: scale,
            viewport: viewportRect,
          });
          const pane = fit.pane;
          onPaneFitChangeRef.current?.(fit);
          updatePreviewBackdropMasks(viewportRect);
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
            annotationCounterAngle: annotationAngleRef.current,
            annotationDefaults: annotationDefaultsRef.current,
            annotationTool: annotationToolRef.current,
            backdrop,
            // A suspended editor stays enabled and keeps its transform;
            // only its input and chrome go away, and the suspend command
            // owns that.
            fitWidth: resizeFitWidth(
              resizeFitRef.current,
              viewportRect.width,
              sessionIdRef.current,
            ),
            interactionOutput: interactionOutputRef.current,
            nativeEditor: true,
            output: currentOutput,
            pane,
            resizeFit: resizeFitRef.current,
            scale,
            selectedAnnotationId: selectedAnnotationIdRef.current,
            selection: selectionRef.current,
            selectionTargets: selectionTargetsRef.current,
            sessionId: sessionIdRef.current,
            viewportSurface,
          });
          if (nextLayout !== lastLayout) {
            lastLayout = nextLayout;
            pendingLayout = {
              annotationCounterAngle: annotationAngleRef.current,
              annotationDefaults: annotationDefaultsRef.current,
              annotationTool: annotationToolRef.current,
              backdrop,
              fitWidth: resizeFitWidth(
                resizeFitRef.current,
                viewportRect.width,
                sessionIdRef.current,
              ),
              interactionOutput: interactionOutputRef.current ?? currentOutput,
              nativeEditor: true,
              output: currentOutput,
              panes: Array.from(
                { length: paneCountRef.current },
                (_, index) => ({
                  index,
                  rect: pane,
                }),
              ),
              scale,
              selectedAnnotationId: selectedAnnotationIdRef.current,
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
  }, [canvasRef, isEnabled]);

  useEffect(() => {
    measureRef.current();
  }, [
    annotationCounterAngle,
    annotationDefaults,
    annotationTool,
    outputKey,
    paneCount,
    selectedAnnotationId,
    selection,
    selectionTargets,
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
    if (!isEnabled) return;
    return clearBackdropMasks;
  }, [isEnabled]);

  const { fitDuringResize, fitPreview, setFitBasis } = useNativePreviewFit({
    layoutRef,
    measureRef,
    reset: resetScreenshotPreviewView,
    resizeFitRef,
    sessionIdRef,
    setBasis: setScreenshotPreviewFitBasis,
    startedRef,
  });

  return { fitDuringResize, fitPreview, setFitBasis };
}
