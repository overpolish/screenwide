// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingVideoTrackId } from "../../types";

import { isOverlayLayer, OverlayLayerId } from "./overlay-layers";
import { RecordingCanvasTool } from "./recording-canvas-tool";

/** What a press on the picture chooses, as the native player reports the
 * pane it landed on, or none. */
export const pressedPane =
  ({
    activeVideoTrack,
    canvasTool,
    clearAnnotations,
    clearOverlaySelection,
    composedVideoTracks,
    onSelectedTrackChange,
    pressOverlay,
  }: {
    activeVideoTrack: RecordingVideoTrackId | null;
    canvasTool: RecordingCanvasTool;
    clearAnnotations: () => void;
    clearOverlaySelection: () => void;
    composedVideoTracks: Set<RecordingVideoTrackId>;
    pressOverlay: (layerId: OverlayLayerId) => void;
    onSelectedTrackChange?: (track: RecordingVideoTrackId | null) => void;
  }) =>
  (paneIndex: number | null) => {
    // Nothing chosen with Select in hand lets go of everything at once: the
    // annotations, the track, and with it the Screen or Camera mode it
    // opened. The engine reports it as one change, so the editor never draws
    // the track chosen without its annotations in between.
    if (paneIndex === null) {
      if (canvasTool !== "select") return;
      clearAnnotations();
      if (activeVideoTrack !== null) onSelectedTrackChange?.(null);
      return;
    }
    // An overlay takes its size and place from its tool, so a press on it
    // chooses it and opens that tool rather than taking up a pane.
    if (isOverlayLayer(paneIndex)) {
      pressOverlay(paneIndex);
      return;
    }
    clearAnnotations();
    clearOverlaySelection();
    const trackId = paneIndex === 0 ? "primary" : "camera";
    if (composedVideoTracks.has(trackId)) onSelectedTrackChange?.(trackId);
  };
