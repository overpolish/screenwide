// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { CircularProgress } from "../../../components/base/circular-progress/circular-progress";
import { t } from "../../../i18n/i18n";
import { useRecordingCaptions } from "../recording/captions/use-recording-captions";
import { RecordingCanvasTool } from "../recording/preview/recording-crop-toggle";
import { useRecordingPreviewSelection } from "../recording/preview/use-recording-preview-selection";
import { useRecordingPreviewTransport } from "../recording/preview/use-recording-preview-transport";
import { useCopyRecordingFrame } from "../recording/use-copy-recording-frame";
import { useToolPanel } from "../tool-panels/use-tool-panel";
import {
  RecordingPreviewLayout,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";

import { usePauseForMomentNotes } from "./moments/moment-note-popover";
import { useRecordingMoments } from "./moments/use-recording-moments";
import { RecordingPlaybackControls } from "./recording-playback-controls";
import { ResizableRecordingTimelineArea } from "./resizable-recording-timeline-area";
import { Playhead } from "./scrub-playhead";
import { RecordingTrackLanes } from "./tracks/recording-track-lanes";
import { useRecordingPreviewTracks } from "./tracks/use-recording-preview-tracks";

import type { ResolvedScrubPreviewProps } from "../recording/preview/recording-preview-props";

/** The transport and the lanes under the picture, from the playback controls
 * in the header down to every track the recording carries. */
export function RecordingPreviewTimelineBand({
  annotations,
  artifactId,
  audioTracks,
  audioVolumeByStream,
  changeCanvasTool,
  changeEnabledTracks,
  changeEnabledVideoTracks,
  changeSelectedTrack,
  copyCurrentFrame,
  durationMs,
  enabledTracks,
  hasAnnotations,
  hasVisiblePanes,
  isCameraSeparate,
  isPreparingAudio,
  keyboardEffects,
  keyboardTimeline,
  layout,
  player,
  playhead,
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
  Pick<ReturnType<typeof useRecordingPreviewSelection>, "keyboardTimeline"> & {
    changeCanvasTool: (tool: RecordingCanvasTool) => void;
    changeEnabledTracks: (tracks: Set<number>) => void;
    changeEnabledVideoTracks: (tracks: Set<RecordingVideoTrackId>) => void;
    changeSelectedTrack: (trackId: RecordingTrackId) => void;
    copyCurrentFrame: ReturnType<
      typeof useCopyRecordingFrame
    >["copyCurrentFrame"];
    hasVisiblePanes: boolean;
    layout: RecordingPreviewLayout;
    playhead: Playhead;
    zoomControl: ReactNode;
  }) {
  const moments = useRecordingMoments(artifactId);
  const captions = useRecordingCaptions({
    artifactId,
    edit: timelineBlade.blade.edit,
    output: recordingOutput?.primary,
  });
  usePauseForMomentNotes(player.pause);
  // The Scene panel follows the scene tool, so the lane takes the tool up and
  // the panel comes with it, the only tool in hand.
  const openScenePanel = () => {
    changeCanvasTool("scene");
  };
  // A caption's look is set in the Captions panel, so picking one opens it
  // from its toolbar button, as the button itself would. With no picture
  // there is nothing for the window to grow for.
  const { openTool, toggle: toggleToolPanel } = useToolPanel("recording");
  const openCaptionsPanel = () => {
    if (openTool === "captions") return;
    const bounds = document
      .querySelector('[data-editor-tool="captions"]')
      ?.getBoundingClientRect();
    if (bounds)
      void toggleToolPanel(
        "captions",
        bounds,
        hasVisiblePanes ? undefined : false,
      );
  };
  // Choosing scenes lets go of whatever else was chosen, so Delete only ever
  // takes away one kind of thing.
  const clearOtherSelections = () => {
    annotations.clearSelection();
    keyboardTimeline.selection.onClear();
    timelineBlade.blade.clearRangeSelection();
    timelineBlade.blade.selectSegment(null);
  };
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
          adjustedKeyboardFragmentIds={keyboardTimeline.adjustedFragmentIds}
          annotationCameraPlacement={annotations.cameraPlacement}
          annotationClips={annotations.clips}
          annotationPinning={annotations.pinning}
          annotationPinStatus={annotations.pinStatus}
          audioTracks={audioTracks}
          blade={timelineBlade.blade}
          captions={captions}
          durationMs={timelineBlade.timelineDurationMs}
          enabledTracks={enabledTracks}
          enabledVideoTracks={selectedVideoTracks}
          hiddenKeyboardFragmentIds={keyboardTimeline.hiddenFragmentIds}
          hiddenKeyboardItemIds={keyboardTimeline.hiddenItemIds}
          isCameraSeparate={isCameraSeparate}
          // Shortcuts turned off leave nothing to place, so the lane goes
          // with them rather than showing items that are not drawn.
          keyboardItems={keyboardEffects.bake ? keyboardTimeline.items : []}
          keyboardSelection={keyboardTimeline.selection}
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
          onSceneSelect={(id, toggle) => {
            clearOtherSelections();
            scenes.selection.onSelect(id, toggle);
          }}
          onScenesSweep={(ids, additive) => {
            if (ids.length > 0) clearOtherSelections();
            scenes.selection.onSelectSwept(ids, additive);
          }}
          onSeek={timelineBlade.seek}
          onSelectCaption={openCaptionsPanel}
          onSelectedTrackChange={changeSelectedTrack}
          onSelectKeyboardShortcut={() => {
            // Picking a shortcut badge puts it in hand, the way picking an
            // annotation does: clear the annotation and take up the Select tool
            // so the on-screen controls show the shortcut rather than a track.
            annotations.clearSelection();
            changeCanvasTool("select");
          }}
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
