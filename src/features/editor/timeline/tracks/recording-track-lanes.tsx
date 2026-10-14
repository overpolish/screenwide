// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { memo, useEffect, useMemo } from "react";

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

import { RecordingOverlayLanes } from "./recording-overlay-lanes";
import { RecordingTrackLanesProps } from "./recording-track-lanes-contract";
import { RecordingVideoTrackRows } from "./recording-video-track-rows";

import type { Caption } from "../../../../bindings/Caption";

const NO_IDS: ReadonlySet<string> = new Set();
const NO_SCENES: RecordingSceneClip[] = [];
const NO_CAPTIONS: readonly Caption[] = [];
const NO_MOMENTS: readonly RecordingMoment[] = [];

/** Memoized because pointer-rate canvas settings do not affect this subtree. */
export const RecordingTrackLanes = memo(function RecordingTrackLanes({
  annotationCameraPlacement,
  annotationClips = [],
  annotationPinStatus,
  annotationPinning,
  audioTracks,
  blade,
  captionSelection,
  captions = NO_CAPTIONS,
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
  onOpenCaptionsPanel,
  onOpenKeyboardPanel,
  onSceneActivate,
  onScenePanelOpen,
  onSceneSelect,
  onScenesChange,
  onScenesClear,
  onScenesDraftChange,
  onScenesSweep,
  onSeek,
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
  // Words chosen in the Transcript panel may lie outside a zoomed view; their
  // start is brought into it, as the playhead seeking there is.
  const highlightStart = blade.transcriptHighlight?.start;
  const reveal = timeline.reveal;
  useEffect(() => {
    if (highlightStart !== undefined) reveal(highlightStart);
  }, [highlightStart, reveal]);
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
          <RecordingOverlayLanes
            areaWidthPx={timeline.areaWidthPx}
            blade={blade}
            captions={captions}
            captionSelection={captionSelection}
            durationMs={durationMs}
            hiddenKeyboardFragmentIds={hiddenKeyboardFragmentIds}
            hiddenKeyboardItemIds={hiddenKeyboardItemIds}
            keyboardItems={keyboardItems}
            keyboardSelection={keyboardSelection}
            onOpenCaptionsPanel={onOpenCaptionsPanel}
            onOpenKeyboardPanel={onOpenKeyboardPanel}
            onSeek={onSeek}
            playhead={playhead}
            sourceDurationMs={sourceDurationMs}
            viewport={timeline.viewport}
          />
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
