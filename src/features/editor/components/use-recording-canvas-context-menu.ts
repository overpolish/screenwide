// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { dismissPopupMenu } from "../../popup-panel/use-popup-menu";
import { RecordingTrackId, RecordingVideoTrackId } from "../types";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import {
  RECORDING_TRACK_MENU_PREFIX,
  recordingTrackMoves,
  useRecordingTrackMenu,
} from "./use-recording-track-menu";

export function useRecordingCanvasContextMenu({
  canvasTool,
  moveVideoTrack,
  onSelectedTrackChange,
  videoTrackOrderList,
  visiblePaneEntries,
}: {
  canvasTool: RecordingCanvasTool;
  moveVideoTrack: (
    track: RecordingVideoTrackId,
    direction: "backward" | "forward",
  ) => void;
  videoTrackOrderList: RecordingVideoTrackId[];
  visiblePaneEntries: { trackId: RecordingVideoTrackId }[];
  onSelectedTrackChange?: (track: RecordingTrackId | null) => void;
}) {
  // A right click on a pane in the native canvas opens the same layer menu the
  // timeline row opens. The native side selects the layer it landed on and
  // reports the point; the menu is drawn here, at the pointer.
  const openTrackMenu = useRecordingTrackMenu(moveVideoTrack);
  const openCanvasTrackMenuRef = useRef<
    (paneIndex: number, x: number, y: number) => void
  >(() => undefined);
  openCanvasTrackMenuRef.current = (paneIndex, x, y) => {
    if (canvasTool !== "select") return;
    const trackId =
      paneIndex === 0 ? "primary" : paneIndex === 1 ? "camera" : null;
    if (
      !trackId ||
      !visiblePaneEntries.some((entry) => entry.trackId === trackId)
    )
      return;
    onSelectedTrackChange?.(trackId);
    void openTrackMenu(
      { x, y },
      trackId,
      recordingTrackMoves(videoTrackOrderList, trackId),
    );
  };
  // The layer menu belongs to the Select tool: putting the tool down takes
  // the menu with it rather than leaving it open over nothing.
  useEffect(() => {
    if (canvasTool !== "select")
      void dismissPopupMenu(RECORDING_TRACK_MENU_PREFIX);
  }, [canvasTool]);
  useEffect(() => {
    // The subscription lands after a hop. A cleanup that runs before it
    // lands, as React's development double-mount does, must still let go of
    // it, or the window hears every click twice and the menu opens and
    // closes in one go.
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void getCurrentWindow()
      .listen<{ paneIndex: number; x: number; y: number }>(
        "preview://context-menu",
        ({ payload }) => {
          openCanvasTrackMenuRef.current(
            payload.paneIndex,
            payload.x,
            payload.y,
          );
        },
      )
      .then((dispose) => {
        if (disposed) dispose();
        else unlisten = dispose;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
