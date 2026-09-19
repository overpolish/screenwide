// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard } from "lucide-react";
import { memo } from "react";

import { recordingAudioStreamIndex, recordingAudioTrackId } from "../types";

import { RecordingAnnotationLane } from "./recording-annotation-lane";
import { RecordingTrackLanesProps } from "./recording-track-lanes-contract";
import { RecordingVideoTrackRows } from "./recording-video-track-rows";
import { ScrubAudioTracks } from "./scrub-audio-tracks";
import { timedLaneMinimumSpan } from "./timed-lane-layout";
import { TimelineAudioMeter } from "./timeline-audio-meter";
import { timelineMeterHeight } from "./timeline-band-metrics";
import { TimelineItemLane } from "./timeline-item-lane";
import { TimelineLanesFrame } from "./timeline-lanes-frame";
import { TimelineScrubberOverlay } from "./timeline-scrubber";
import { TimelineSnapContext } from "./timeline-snap";
import { TimelineHeader } from "./timeline-zoom-toolbar";
import { useTimelineNavigation } from "./use-timeline-navigation";
import { useTimelineSnapValue } from "./use-timeline-snap-value";

/** The narrowest a shortcut badge is drawn, in pixels; kept here beside the
 * lane that carries it so the lane and the meter agree on one number. */
const KEYBOARD_MINIMUM_ITEM_WIDTH_PX = 48;

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
  onSelectKeyboardShortcut,
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
  // Overlapping shortcut badges stack into sublanes, so the lane is told the
  // span a badge needs; derived from the same shared stacking it renders from.
  const keyboardMinimumSpan = timedLaneMinimumSpan({
    contentWidthPx: timeline.areaWidthPx,
    minimumItemWidthPx: KEYBOARD_MINIMUM_ITEM_WIDTH_PX,
    zoom: timeline.viewport.zoom,
  });
  const snap = useTimelineSnapValue({
    annotationClips,
    blade,
    hiddenKeyboardFragmentIds,
    hiddenKeyboardItemIds,
    keyboardItems,
    playhead,
    sourceDurationMs,
  });
  return (
    <section
      aria-label="Recording timeline"
      className="flex h-full min-h-0 flex-col"
      {...timeline.interactionProps}
    >
      <TimelineSnapContext value={snap}>
        <TimelineLanesFrame
          meter={
            audioTracks.length > 0
              ? (visibleHeight) => (
                  <TimelineAudioMeter
                    audioTracks={audioTracks}
                    enabledTracks={enabledTracks}
                    height={timelineMeterHeight(visibleHeight)}
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
              minimumItemWidthPx={KEYBOARD_MINIMUM_ITEM_WIDTH_PX}
              minimumSpan={keyboardMinimumSpan}
              onClearSelection={keyboardSelection.onClear}
              onSelect={(fragment, outputPosition, toggle) => {
                blade.clearRangeSelection();
                blade.selectSegment(null);
                keyboardSelection.onSelect(fragment.fragmentId, toggle);
                onSelectKeyboardShortcut?.();
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
      </TimelineSnapContext>
    </section>
  );
});
