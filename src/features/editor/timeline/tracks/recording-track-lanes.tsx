// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard } from "lucide-react";
import { memo, useMemo } from "react";

import { t } from "../../../../i18n/i18n";
import { RecordingAnnotationLane } from "../../recording/annotations/recording-annotation-lane";
import { RecordingSceneLane } from "../../recording/scenes/recording-scene-lane";
import { RecordingSceneClip } from "../../recording/scenes/recording-scenes";
import { recordingAudioStreamIndex, recordingAudioTrackId } from "../../types";
import { ScrubAudioTracks } from "../audio/scrub-audio-tracks";
import { TimelineAudioMeter } from "../audio/timeline-audio-meter";
import {
  RecordingMoment,
  visibleRecordingMoments,
} from "../moments/recording-moments";
import { useMomentNavigation } from "../moments/use-moment-navigation";
import { timelineMeterHeight } from "../timeline-band-metrics";
import { TimelineLanesFrame } from "../timeline-lanes-frame";
import { TimelineScrubberOverlay } from "../timeline-scrubber";
import { TimelineSnapContext } from "../timeline-snap";
import { TimelineHeader } from "../timeline-zoom-toolbar";
import { useTimelineNavigation } from "../use-timeline-navigation";
import { useTimelineSnapValue } from "../use-timeline-snap-value";

import { RecordingTrackLanesProps } from "./recording-track-lanes-contract";
import { RecordingVideoTrackRows } from "./recording-video-track-rows";
import { timedLaneMinimumSpan } from "./timed-lane-layout";
import { TimelineItemLane } from "./timeline-item-lane";

/** The narrowest a shortcut badge is drawn, in pixels; kept here beside the
 * lane that carries it so the lane and the meter agree on one number. */
const KEYBOARD_MINIMUM_ITEM_WIDTH_PX = 48;
const NO_IDS: ReadonlySet<string> = new Set();
const NO_SCENES: RecordingSceneClip[] = [];
const NO_MOMENTS: readonly RecordingMoment[] = [];

/** Memoized because pointer-rate canvas settings do not affect this subtree. */
export const RecordingTrackLanes = memo(function RecordingTrackLanes({
  adjustedKeyboardFragmentIds,
  annotationCameraPlacement,
  annotationClips = [],
  annotationPinStatus,
  annotationPinning,
  audioTracks,
  blade,
  durationMs,
  enabledTracks,
  enabledVideoTracks,
  hiddenKeyboardFragmentIds,
  hiddenKeyboardItemIds,
  isCameraSeparate = false,
  keyboardItems,
  keyboardSelection,
  layout,
  moments = NO_MOMENTS,
  onAnnotationSelect,
  onAnnotationsChange,
  onAnnotationsClear,
  onAnnotationsPreview,
  onAnnotationsSweep,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onSceneActivate,
  onScenePanelOpen,
  onSceneSelect,
  onScenesChange,
  onScenesClear,
  onScenesDraftChange,
  onScenesSweep,
  onSeek,
  onSelectKeyboardShortcut,
  onSelectedTrackChange,
  playhead,
  sceneClips = NO_SCENES,
  scenesPaused = false,
  selectedAnnotationIds = NO_IDS,
  selectedSceneIds = NO_IDS,
  selectedTrack,
  sourceDurationMs,
  thumbnails,
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
  const visibleMoments = useMemo(
    () => visibleRecordingMoments(blade.edit, moments, sourceDurationMs),
    [blade.edit, moments, sourceDurationMs],
  );
  useMomentNavigation({ moments: visibleMoments, onSeek, playhead });
  const snap = useTimelineSnapValue({
    annotationClips,
    blade,
    hiddenKeyboardFragmentIds,
    hiddenKeyboardItemIds,
    keyboardItems,
    moments: visibleMoments,
    playhead,
    sourceDurationMs,
  });
  return (
    <section
      aria-label={t("editor-timeline-label")}
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
              moments={visibleMoments}
              onFit={timeline.fit}
              onSeek={onSeek}
              onZoom={timeline.zoom}
              playhead={playhead}
              viewport={timeline.viewport}
            />
          }
        >
          {onScenesChange &&
          onSceneActivate &&
          onSceneSelect &&
          onScenesClear &&
          onScenesSweep ? (
            <RecordingSceneLane
              clips={sceneClips}
              edit={blade.edit}
              isPaused={scenesPaused}
              onActivate={onSceneActivate}
              onChange={onScenesChange}
              onClearSelection={onScenesClear}
              onDraftChange={onScenesDraftChange}
              onOpenPanel={onScenePanelOpen}
              onSeek={onSeek}
              onSelect={onSceneSelect}
              onSelectSwept={onScenesSweep}
              selectedIds={selectedSceneIds}
              sourceDurationMs={sourceDurationMs}
              viewport={timeline.viewport}
            />
          ) : null}
          {keyboardItems.length > 0 ? (
            <TimelineItemLane
              edit={blade.edit}
              hiddenFragmentIds={hiddenKeyboardFragmentIds}
              hiddenItemIds={hiddenKeyboardItemIds}
              icon={<Keyboard />}
              items={keyboardItems}
              label={t("editor-timeline-shortcuts")}
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
          <RecordingVideoTrackRows
            blade={blade}
            enabledTracks={enabledTracks}
            enabledVideoTracks={enabledVideoTracks}
            isCameraSeparate={isCameraSeparate}
            layout={layout}
            onEnabledVideoTracksChange={onEnabledVideoTracksChange}
            onSelectedTrackChange={onSelectedTrackChange}
            selectedTrack={selectedTrack}
            thumbnails={thumbnails}
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
          {onAnnotationSelect &&
          onAnnotationsChange &&
          onAnnotationsClear &&
          onAnnotationsSweep ? (
            <RecordingAnnotationLane
              cameraPlacement={annotationCameraPlacement}
              clips={annotationClips}
              edit={blade.edit}
              onChange={onAnnotationsChange}
              onClearSelection={onAnnotationsClear}
              onPreview={onAnnotationsPreview}
              onSeek={onSeek}
              onSelect={onAnnotationSelect}
              onSelectSwept={onAnnotationsSweep}
              pinning={annotationPinning}
              pinStatus={annotationPinStatus}
              selectedIds={selectedAnnotationIds}
              showsLayer={layout.panes.some((pane) => pane.kind === "camera")}
              sourceDurationMs={sourceDurationMs}
              viewport={timeline.viewport}
            />
          ) : null}
        </TimelineLanesFrame>
      </TimelineSnapContext>
    </section>
  );
});
