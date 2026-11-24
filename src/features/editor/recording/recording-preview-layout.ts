// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { Dispatch, SetStateAction } from "react";

import { pageWindowSize } from "../preview/page-window-size";
import { normalizedCameraOverlay } from "../preview/preview-settings-normalization";
import {
  normalizedScreenshotOutput,
  type RecordingOutputSettings,
} from "../screenshot/screenshot-output";

import type { AnnotationTool } from "../annotations/annotation-defaults";
import type { PreviewSelectionLayout } from "../api";
import type { ViewportCorners } from "../preview/preview-backdrop";
import type { CameraOverlaySettings } from "../types";

// The recording preview's layout, as the page measures it, sent to the
// native surface one at a time: while one is on its way only the newest
// waits, so a burst of resizes never queues behind itself.

type Box = { height: number; width: number; x: number; y: number };

/**
 * The audio-only preview's viewport. It has no pane canvases of its own, so
 * the native ribbon is laid out from this element directly.
 */
export const audioRibbonViewport = () =>
  document.querySelector<HTMLElement>("[data-audio-ribbon-viewport]");

export type RecordingPreviewLayout = {
  backdrop: [number, number, number, number];
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  nativeEditor: boolean;
  panes: { index: number; rect: Box }[];
  recordingOutput: RecordingOutputSettings;
  requestId: number;
  scale: number;
  sessionId: number;
  viewport: Box;
  /** How round the viewport's corners are where it meets the chrome. */
  viewportCorners: ViewportCorners;
  /** The annotation tool in hand, when one is. It rides with the layout so
   * the tool and the selection it has to agree with reach the native chrome
   * together: split across two commands, whichever lands first paints the
   * layer's frame for a frame. */
  annotationTool?: AnnotationTool | null;
  /** The picture stays where it is on screen as the viewport changes: only
   * its bottom edge moved, as the strip under it grew or shrank. */
  keepView?: boolean;
  /** Whether ⌘ (Ctrl) holds Select over the tool: the chosen layer's chrome
   * stands down and a press on it draws a band. */
  selectHeld?: boolean;
  selection?: PreviewSelectionLayout | null;
  selectionTargets?: PreviewSelectionLayout[] | null;
  /** The page's size, width then height, the viewport was measured in. */
  windowSize?: [number, number];
};

/** What decides which chrome owns the picture: the annotation tool in hand
 * and whether ⌘ holds Select over it. Both ride with the layout, beside the
 * selection their chrome has to agree with. */
export type RecordingPreviewChrome = Pick<
  RecordingPreviewLayout,
  "annotationTool" | "selectHeld"
>;

const layoutRecordingPreviewSurface = ({
  annotationTool,
  cameraOverlay,
  keepView,
  recordingOutput,
  selectHeld,
  selection,
  selectionTargets,
  ...layout
}: RecordingPreviewLayout) =>
  invoke<null>("layout_recording_preview_surface", {
    layout: {
      ...layout,
      annotationTool: annotationTool ?? null,
      cameraOverlay: normalizedCameraOverlay(
        cameraOverlay,
        recordingOutput.primary,
      ),
      keepView: keepView ?? false,
      recordingOutput: {
        camera: normalizedScreenshotOutput(recordingOutput.camera),
        primary: normalizedScreenshotOutput(recordingOutput.primary),
      },
      selectHeld: selectHeld ?? false,
      selection: selection ?? null,
      selectionTargets: selectionTargets ?? null,
    },
  });

/**
 * Sends layouts one at a time, the newest waiting while one is on its way,
 * and a layout the same as the last left out. Each is told whether the
 * picture keeps its place: when the window kept its size and only the
 * viewport's bottom edge moved since the last one sent, as the strip under
 * it grew or shrank, so what is under the pointer stays under it. It is
 * worked out against what was sent, since a layout replaced while another is
 * on its way never reaches the native side. The page is measured again
 * (`measure`) on every resize of the window, which the native side carries
 * the viewport with until a layout measured at the window's new size
 * arrives.
 */
export function createRecordingLayoutQueue({
  measure,
  nextRequestId,
  onError,
}: {
  measure: { readonly current: () => void };
  nextRequestId: () => number;
  /** The trouble shown under the picture: a failed layout's, cleared once
   * a later one lands while it is still the one shown, so the picture that
   * is fine again does not keep it and trouble reported since stays. */
  onError: Dispatch<SetStateAction<string | null>>;
}) {
  const remeasure = () => {
    measure.current();
  };
  window.addEventListener("resize", remeasure);
  let disposed = false;
  let failed: string | null = null;
  let inFlight = false;
  let last = "";
  let pending: {
    layout: RecordingPreviewLayout;
    windowSize: [number, number];
  } | null = null;
  let sent: { viewport: Box; windowSize: [number, number] } | null = null;

  const flush = () => {
    if (disposed || inFlight || !pending) return;
    const { layout, windowSize } = pending;
    const { viewport } = layout;
    const keepView =
      sent !== null &&
      sent.windowSize[0] === windowSize[0] &&
      sent.windowSize[1] === windowSize[1] &&
      sent.viewport.x === viewport.x &&
      sent.viewport.y === viewport.y &&
      sent.viewport.width === viewport.width &&
      sent.viewport.height !== viewport.height;
    sent = { viewport, windowSize };
    pending = null;
    inFlight = true;
    void layoutRecordingPreviewSurface({ ...layout, keepView, windowSize })
      .then(() => {
        if (disposed || failed === null) return;
        const cleared = failed;
        onError((shown) => (shown === cleared ? null : shown));
        failed = null;
      })
      .catch((cause: unknown) => {
        if (disposed) return;
        failed = String(cause);
        onError(failed);
      })
      .finally(() => {
        inFlight = false;
        flush();
      });
  };

  return {
    dispose: () => {
      disposed = true;
      window.removeEventListener("resize", remeasure);
    },
    queue: (layout: RecordingPreviewLayout) => {
      const windowSize = pageWindowSize();
      // The window's size is part of what is sent: a resize that leaves the
      // viewport as it was still has to reach the native side.
      const next = JSON.stringify([layout, windowSize]);
      if (next === last) return;
      last = next;
      pending = {
        layout: { ...layout, requestId: nextRequestId() },
        windowSize,
      };
      flush();
    },
  };
}
