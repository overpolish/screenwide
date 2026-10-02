// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScrubPreview } from "../recording/preview/scrub-preview";
import {
  scaledDimensions,
  scaledVideoDimensions,
  sourceScalePercent,
} from "../resolution";

import { RecordingSectionProps } from "./editor-preview-section-props";

/**
 * The recording section: a preview you skim, with what the file is underneath.
 *
 * Framed exactly like the still beside it - no box, no border, just the
 * picture and its shadow - because they are the same kind of thing to the
 * person deciding whether to keep it.
 */
export function RecordingSection({
  artifact,
  audioTrackVolumes,
  bakeCamera,
  cameraOverlay,
  cameraResolutionScalePercent,
  cursorEffects,
  enabledStreamIndices,
  enabledVideoTracks,
  hasCursorData,
  hasKeyboardData,
  isPreparingRecordingAudio,
  isPreparingRecordingPreview,
  isSaving,
  isSheetOpen,
  keyboardEffects,
  onCameraOverlayChange,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onKeyboardEffectsChange,
  onRecordingOutputChange,
  onRecordingTimelineEditChange,
  onSelectedTrackChange,
  recordingOutput,
  recordingPreviewError,
  recordingPreviewLayout,
  recordingPreviewTracks,
  recordingTimelineEdit,
  resolutionScalePercent,
  selectedTrack,
}: RecordingSectionProps) {
  const primaryOutputDimensions = recordingOutput
    ? {
        height: recordingOutput.primary.height,
        width: recordingOutput.primary.width,
      }
    : scaledDimensions(
        artifact,
        resolutionScalePercent ?? sourceScalePercent(artifact),
      );
  const cameraOutputDimensions = recordingOutput
    ? {
        height: recordingOutput.camera.height,
        width: recordingOutput.camera.width,
      }
    : artifact.camera
      ? scaledVideoDimensions({
          height: artifact.camera.height,
          scale: cameraResolutionScalePercent ?? 100,
          sourceScale: 100,
          width: artifact.camera.width,
        })
      : undefined;

  return (
    <div className="flex min-h-0 grow flex-col">
      <ScrubPreview
        artifactId={artifact.id}
        audioError={recordingPreviewError}
        audioTracks={recordingPreviewTracks}
        audioTrackVolumes={audioTrackVolumes}
        bakeCamera={bakeCamera}
        cameraOverlay={cameraOverlay}
        cursorEffects={cursorEffects}
        durationMs={artifact.durationMs}
        enabledStreamIndices={enabledStreamIndices}
        enabledVideoTracks={enabledVideoTracks}
        hasCursorData={hasCursorData}
        hasKeyboardData={hasKeyboardData}
        isPreparingAudio={isPreparingRecordingAudio}
        isPreparingPreview={isPreparingRecordingPreview}
        isSaving={isSaving}
        isSheetOpen={isSheetOpen}
        key={artifact.id}
        keyboardEffects={keyboardEffects}
        keyboardMaximumWidthUnits={artifact.keyboardMaximumWidthUnits}
        onCameraOverlayChange={onCameraOverlayChange}
        onEnabledTracksChange={onEnabledTracksChange}
        onEnabledVideoTracksChange={onEnabledVideoTracksChange}
        onKeyboardEffectsChange={onKeyboardEffectsChange}
        onRecordingOutputChange={onRecordingOutputChange}
        onRecordingTimelineEditChange={onRecordingTimelineEditChange}
        onSelectedTrackChange={onSelectedTrackChange}
        previewLayout={recordingPreviewLayout}
        previewOutputDimensions={{
          primary: primaryOutputDimensions,
          ...(cameraOutputDimensions ? { camera: cameraOutputDimensions } : {}),
        }}
        previewSourceDimensions={{
          primary: { height: artifact.height, width: artifact.width },
          ...(artifact.camera
            ? {
                camera: {
                  height: artifact.camera.height,
                  width: artifact.camera.width,
                },
              }
            : {}),
        }}
        recordingOutput={recordingOutput}
        recordingTimelineEdit={recordingTimelineEdit}
        selectedTrack={selectedTrack}
      />
    </div>
  );
}
