// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { CircularProgress } from "../../../components/base/circular-progress/circular-progress";
import {
  RecordingPreviewLayout,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";
import { useCopyRecordingFrame } from "../use-copy-recording-frame";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import { RecordingPlaybackControls } from "./recording-playback-controls";
import { RecordingTrackLanes } from "./recording-track-lanes";
import { ResizableRecordingTimelineArea } from "./resizable-recording-timeline-area";
import { Playhead } from "./scrub-playhead";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";
import { useRecordingPreviewTracks } from "./use-recording-preview-tracks";
import { useRecordingPreviewTransport } from "./use-recording-preview-transport";

import type { ResolvedScrubPreviewProps } from "./recording-preview-props";

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
  isPreparingAudio,
  keyboardEffects,
  keyboardTimeline,
  layout,
  onVideoTrackOrderChange,
  player,
  playhead,
  selectedTrack,
  selectedVideoTracks,
  timelineBlade,
  timelineThumbnails,
  videoTrackOrderList,
  zoomControl,
}: Pick<
  ResolvedScrubPreviewProps,
  | "artifactId"
  | "audioTracks"
  | "durationMs"
  | "isPreparingAudio"
  | "keyboardEffects"
  | "onVideoTrackOrderChange"
  | "selectedTrack"
> &
  Pick<
    ReturnType<typeof useRecordingPreviewTransport>,
    "annotations" | "player" | "timelineBlade" | "timelineThumbnails"
  > &
  Pick<
    ReturnType<typeof useRecordingPreviewTracks>,
    | "audioVolumeByStream"
    | "enabledTracks"
    | "selectedVideoTracks"
    | "videoTrackOrderList"
  > &
  Pick<ReturnType<typeof useRecordingPreviewSelection>, "keyboardTimeline"> & {
    changeCanvasTool: (tool: RecordingCanvasTool) => void;
    changeEnabledTracks: (tracks: Set<number>) => void;
    changeEnabledVideoTracks: (tracks: Set<RecordingVideoTrackId>) => void;
    changeSelectedTrack: (trackId: RecordingTrackId) => void;
    copyCurrentFrame: ReturnType<
      typeof useCopyRecordingFrame
    >["copyCurrentFrame"];
    layout: RecordingPreviewLayout;
    playhead: Playhead;
    zoomControl: ReactNode;
  }) {
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
            aria-label="Preparing audio preview"
            isIndeterminate
            size="small"
          />
          Preparing audio tracks
        </div>
      ) : (
        <RecordingTrackLanes
          adjustedKeyboardFragmentIds={keyboardTimeline.adjustedFragmentIds}
          annotationClips={annotations.clips}
          annotationPinStatus={annotations.pinStatus}
          audioTracks={audioTracks}
          blade={timelineBlade.blade}
          durationMs={timelineBlade.timelineDurationMs}
          enabledTracks={enabledTracks}
          enabledVideoTracks={selectedVideoTracks}
          hiddenKeyboardFragmentIds={keyboardTimeline.hiddenFragmentIds}
          hiddenKeyboardItemIds={keyboardTimeline.hiddenItemIds}
          // Shortcuts turned off leave nothing to place, so the lane goes
          // with them rather than showing items that are not drawn.
          keyboardItems={keyboardEffects.bake ? keyboardTimeline.items : []}
          keyboardSelection={keyboardTimeline.selection}
          layout={layout}
          onAnnotationPinCorrectionsClear={annotations.onClearPinCorrections}
          onAnnotationPinnedChange={annotations.onPinnedChange}
          onAnnotationsChange={annotations.onClipsChange}
          // Choosing an annotation from its lane picks the Select tool up, the
          // way choosing a camera or screen clip does, so the annotation is in
          // hand rather than merely highlighted.
          onAnnotationSelect={(id) => {
            annotations.onSelect(id);
            changeCanvasTool("select");
          }}
          onAnnotationsPreview={annotations.onPreviewClips}
          onEnabledTracksChange={changeEnabledTracks}
          onEnabledVideoTracksChange={changeEnabledVideoTracks}
          onSeek={timelineBlade.seek}
          onSelectedTrackChange={changeSelectedTrack}
          onSelectKeyboardShortcut={() => {
            // Picking a shortcut badge puts it in hand, the way picking an
            // annotation does: clear the annotation and take up the Select tool
            // so the on-screen controls show the shortcut rather than a track.
            annotations.clearSelection();
            changeCanvasTool("select");
          }}
          onVideoTrackOrderChange={onVideoTrackOrderChange}
          playhead={playhead}
          selectedAnnotationId={annotations.selectedId}
          selectedTrack={annotations.hasSelection ? null : selectedTrack}
          sourceDurationMs={durationMs}
          thumbnails={timelineThumbnails}
          videoTrackOrder={videoTrackOrderList}
          volumes={audioVolumeByStream}
        />
      )}
    </ResizableRecordingTimelineArea>
  );
}
