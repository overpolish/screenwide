// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, RefObject } from "react";

import { CircularProgress } from "../../../components/base/circular-progress/circular-progress";
import { t } from "../../../i18n/i18n";
import { useCameraBackgroundRanges } from "../recording/camera/use-camera-background-ranges";
import { useRecordingCaptions } from "../recording/captions/use-recording-captions";
import {
  CAPTIONS_LAYER_ID,
  KEYBOARD_LAYER_ID,
  OverlayLayerId,
} from "../recording/preview/overlay-layers";
import { RecordingCanvasTool } from "../recording/preview/recording-crop-toggle";
import { useRecordingPreviewSelection } from "../recording/preview/use-recording-preview-selection";
import { useRecordingPreviewTransport } from "../recording/preview/use-recording-preview-transport";
import { useCopyRecordingFrame } from "../recording/use-copy-recording-frame";
import {
  RecordingPreviewLayout,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";

import { usePauseForMomentNotes } from "./moments/moment-note-popover";
import { useRecordingMoments } from "./moments/use-recording-moments";
import { overlayPress } from "./overlay-press";
import { RecordingPlaybackControls } from "./recording-playback-controls";
import { ResizableRecordingTimelineArea } from "./resizable-recording-timeline-area";
import { Playhead } from "./scrub-playhead";
import { RecordingTrackLanes } from "./tracks/recording-track-lanes";
import { useRecordingPreviewTracks } from "./tracks/use-recording-preview-tracks";
import { lettingGoFirst } from "./tracks/use-timed-span-selection";

import type { TimelineItemSelection } from "./tracks/timeline-item-selection";
import type { ResolvedScrubPreviewProps } from "../recording/preview/recording-preview-props";

/** The transport and the lanes under the picture, from the playback controls
 * in the header down to every track the recording carries. */
export function RecordingPreviewTimelineBand({
  annotations,
  artifactId,
  audioTracks,
  audioVolumeByStream,
  captionSelection,
  changeCanvasTool,
  changeEnabledTracks,
  changeEnabledVideoTracks,
  changeSelectedTrack,
  copyCurrentFrame,
  durationMs,
  enabledTracks,
  hasAnnotations,
  isCameraSeparate,
  isPreparingAudio,
  keyboardEffects,
  keyboardTimeline,
  layout,
  onRecordingTimelineEditChange,
  openOverlayTool,
  player,
  playhead,
  pressOverlayRef,
  recordingOutput,
  scenes,
  selectedTrack,
  selectedVideoTracks,
  timelineBlade,
  timelineThumbnails,
  zoomControl,
}: Pick<
  ResolvedScrubPreviewProps,
  | "artifactId"
  | "audioTracks"
  | "durationMs"
  | "hasAnnotations"
  | "isPreparingAudio"
  | "keyboardEffects"
  | "onRecordingTimelineEditChange"
  | "recordingOutput"
  | "selectedTrack"
> &
  Pick<
    ReturnType<typeof useRecordingPreviewTransport>,
    "annotations" | "player" | "scenes" | "timelineBlade" | "timelineThumbnails"
  > &
  Pick<
    ReturnType<typeof useRecordingPreviewTracks>,
    | "audioVolumeByStream"
    | "enabledTracks"
    | "isCameraSeparate"
    | "selectedVideoTracks"
  > &
  Pick<
    ReturnType<typeof useRecordingPreviewSelection>,
    "captionSelection" | "keyboardTimeline"
  > & {
    changeCanvasTool: (tool: RecordingCanvasTool) => void;
    changeEnabledTracks: (tracks: Set<number>) => void;
    changeEnabledVideoTracks: (tracks: Set<RecordingVideoTrackId>) => void;
    changeSelectedTrack: (trackId: RecordingTrackId) => void;
    copyCurrentFrame: ReturnType<
      typeof useCopyRecordingFrame
    >["copyCurrentFrame"];
    layout: RecordingPreviewLayout;
    openOverlayTool: (layerId: OverlayLayerId) => void;
    playhead: Playhead;
    pressOverlayRef: RefObject<(layerId: OverlayLayerId) => void>;
    zoomControl: ReactNode;
  }) {
  const moments = useRecordingMoments(artifactId);
  const captions = useRecordingCaptions({
    artifactId,
    edit: timelineBlade.blade.edit,
    output: recordingOutput?.primary,
  });
  usePauseForMomentNotes(player.pause);
  const cameraBackground = useCameraBackgroundRanges({
    artifactId,
    edit: timelineBlade.blade.edit,
    getPositionMs: player.getPositionMs,
    onEdit: (next) => onRecordingTimelineEditChange?.(next),
    playhead,
    sourceDurationMs: durationMs,
  });
  // The Scene panel follows the scene tool, so the lane takes the tool up and
  // the panel comes with it, the only tool in hand.
  const openScenePanel = () => {
    changeCanvasTool("scene");
  };
  // Choosing scenes lets go of whatever else was chosen, so Delete only ever
  // takes away one kind of thing.
  const clearOtherSelections = () => {
    annotations.clearSelection();
    captionSelection.onClear();
    keyboardTimeline.selection.onClear();
    timelineBlade.blade.clearRangeSelection();
    timelineBlade.blade.selectSegment(null);
  };
  // Shortcuts and captions likewise; each lets go of the other itself.
  const clearForOverlay = () => {
    annotations.clearSelection();
    scenes.selection.onClear();
    cameraBackground.onClearSelection();
    timelineBlade.blade.clearRangeSelection();
    timelineBlade.blade.selectSegment(null);
  };
  // Scenes and the camera's background ranges let go of each other too.
  const sceneChoice = lettingGoFirst(scenes.selection, () => {
    clearOtherSelections();
    cameraBackground.onClearSelection();
  });
  const choosingAlone = (
    selection: TimelineItemSelection<string>,
  ): TimelineItemSelection<string> => ({
    ...selection,
    onSelect: (id, toggle) => {
      clearForOverlay();
      selection.onSelect(id, toggle);
    },
    onSelectSwept: (ids, additive) => {
      if (ids.length > 0) clearForOverlay();
      selection.onSelectSwept(ids, additive);
    },
  });
  // A press on an overlay in the picture reads the lanes as they are now.
  pressOverlayRef.current = overlayPress({
    captionSelection,
    captions,
    edit: timelineBlade.blade.edit,
    keyboard: keyboardTimeline,
    onChoose: clearForOverlay,
    openOverlayTool,
    playhead,
    sourceDurationMs: durationMs,
    timelineDurationMs: timelineBlade.timelineDurationMs,
  });
  return (
    <ResizableRecordingTimelineArea
      artifactId={artifactId}
      header={
        <RecordingPlaybackControls
          durationMs={timelineBlade.timelineDurationMs}
          isPlaying={player.isPlaying}
          onCopyCurrentFrame={copyCurrentFrame}
          onPause={player.pause}
          onPlay={player.play}
          onPlaybackRateChange={player.setPlaybackRate}
          playbackRate={player.playbackRate}
          playhead={playhead}
          zoomControl={zoomControl}
        />
      }
      ready={!isPreparingAudio}
    >
      {isPreparingAudio ? (
        <div className="flex shrink-0 items-center justify-center gap-control-inset py-layout text-body text-content-fg-secondary">
          <CircularProgress
            aria-label={t("editor-timeline-preparing-audio-label")}
            isIndeterminate
            size="small"
          />
          {t("editor-timeline-preparing-audio")}
        </div>
      ) : (
        <RecordingTrackLanes
          annotationCameraPlacement={annotations.cameraPlacement}
          annotationClips={annotations.clips}
          annotationPinning={annotations.pinning}
          annotationPinStatus={annotations.pinStatus}
          audioTracks={audioTracks}
          blade={timelineBlade.blade}
          cameraBackground={{
            ...cameraBackground,
            ...lettingGoFirst(cameraBackground, () => {
              clearOtherSelections();
              scenes.selection.onClear();
            }),
            // A range clicked in its strip takes up Select with the camera
            // chosen, whose panel shows the range.
            onActivate: () => {
              changeCanvasTool("select");
            },
          }}
          captions={captions}
          captionSelection={choosingAlone(captionSelection)}
          durationMs={timelineBlade.timelineDurationMs}
          enabledTracks={enabledTracks}
          enabledVideoTracks={selectedVideoTracks}
          hiddenKeyboardFragmentIds={keyboardTimeline.hiddenFragmentIds}
          hiddenKeyboardItemIds={keyboardTimeline.hiddenItemIds}
          isCameraSeparate={isCameraSeparate}
          // Shortcuts turned off leave nothing to place, so the lane goes
          // with them rather than showing items that are not drawn.
          keyboardItems={keyboardEffects.bake ? keyboardTimeline.items : []}
          keyboardSelection={choosingAlone(keyboardTimeline.selection)}
          layout={layout}
          moments={moments}
          onAnnotationsChange={
            hasAnnotations ? annotations.onClipsChange : undefined
          }
          onAnnotationsClear={annotations.clearSelection}
          // Choosing an annotation from its lane picks the Select tool up, the
          // way choosing a camera or screen clip does, so the annotation is in
          // hand rather than merely highlighted.
          onAnnotationSelect={(id, toggle) => {
            annotations.onSelect(id, toggle);
            changeCanvasTool("select");
          }}
          onAnnotationsPreview={annotations.onPreviewClips}
          onAnnotationsSweep={(ids, additive) => {
            annotations.onSelectSwept(ids, additive);
            if (ids.length > 0) changeCanvasTool("select");
          }}
          onEnabledTracksChange={changeEnabledTracks}
          onEnabledVideoTracksChange={changeEnabledVideoTracks}
          onOpenCaptionsPanel={() => {
            openOverlayTool(CAPTIONS_LAYER_ID);
          }}
          onOpenKeyboardPanel={() => {
            openOverlayTool(KEYBOARD_LAYER_ID);
          }}
          // A scene clicked in its lane takes up the Scene tool, the way a
          // chosen annotation takes up Select: what else was chosen lets go.
          onSceneActivate={() => {
            clearOtherSelections();
            openScenePanel();
          }}
          onScenePanelOpen={openScenePanel}
          onScenesChange={scenes.isAvailable ? scenes.onClipsChange : undefined}
          onScenesClear={scenes.selection.onClear}
          onScenesDraftChange={scenes.onDraftChange}
          onSceneSelect={sceneChoice.onSelect}
          onScenesSweep={sceneChoice.onSelectSwept}
          onSeek={timelineBlade.seek}
          onSelectedTrackChange={changeSelectedTrack}
          playhead={playhead}
          sceneClips={scenes.clips}
          scenesPaused={!scenes.canPlaceCamera}
          selectedAnnotationIds={annotations.selectedIds}
          selectedSceneIds={scenes.selection.ids}
          selectedTrack={annotations.hasSelection ? null : selectedTrack}
          sourceDurationMs={durationMs}
          thumbnails={timelineThumbnails}
          volumes={audioVolumeByStream}
        />
      )}
    </ResizableRecordingTimelineArea>
  );
}
