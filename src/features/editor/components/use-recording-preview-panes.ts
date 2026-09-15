// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useMemo } from "react";

import { RecordingPreviewLayout, RecordingVideoTrackId } from "../types";

import { RECORDING_PREVIEW_PANE_GAP } from "./recording-preview-layout";

/** The panes that are actually on screen, in layer order, and the workspace
 * they are laid out into: switched-off tracks leave no gap behind them. */
export function useRecordingPreviewPanes({
  cameraCanvasRef,
  layout,
  screenCanvasRef,
  selectedVideoTracks,
  videoTrackOrder,
}: {
  cameraCanvasRef: RefObject<HTMLCanvasElement | null>;
  layout: RecordingPreviewLayout | null;
  screenCanvasRef: RefObject<HTMLCanvasElement | null>;
  selectedVideoTracks: Set<RecordingVideoTrackId>;
  videoTrackOrder: readonly RecordingVideoTrackId[];
}) {
  const canvasRefs = useMemo(
    () => [screenCanvasRef, cameraCanvasRef],
    [cameraCanvasRef, screenCanvasRef],
  );
  const visiblePaneEntries = useMemo(
    () =>
      layout?.panes
        .map((pane, index) => ({
          canvasRef: canvasRefs[index],
          pane,
          trackId: index === 0 ? ("primary" as const) : ("camera" as const),
        }))
        .filter(({ trackId }) => selectedVideoTracks.has(trackId))
        .sort(
          (left, right) =>
            videoTrackOrder.indexOf(left.trackId) -
            videoTrackOrder.indexOf(right.trackId),
        ) ?? [],
    [canvasRefs, layout, selectedVideoTracks, videoTrackOrder],
  );
  const visibleLayout = useMemo(() => {
    if (!layout) return null;
    const height = visiblePaneEntries.reduce(
      (maximum, { pane }) => Math.max(maximum, pane.height),
      0,
    );
    let x = 0;
    const panes = visiblePaneEntries.map(({ pane }) => {
      const visiblePane = { ...pane, x, y: (height - pane.height) / 2 };
      x += pane.width + RECORDING_PREVIEW_PANE_GAP;
      return visiblePane;
    });
    return {
      height,
      panes,
      width: Math.max(0, x - RECORDING_PREVIEW_PANE_GAP),
    };
  }, [layout, visiblePaneEntries]);
  const visibleCanvasRefs = useMemo(
    () => visiblePaneEntries.map(({ canvasRef }) => canvasRef),
    [visiblePaneEntries],
  );
  return { visibleCanvasRefs, visibleLayout, visiblePaneEntries };
}
