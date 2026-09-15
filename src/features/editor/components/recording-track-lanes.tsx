// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard } from "lucide-react";
import { memo } from "react";

import { recordingAudioStreamIndex, recordingAudioTrackId } from "../types";

import { RecordingAnnotationLane } from "./recording-annotation-lane";
import { recordingAnnotationRows } from "./recording-annotation-layout";
import { RecordingTrackLanesProps } from "./recording-track-lanes-contract";
import { RecordingVideoTrackRows } from "./recording-video-track-rows";
import { ScrubAudioTracks } from "./scrub-audio-tracks";
import { TIMED_LANE_ROW_HEIGHT_PX } from "./timed-lane-layout";
import { TimelineAudioMeter } from "./timeline-audio-meter";
import {
  CONTROL_GAP_PX,
  timelineMeterHeight,
} from "./timeline-band-metrics";
import { TimelineItemLane } from "./timeline-item-lane";
import { TimelineLanesFrame } from "./timeline-lanes-frame";
import { TimelineScrubberOverlay } from "./timeline-scrubber";
import { TimelineHeader } from "./timeline-zoom-toolbar";
import { useTimedLaneRows } from "./use-timed-lane-rows";
import { useTimelineNavigation } from "./use-timeline-navigation";

/** Memoized because pointer-rate canvas settings do not affect this subtree. */
export const RecordingTrackLanes = memo(function RecordingTrackLanes({
  adjustedKeyboardFragmentIds,
  annotationClips = [],
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
  onAnnotationSelect,
  onAnnotationsChange,
  onAnnotationsPreview,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onSeek,
  onSelectedTrackChange,
  onVideoTrackOrderChange,
  playhead,
  selectedAnnotationId = null,
  selectedTrack,
  sourceDurationMs,
  thumbnails,
  videoTrackOrder,
  volumes,
}: RecordingTrackLanesProps) {
  const timeline = useTimelineNavigation(blade.edit.artifactId);
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
  const annotationRowCount =
    onAnnotationSelect && onAnnotationsChange
      ? Math.max(
          1,
          recordingAnnotationRows(annotationClips, blade.edit, sourceDurationMs)
            .rowCount,
        )
      : 0;
  const rowCount =
    layout.panes.length +
    audioTracks.length +
    (keyboardRowCount > 0 ? 1 : 0) +
    (annotationRowCount > 0 ? 1 : 0);
  // The lane strip under the ruler: every track row, the gaps between them,
  // and the sublanes the shortcut lane grows by. The meter beside the lanes is
  // sized to it, so it never stretches past the lanes it measures - and never
  // past the strip of rows on screen either, since it no longer scrolls.
  const laneContentHeight =
    rowCount * TIMED_LANE_ROW_HEIGHT_PX +
    Math.max(0, rowCount - 1) * CONTROL_GAP_PX +
    (Math.max(0, keyboardRowCount - 1) + Math.max(0, annotationRowCount - 1)) *
      TIMED_LANE_ROW_HEIGHT_PX;
  return (
    <section
      aria-label="Recording timeline"
      className="flex h-full min-h-0 flex-col"
      {...timeline.interactionProps}
    >
      <TimelineLanesFrame
        meter={
          audioTracks.length > 0
            ? (visibleHeight) => (
                <TimelineAudioMeter
                  audioTracks={audioTracks}
                  enabledTracks={enabledTracks}
                  height={timelineMeterHeight({
                    laneContentHeight,
                    visibleHeight,
                  })}
                  playhead={playhead}
                  volumes={volumes}
                />
              )
            : undefined
        }
        overlays={
          <TimelineScrubberOverlay
            blade={blade}
            playhead={playhead}
            viewport={timeline.viewport}
          />
        }
        ruler={
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
        }
      >
        <RecordingVideoTrackRows
          blade={blade}
          enabledTracks={enabledTracks}
          enabledVideoTracks={enabledVideoTracks}
          layout={layout}
          onEnabledVideoTracksChange={onEnabledVideoTracksChange}
          onSelectedTrackChange={onSelectedTrackChange}
          onVideoTrackOrderChange={onVideoTrackOrderChange}
          selectedTrack={selectedTrack}
          thumbnails={thumbnails}
          videoTrackOrder={videoTrackOrder}
          viewport={timeline.viewport}
        />
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
        {onAnnotationSelect && onAnnotationsChange ? (
          <RecordingAnnotationLane
            clips={annotationClips}
            edit={blade.edit}
            onChange={onAnnotationsChange}
            onPreview={onAnnotationsPreview}
            onSeek={onSeek}
            onSelect={onAnnotationSelect}
            selectedId={selectedAnnotationId}
            sourceDurationMs={sourceDurationMs}
            viewport={timeline.viewport}
          />
        ) : null}
      </TimelineLanesFrame>
    </section>
  );
});
