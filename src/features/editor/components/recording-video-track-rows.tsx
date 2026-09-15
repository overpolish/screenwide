// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, Monitor } from "lucide-react";

import { RecordingVideoTrackId } from "../types";

import { RecordingTrackLanesProps } from "./recording-track-lanes-contract";
import { TimelineTrackHeader } from "./timeline-track-header";
import { TimelineVideoClip } from "./timeline-video-clip";
import { TimelineViewportState } from "./timeline-viewport";
import { recordingTrackMoves } from "./use-recording-track-menu";
import { useRecordingTrackReorder } from "./use-recording-track-reorder";

type VideoTrackRowsProps = Pick<
  RecordingTrackLanesProps,
  | "blade"
  | "enabledTracks"
  | "enabledVideoTracks"
  | "layout"
  | "onEnabledVideoTracksChange"
  | "onSelectedTrackChange"
  | "onVideoTrackOrderChange"
  | "selectedTrack"
  | "thumbnails"
  | "videoTrackOrder"
> & { viewport: TimelineViewportState };

export function RecordingVideoTrackRows({
  blade,
  enabledTracks,
  enabledVideoTracks,
  layout,
  onEnabledVideoTracksChange,
  onSelectedTrackChange,
  onVideoTrackOrderChange,
  selectedTrack,
  thumbnails,
  videoTrackOrder,
  viewport,
}: VideoTrackRowsProps) {
  // The screen and camera panes as rows, in the order the edit keeps them -
  // the only rows a drag can reorder.
  const videoRows = layout.panes
    .map((pane, index) => ({
      pane,
      trackId: index === 0 ? ("primary" as const) : ("camera" as const),
    }))
    .sort(
      (left, right) =>
        videoTrackOrder.indexOf(left.trackId) -
        videoTrackOrder.indexOf(right.trackId),
    );
  const {
    beginDrag,
    cancelDrag,
    drag,
    finishDrag,
    openTrackMenu,
    rowElementsRef,
    updateDrag,
  } = useRecordingTrackReorder(
    videoTrackOrder,
    videoRows,
    onVideoTrackOrderChange,
  );
  return (
    <>
      {videoRows.map(({ pane, trackId }, rowIndex) => {
        const Icon = pane.kind === "camera" ? Camera : Monitor;
        const label = pane.kind === "camera" ? "Camera" : "Screen";
        const enabled = enabledVideoTracks.has(trackId);
        const mustRemainEnabled =
          enabled && enabledVideoTracks.size === 1 && enabledTracks.size === 0;
        return (
          <div
            className={`relative flex items-center gap-section transition-opacity ${drag?.source === trackId ? "opacity-50" : ""}`}
            key={trackId}
            onContextMenu={(event) => {
              event.preventDefault();
              onSelectedTrackChange(trackId);
              void openTrackMenu(
                { x: event.clientX, y: event.clientY },
                trackId,
                recordingTrackMoves(videoTrackOrder, trackId),
              );
            }}
            ref={(element) => {
              if (element) rowElementsRef.current.set(trackId, element);
              else rowElementsRef.current.delete(trackId);
            }}
          >
            {/* Where the dragged track would land, drawn in the gap between
                rows in the accent the app selects with. */}
            {drag?.dropIndex === rowIndex ? (
              <div className="pointer-events-none absolute -top-tight right-0 left-0 z-20 h-tight rounded-control bg-primary" />
            ) : null}
            {rowIndex === videoRows.length - 1 &&
            drag?.dropIndex === videoRows.length ? (
              <div className="pointer-events-none absolute -bottom-tight right-0 left-0 z-20 h-tight rounded-control bg-primary" />
            ) : null}
            <TimelineTrackHeader
              // The press that starts a reorder is taken on the way down: the
              // button that selects the track stops the bubbling one.
              dragProps={{
                onPointerCancel: cancelDrag,
                onPointerDownCapture: beginDrag(trackId),
                onPointerMove: updateDrag,
                onPointerUp: finishDrag,
              }}
              icon={<Icon />}
              inclusion={{
                isIncluded: enabled,
                isRequired: mustRemainEnabled,
                onChange: () => {
                  const next = new Set<RecordingVideoTrackId>(
                    enabledVideoTracks,
                  );
                  if (next.has(trackId)) {
                    if (mustRemainEnabled) return;
                    next.delete(trackId);
                  } else next.add(trackId);
                  onEnabledVideoTracksChange(next);
                },
              }}
              isDragging={drag?.source === trackId}
              isSelected={selectedTrack === trackId}
              label={label}
              onSelect={() => {
                onSelectedTrackChange(trackId);
              }}
            />
            <TimelineVideoClip
              blade={blade}
              enabled={enabled}
              onSelect={onSelectedTrackChange}
              selected={selectedTrack === trackId}
              thumbnails={thumbnails[trackId]}
              trackId={trackId}
              viewport={viewport}
            />
          </div>
        );
      })}
    </>
  );
}
