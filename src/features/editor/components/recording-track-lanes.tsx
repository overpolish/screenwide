// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, Keyboard, Monitor } from "lucide-react";
import {
  memo,
  PointerEvent as ReactPointerEvent,
  useRef,
  useState,
} from "react";

import {
  recordingAudioStreamIndex,
  recordingAudioTrackId,
  RecordingVideoTrackId,
} from "../types";

import { RecordingTrackLanesProps } from "./recording-track-lanes-contract";
import { ScrubAudioTracks } from "./scrub-audio-tracks";
import { TIMED_LANE_ROW_HEIGHT_PX } from "./timed-lane-layout";
import { TimelineAudioMeter } from "./timeline-audio-meter";
import { TimelineItemLane } from "./timeline-item-lane";
import { TimelineScrubberOverlay } from "./timeline-scrubber";
import {
  TIMELINE_TRACK_SWITCH_SELECTOR,
  TimelineTrackHeader,
} from "./timeline-track-header";
import { TimelineVideoClip } from "./timeline-video-clip";
import { TimelineHeader } from "./timeline-zoom-toolbar";
import {
  recordingTrackMoves,
  useRecordingTrackMenu,
} from "./use-recording-track-menu";
import { useTimedLaneRows } from "./use-timed-lane-rows";
import { useTimelineNavigation } from "./use-timeline-navigation";

/**
 * The band's row rhythm in CSS pixels. Every row is `--spacing-control-height`
 * tall and rows are separated by `--spacing-control`; the meter beside the
 * lanes is sized in JS, so it needs what those tokens resolve to.
 */
const ROW_GAP_PX = 4;

/** Movement that turns a press on a track header into a reorder. */
const DRAG_THRESHOLD_PX = 4;

