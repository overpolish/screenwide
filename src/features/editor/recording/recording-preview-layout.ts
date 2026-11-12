// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

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
type Size = { height: number; width: number };

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
 * on its way never reaches the native side.
 */
export function createRecordingLayoutQueue({
  nextRequestId,
  onError,
}: {
  nextRequestId: () => number;
  onError: (message: string) => void;
}) {
  let disposed = false;
  let inFlight = false;
  let last = "";
  let pending: { layout: RecordingPreviewLayout; window: Size } | null = null;
  let sent: { viewport: Box; window: Size } | null = null;

  const flush = () => {
    if (disposed || inFlight || !pending) return;
    const { layout, window: size } = pending;
    const { viewport } = layout;
    const keepView =
      sent !== null &&
      sent.window.width === size.width &&
      sent.window.height === size.height &&
      sent.viewport.x === viewport.x &&
      sent.viewport.y === viewport.y &&
      sent.viewport.width === viewport.width &&
      sent.viewport.height !== viewport.height;
    sent = { viewport, window: size };
    pending = null;
    inFlight = true;
    void layoutRecordingPreviewSurface({ ...layout, keepView })
      .catch((cause: unknown) => {
        if (!disposed) onError(String(cause));
      })
      .finally(() => {
        inFlight = false;
        flush();
      });
  };

  return {
    dispose: () => {
      disposed = true;
    },
    queue: (layout: RecordingPreviewLayout) => {
      const next = JSON.stringify(layout);
      if (next === last) return;
      last = next;
      pending = {
        layout: { ...layout, requestId: nextRequestId() },
        window: { height: window.innerHeight, width: window.innerWidth },
      };
      flush();
    },
  };
}
