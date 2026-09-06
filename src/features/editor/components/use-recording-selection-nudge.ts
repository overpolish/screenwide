// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef } from "react";

import { RecordingOutputSettings } from "../screenshot-output";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../types";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";

/** Replays a native move gesture so arrow nudges share drag undo semantics. */
export function useRecordingSelectionNudge({
  activeTrack,
  applyGesture,
  cameraOverlay,
  editsBakedCamera,
  gestureAccepted,
  output,
  outputDimensions,
}: {
  activeTrack: RecordingVideoTrackId | null;
  applyGesture: (event: RecordingSelectionGestureEvent) => void;
  cameraOverlay: CameraOverlaySettings;
  editsBakedCamera: boolean;
  gestureAccepted: () => boolean;
  output: RecordingOutputSettings;
  outputDimensions?: Partial<
    Record<RecordingVideoTrackId, { height: number; width: number }>
  >;
}) {
  const contextRef = useRef<{
    applyGesture: (event: RecordingSelectionGestureEvent) => void;
    gestureAccepted: () => boolean;
    origin: object | null;
    outputSize: { height: number; width: number } | undefined;
    paneIndex: number;
  }>({
    applyGesture,
    gestureAccepted,
    origin: null,
    outputSize: undefined,
    paneIndex: 0,
  });
  contextRef.current = {
    applyGesture,
    gestureAccepted,
    origin: !activeTrack
      ? null
      : editsBakedCamera
        ? cameraOverlay
        : output[activeTrack],
    outputSize: activeTrack
      ? outputDimensions?.[editsBakedCamera ? "primary" : activeTrack]
      : undefined,
    paneIndex: activeTrack === "camera" ? 1 : 0,
  };
  const accumulatedRef = useRef<{
    deltaX: number;
    deltaY: number;
    origin: object;
  } | null>(null);
  return useCallback(
    (directionX: number, directionY: number, coarse: boolean) => {
      const { applyGesture, gestureAccepted, origin, outputSize, paneIndex } =
        contextRef.current;
      if (!origin) return;
      const pixels = coarse ? 10 : 1;
      const stepX = outputSize
        ? pixels / Math.max(1, outputSize.width)
        : pixels / 1_000;
      const stepY = outputSize
        ? pixels / Math.max(1, outputSize.height)
        : pixels / 1_000;
      const gesture = {
        edges: 0,
        operation: "move" as const,
        paneIndex,
        scale: 1,
      };
      applyGesture({ ...gesture, deltaX: 0, deltaY: 0, phase: "begin" });
      if (!gestureAccepted()) return;
      const accumulated =
        accumulatedRef.current?.origin === origin
          ? accumulatedRef.current
          : { deltaX: 0, deltaY: 0, origin };
      applyGesture({
        ...gesture,
        deltaX: accumulated.deltaX,
        deltaY: accumulated.deltaY,
        phase: "update",
      });
      accumulated.deltaX += directionX * stepX;
      accumulated.deltaY += directionY * stepY;
      accumulatedRef.current = accumulated;
      applyGesture({
        ...gesture,
        deltaX: accumulated.deltaX,
        deltaY: accumulated.deltaY,
        phase: "end",
      });
    },
    [],
  );
}