/** Memoized because pointer-rate canvas settings do not affect this subtree. */
export const RecordingTrackLanes = memo(function RecordingTrackLanes({
  adjustedKeyboardFragmentIds,
  audioTracks,
  blade,
  durationMs,
  enabledTracks,
  enabledVideoTracks,
  hiddenKeyboardFragmentIds,
  hiddenKeyboardItemIds,
  keyboardItems,
  keyboardSelection,
  layout,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onSeek,
  onSelectedTrackChange,
  onVideoTrackOrderChange,
  playhead,
  selectedTrack,
  sourceDurationMs,
  thumbnails,
  videoTrackOrder,
  volumes,
}: RecordingTrackLanesProps) {
  const [drag, setDrag] = useState<{
    dropIndex: number;
    source: RecordingVideoTrackId;
  } | null>(null);
  const timeline = useTimelineNavigation(blade.edit.artifactId);
  const dragRef = useRef<{
    dropIndex: number;
    pointerId: number;
    source: RecordingVideoTrackId;
    startY: number;
    started: boolean;
  } | null>(null);
  const rowElementsRef = useRef(
    new Map<RecordingVideoTrackId, HTMLDivElement>(),
  );
  // The shortcut lane stacks overlapping badges into sublanes, so the meter
  // must cover however tall it grows; derived from the same shared stacking
  // the lane itself renders from.
  const keyboardRows = useTimedLaneRows({
    edit: blade.edit,
    hiddenFragmentIds: hiddenKeyboardFragmentIds,
    hiddenItemIds: hiddenKeyboardItemIds,
    items: keyboardItems,
    sourceDurationMs,
  });
  const keyboardRowCount = keyboardItems.length > 0 ? keyboardRows.rowCount : 0;
  const rowCount =
    layout.panes.length + audioTracks.length + (keyboardRowCount > 0 ? 1 : 0);
  // The meter stands beside the whole column: the toolbar row, every track
  // row under it, and the sublanes the shortcut lane grows by.
  const meterHeight =
    (rowCount + 1) * TIMED_LANE_ROW_HEIGHT_PX +
    rowCount * ROW_GAP_PX +
    Math.max(0, keyboardRowCount - 1) * TIMED_LANE_ROW_HEIGHT_PX;
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
  const applyOrder = (source: RecordingVideoTrackId, dropIndex: number) => {
    const sourceIndex = videoTrackOrder.indexOf(source);
    if (sourceIndex === -1) return;
    const next = videoTrackOrder.filter((track) => track !== source);
    const insertionIndex = Math.max(
      0,
      Math.min(
        next.length,
        dropIndex > sourceIndex ? dropIndex - 1 : dropIndex,
      ),
    );
    next.splice(insertionIndex, 0, source);
    if (next.some((track, index) => track !== videoTrackOrder[index]))
      onVideoTrackOrderChange?.(next);
  };
  const moveTrack = (
    track: RecordingVideoTrackId,
    direction: "backward" | "forward",
  ) => {
    const index = videoTrackOrder.indexOf(track);
    const nextIndex = direction === "forward" ? index - 1 : index + 1;
    if (index === -1 || nextIndex < 0 || nextIndex >= videoTrackOrder.length)
      return;
    const next = [...videoTrackOrder];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    onVideoTrackOrderChange?.(next);
  };
  const openTrackMenu = useRecordingTrackMenu(moveTrack);
  const beginDrag =
    (source: RecordingVideoTrackId) =>
    (event: ReactPointerEvent<HTMLDivElement>) => {
      if (event.button !== 0 || videoRows.length < 2) return;
      // The switch acts on the track it sits in, so a press on it is never
      // the start of a reorder: capturing the pointer here would take the
      // switch's click with it. The rest of the header is the drag handle,
      // the button that selects the track included.
      if (
        event.target instanceof Element &&
        event.target.closest(TIMELINE_TRACK_SWITCH_SELECTOR)
      )
        return;
      // Only the press point is recorded here. Capturing the pointer now
      // would redirect the pointerup and the click that follows it to this
      // row, and the header's button would never complete its press - the
      // same mechanism that once swallowed the switch. The capture is taken
      // when the press turns into a reorder, and a press that never moves
      // stays an ordinary press on the button that selects the track.
      dragRef.current = {
        dropIndex: videoTrackOrder.indexOf(source),
        pointerId: event.pointerId,
        source,
        startY: event.clientY,
        started: false,
      };
    };
  const updateDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const active = dragRef.current;
    if (!active) return;
    if (
      !active.started &&
      Math.abs(event.clientY - active.startY) <= DRAG_THRESHOLD_PX
    )
      return;
    if (!active.started) {
      // From here the row owns the pointer: the reorder has started, so the
      // press it grew out of is meant to be cancelled.
      event.currentTarget.setPointerCapture(active.pointerId);
    }
    active.started = true;
    event.preventDefault();
    let dropIndex = videoRows.length;
    for (let index = 0; index < videoRows.length; index += 1) {
      const row = rowElementsRef.current.get(videoRows[index].trackId);
      if (!row) continue;
      const bounds = row.getBoundingClientRect();
      if (event.clientY < bounds.top + bounds.height / 2) {
        dropIndex = index;
        break;
      }
    }
    active.dropIndex = dropIndex;
    setDrag({ dropIndex, source: active.source });
  };
  const finishDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const active = dragRef.current;
    dragRef.current = null;
    if (active?.started) applyOrder(active.source, active.dropIndex);
    setDrag(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId))
      event.currentTarget.releasePointerCapture(event.pointerId);
  };
  const cancelDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    dragRef.current = null;
    setDrag(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId))
      event.currentTarget.releasePointerCapture(event.pointerId);
  };
  return (
    <section
      aria-label="Recording timeline"
      className="shrink-0 pb-control-inset"
      {...timeline.interactionProps}
    >
      <div className="flex items-stretch gap-control">
        <div className="relative flex min-w-0 grow flex-col gap-control">
          <TimelineHeader
            areaRef={timeline.areaRef}
            blade={blade}
            durationMs={durationMs}
            onFit={timeline.fit}
            onSeek={onSeek}
            onZoom={timeline.zoom}
            playhead={playhead}
            viewport={timeline.viewport}
          />
          {videoRows.map(({ pane, trackId }, rowIndex) => {
            const Icon = pane.kind === "camera" ? Camera : Monitor;
            const label = pane.kind === "camera" ? "Camera" : "Screen";
            const enabled = enabledVideoTracks.has(trackId);
            const mustRemainEnabled =
              enabled &&
              enabledVideoTracks.size === 1 &&
              enabledTracks.size === 0;
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
                {/* Where the dragged track would land, drawn in the gap
                    between rows in the accent the app selects with. */}
                {drag?.dropIndex === rowIndex ? (
                  <div className="pointer-events-none absolute -top-tight right-0 left-0 z-20 h-tight rounded-control bg-primary" />
                ) : null}
                {rowIndex === videoRows.length - 1 &&
                drag?.dropIndex === videoRows.length ? (
                  <div className="pointer-events-none absolute -bottom-tight right-0 left-0 z-20 h-tight rounded-control bg-primary" />
                ) : null}
                <TimelineTrackHeader
                  // The press that starts a reorder is taken on the way down:
                  // the button that selects the track stops the bubbling one.
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
                      const next = new Set(enabledVideoTracks);
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
                  viewport={timeline.viewport}
                />
              </div>
            );
          })}
          {audioTracks.length > 0 ? (
            <ScrubAudioTracks
              audioTracks={audioTracks}
              blade={blade}
              enabledTracks={enabledTracks}
              hasEnabledVideo={enabledVideoTracks.size > 0}
              onEnabledTracksChange={onEnabledTracksChange}
              onSelectTrack={(streamIndex) => {
                onSelectedTrackChange(recordingAudioTrackId(streamIndex));
              }}
              selectedTrack={recordingAudioStreamIndex(selectedTrack)}
              viewport={timeline.viewport}
              volumes={volumes}
            />
          ) : null}
          {keyboardItems.length > 0 ? (
            <TimelineItemLane
              edit={blade.edit}
              hiddenFragmentIds={hiddenKeyboardFragmentIds}
              hiddenItemIds={hiddenKeyboardItemIds}
              icon={<Keyboard />}
              items={keyboardItems}
              label="Shortcuts"
              minimumItemWidthPx={48}
              onClearSelection={keyboardSelection.onClear}
              onSelect={(fragment, outputPosition, toggle) => {
                blade.clearRangeSelection();
                blade.selectSegment(null);
                keyboardSelection.onSelect(fragment.fragmentId, toggle);
                onSeek(outputPosition, "end");
              }}
              selectedFragmentIds={keyboardSelection.ids}
              sourceDurationMs={sourceDurationMs}
              viewport={timeline.viewport}
              warningFragmentIds={adjustedKeyboardFragmentIds}
            />
          ) : null}
          <TimelineScrubberOverlay
            blade={blade}
            playhead={playhead}
            viewport={timeline.viewport}
          />
        </div>
        {audioTracks.length > 0 ? (
          <TimelineAudioMeter
            audioTracks={audioTracks}
            enabledTracks={enabledTracks}
            height={meterHeight}
            playhead={playhead}
            volumes={volumes}
          />
        ) : null}
      </div>
    </section>
  );
});
