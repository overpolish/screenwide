// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cameraOverlayGeometry } from "../camera-overlay-geometry";
import {
  RecordingOutputSettings,
  screenshotOutputDimensions,
} from "../screenshot-output";
import { isAnnotationTool } from "../tool-panels/tool-registry";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../types";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import { normalizedRecordingSelection } from "./recording-selection";

import type { AnnotationKind } from "../../../components/shared/annotation-style/types";

const FRAME_LAYER_ID = 0xffffffff;

type VideoSourceDimensions = Partial<
  Record<RecordingVideoTrackId, { height: number; width: number }>
>;

/**
 * Where the baked camera sits inside the screen output, as shares of it.
 *
 * The overlay draws this rect and the hit targets test against it, so both
 * measure it here rather than each deriving it from the overlay geometry.
 */
function bakedCameraSelection({
  cameraOverlay,
  cameraSource,
  isCropping,
  output,
  primarySource,
}: {
  cameraOverlay: CameraOverlaySettings;
  cameraSource: { height: number; width: number };
  isCropping: boolean;
  output: { height: number; width: number };
  primarySource: { height: number; width: number };
}) {
  const geometry = cameraOverlayGeometry(
    {
      height: output.height,
      kind: "screen",
      sourceHeight: primarySource.height,
      sourceWidth: primarySource.width,
      width: output.width,
      x: 0,
      y: 0,
    },
    {
      height: cameraSource.height,
      kind: "camera",
      sourceHeight: cameraSource.height,
      sourceWidth: cameraSource.width,
      width: cameraSource.width,
      x: 0,
      y: 0,
    },
    cameraOverlay,
  );
  const share = (rect: {
    height: number;
    width: number;
    x: number;
    y: number;
  }) => ({
    height: rect.height / Math.max(1, output.height),
    width: rect.width / Math.max(1, output.width),
    x: rect.x / Math.max(1, output.width),
    y: rect.y / Math.max(1, output.height),
  });
  return {
    cropMode: isCropping,
    image: share(geometry.camera),
    layerId: 1,
    paneIndex: 0,
    radiusPercent: cameraOverlay.radiusPercent,
    rect: share(geometry.frame),
  };
}

/**
 * The one selection the native overlay draws for the video panes: the frame
 * handle under the Frame tool, the baked camera's own rect when the camera is
 * composited into the screen output, or the selected pane's own rect.
 */
/** Only the crop tool draws its own layer chrome. A drawing tool leaves it in
 * the select tool's hands: the annotations are drawn over the picture, and the
 * layer underneath is still the thing a press outside one acts on. */
const selectionChromeTool = (tool: AnnotationKind | "crop" | "select") =>
  tool === "crop" ? "crop" : "select";

export function recordingVideoSelectionOverlay({
  activeVideoTrack,
  cameraOverlay,
  canPreviewBakedCamera,
  canvasTool,
  effectiveRecordingOutput,
  previewSourceDimensions,
  selectedVideoTracks,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  cameraOverlay: CameraOverlaySettings;
  canPreviewBakedCamera: boolean;
  canvasTool: RecordingCanvasTool;
  effectiveRecordingOutput: RecordingOutputSettings;
  previewSourceDimensions: VideoSourceDimensions;
  selectedVideoTracks: ReadonlySet<RecordingVideoTrackId>;
}) {
  if (canvasTool === "canvas") {
    // Frame is a synthetic selection, just as in the screenshot workspace.
    // In baked mode it always belongs to the primary output; in split mode
    // it belongs to the currently selected video frame.
    const frameTrack = canPreviewBakedCamera ? "primary" : activeVideoTrack;
    if (!frameTrack || !selectedVideoTracks.has(frameTrack)) return null;
    return {
      layerId: FRAME_LAYER_ID,
      paneIndex: frameTrack === "primary" ? 0 : 1,
      radiusPercent: 0,
      rect: { height: 1, width: 1, x: 0, y: 0 },
    };
  }
  if (
    (canvasTool !== "crop" && !isAnnotationTool(canvasTool)) ||
    !activeVideoTrack ||
    !selectedVideoTracks.has(activeVideoTrack)
  )
    return null;
  const primaryOutput = screenshotOutputDimensions(
    effectiveRecordingOutput.primary,
  );
  const primarySource = previewSourceDimensions.primary;
  if (!primarySource) return null;
  if (canPreviewBakedCamera) {
    if (activeVideoTrack === "primary") {
      return normalizedRecordingSelection({
        mode: selectionChromeTool(canvasTool),
        output: effectiveRecordingOutput.primary,
        paneIndex: 0,
        source: primarySource,
      });
    }
    const cameraSource = previewSourceDimensions.camera;
    if (!cameraSource) return null;
    return bakedCameraSelection({
      cameraOverlay,
      cameraSource,
      isCropping: canvasTool === "crop",
      output: primaryOutput,
      primarySource,
    });
  }
  const paneIndex = activeVideoTrack === "primary" ? 0 : 1;
  const source =
    activeVideoTrack === "primary"
      ? primarySource
      : previewSourceDimensions.camera;
  if (!source) return null;
  return normalizedRecordingSelection({
    mode: selectionChromeTool(canvasTool),
    output: effectiveRecordingOutput[activeVideoTrack],
    paneIndex,
    source,
  });
}

/**
 * Every rect the native overlay will accept a gesture on, whether or not it is
 * the one currently selected.
 */
export function recordingVideoSelectionTargets({
  cameraOverlay,
  canPreviewBakedCamera,
  canvasTool,
  effectiveRecordingOutput,
  previewSourceDimensions,
  selectedVideoTracks,
}: {
  cameraOverlay: CameraOverlaySettings;
  canPreviewBakedCamera: boolean;
  canvasTool: RecordingCanvasTool;
  effectiveRecordingOutput: RecordingOutputSettings;
  previewSourceDimensions: VideoSourceDimensions;
  selectedVideoTracks: ReadonlySet<RecordingVideoTrackId>;
}) {
  if (canvasTool === "canvas")
    return (["primary", "camera"] as const).flatMap((trackId) =>
      selectedVideoTracks.has(trackId) && previewSourceDimensions[trackId]
        ? [
            {
              layerId: FRAME_LAYER_ID,
              paneIndex: trackId === "primary" ? 0 : 1,
              radiusPercent: 0,
              rect: { height: 1, width: 1, x: 0, y: 0 },
            },
          ]
        : [],
    );
  if (canvasTool !== "select" && canvasTool !== "crop") return null;
  if (canPreviewBakedCamera) {
    const primarySource = previewSourceDimensions.primary;
    const cameraSource = previewSourceDimensions.camera;
    if (!primarySource || !cameraSource) return null;
    const output = screenshotOutputDimensions(effectiveRecordingOutput.primary);
    return [
      normalizedRecordingSelection({
        mode: canvasTool,
        output: effectiveRecordingOutput.primary,
        paneIndex: 0,
        source: primarySource,
      }),
      bakedCameraSelection({
        cameraOverlay,
        cameraSource,
        isCropping: canvasTool === "crop",
        output,
        primarySource,
      }),
    ];
  }
  return (["primary", "camera"] as const).flatMap((trackId) => {
    if (!selectedVideoTracks.has(trackId)) return [];
    const source = previewSourceDimensions[trackId];
    if (!source) return [];
    return [
      normalizedRecordingSelection({
        mode: canvasTool,
        output: effectiveRecordingOutput[trackId],
        paneIndex: trackId === "primary" ? 0 : 1,
        source,
      }),
    ];
  });
}
