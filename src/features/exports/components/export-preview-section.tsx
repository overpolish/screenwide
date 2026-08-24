// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleDotDashed, Crop, MousePointer2, ScanSquare } from "lucide-react";
import { ReactNode, useRef, useState } from "react";

import {
  scaledDimensions,
  scaledVideoDimensions,
  sourceScalePercent,
} from "../resolution";
import { resetCommittedScreenshotCrop } from "../screenshot-crop";
import {
  RecordingOutputSettings,
  resetScreenshotTransform,
  resizeScreenshotWorkspaceCentered,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
  screenshotOutputDimensions,
  screenshotWorkspaceItemOutput,
} from "../screenshot-output";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  ExportArtifact,
  PreparedAudioTrack,
  RecordingPreviewLayout,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";
import { useExportWindowShortcuts } from "../use-export-window-shortcuts";

import { PreviewToolbar } from "./preview-toolbar";
import { maximumZoom, MINIMUM_ZOOM_CEILING } from "./preview-transform";
import { PreviewViewport } from "./preview-viewport";
import {
  deleteScreenshotLayer,
  moveScreenshotLayer,
} from "./screenshot-layer-actions";
import { ScreenshotToolToggle } from "./screenshot-tool-toggle";
import { ScrubPreview } from "./scrub-preview";
import { useScreenshotRecenter } from "./use-screenshot-recenter";

/**
 * The screenshot section. Sibling to `RecordingSection`, and the reason the
 * frame around them does not know what it is showing.
 */
export function ScreenshotSection({
  artifact,
  isSaving = false,
  onBackgroundRadiusChange,
  onBackgroundRadiusChangeEnd,
  onCanvasResize,
  onOutputChange,
  onRadiusChangeEnd,
  onSelectedItemChange,
  screenshotOutput,
  selectedItemId = null,
}: {
  artifact: Extract<ExportArtifact, { kind: "screenshot" }>;
  isSaving?: boolean;
  onBackgroundRadiusChange?: (radiusPercent: number) => void;
  onBackgroundRadiusChangeEnd?: () => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onOutputChange?: (
    settings: ScreenshotOutputSettings,
    itemId?: number,
  ) => void;
  onRadiusChangeEnd?: () => void;
  onSelectedItemChange?: (itemId: number | null) => void;
  screenshotOutput?: ScreenshotWorkspaceOutputSettings;
  selectedItemId?: number | null;
}) {
  const [zoomPercent, setZoomPercent] = useState(100);
  const [maximumZoomPercent, setMaximumZoomPercent] = useState(
    MINIMUM_ZOOM_CEILING * 100,
  );
  const [tool, setTool] = useState<
    "canvas" | "crop" | "recenter" | "select" | null
  >("select");
  const newestItemId = artifact.items[artifact.items.length - 1]?.id ?? null;
  const moveSelectedLayer = (
    direction: "backward" | "forward",
    itemId = selectedItemId,
  ) => {
    if (!screenshotOutput || itemId === null) return;
    const next = moveScreenshotLayer({
      direction,
      itemId,
      settings: screenshotOutput,
    });
    if (next !== screenshotOutput) onCanvasResize?.(next);
  };
  const deleteSelectedLayer = (itemId = selectedItemId) => {
    if (
      !screenshotOutput ||
      itemId === null ||
      screenshotOutput.items.length <= 1
    )
      return;
    const result = deleteScreenshotLayer({
      itemId,
      settings: screenshotOutput,
    });
    if (!result) return;
    onCanvasResize?.(result.settings);
    onSelectedItemChange?.(result.nextSelectedItemId);
  };
  const outputDimensions = screenshotOutput
    ? screenshotOutputDimensions(screenshotOutput)
    : { height: artifact.height, width: artifact.width };
  const selectedItem = artifact.items.find(
    (item) => item.id === selectedItemId,
  );
  const selectedOutput =
    screenshotOutput && selectedItem
      ? screenshotWorkspaceItemOutput(screenshotOutput, selectedItem.id)
      : null;
  const recenter = useScreenshotRecenter({
    artifactId: artifact.id,
    onOutputChange,
    selectedItem,
    selectedOutput,
  });
  const setRecenterSelected = (selected: boolean) => {
    setTool(selected ? "recenter" : null);
    if (selected) recenter.prepare();
  };
  // A keyboard nudge writes exactly the fields a select-tool drag writes - the
  // `move` branch of `selectionGesture` in preview-viewport.tsx. It is not
  // wrapped in an edit gesture on purpose: the history hook already groups
  // same-key edits that land within its grouping delay, so a held arrow folds
  // into one undo step while a pause starts the next. No snapping: the arrows
  // are the way to place a layer precisely.
  const nudgeRef = useRef<{
    after: ScreenshotOutputSettings;
    beforeX: number;
    beforeY: number;
    itemId: number;
  } | null>(null);
  const nudgeSelectedLayer = (
    directionX: number,
    directionY: number,
    coarse: boolean,
  ) => {
    if (!selectedItem || !selectedOutput) return;
    const pixels = coarse ? 10 : 1;
    const deltaX = (directionX * pixels * 100) / outputDimensions.width;
    const deltaY = (directionY * pixels * 100) / outputDimensions.height;
    // Key repeat can outrun React. While the committed settings have not come
    // back yet the props still hold the previous press's starting point, so
    // continue from what that press sent instead of repeating it.
    const pending = nudgeRef.current;
    const base =
      pending &&
      pending.itemId === selectedItem.id &&
      pending.beforeX === selectedOutput.screenshotCropXPercent &&
      pending.beforeY === selectedOutput.screenshotCropYPercent
        ? pending.after
        : selectedOutput;
    const next = {
      ...base,
      screenshotCropXPercent: base.screenshotCropXPercent + deltaX,
      screenshotCropYPercent: base.screenshotCropYPercent + deltaY,
      screenshotImageXPercent: base.screenshotImageXPercent + deltaX,
      screenshotImageYPercent: base.screenshotImageYPercent + deltaY,
    };
    nudgeRef.current = {
      after: next,
      beforeX: selectedOutput.screenshotCropXPercent,
      beforeY: selectedOutput.screenshotCropYPercent,
      itemId: selectedItem.id,
    };
    onOutputChange?.(next, selectedItem.id);
  };
  useExportWindowShortcuts({
    onDelete: deleteSelectedLayer,
    onMoveBackward: () => {
      moveSelectedLayer("backward");
    },
    onMoveForward: () => {
      moveSelectedLayer("forward");
    },
    onNudge:
      tool === "select" && selectedItem && selectedOutput
        ? nudgeSelectedLayer
        : undefined,
    onRecenter: () => {
      setRecenterSelected(tool !== "recenter");
    },
    onResizeCanvas: () => {
      setTool((current) => (current === "canvas" ? null : "canvas"));
    },
    onSelectTool: () => {
      setTool((current) => (current === "select" ? null : "select"));
    },
    onToggleCrop: () => {
      if (selectedItemId === null) onSelectedItemChange?.(newestItemId);
      setTool((current) => (current === "crop" ? null : "crop"));
    },
  });

  return (
    <div className="flex min-h-0 min-w-0 grow flex-col">
      <PreviewToolbar
        maximumZoomPercent={maximumZoomPercent}
        onZoomChange={setZoomPercent}
        tools={
          <div className="flex items-center gap-1">
            <ScreenshotToolToggle
              isSelected={tool === "select"}
              label="Select"
              name="Select screenshot"
              onReset={() => {
                if (!selectedItem || !selectedOutput) return;
                onOutputChange?.(
                  resetScreenshotTransform(selectedOutput, selectedItem),
                  selectedItem.id,
                );
              }}
              onSelectedChange={(selected) => {
                setTool(selected ? "select" : null);
              }}
              shortcut="V"
            >
              <MousePointer2 size={15} />
            </ScreenshotToolToggle>
            <ScreenshotToolToggle
              isSelected={tool === "canvas"}
              label="Resize canvas"
              name="Resize canvas"
              onReset={() => {
                if (!screenshotOutput) return;
                onCanvasResize?.(
                  resizeScreenshotWorkspaceCentered({
                    height: artifact.height,
                    settings: screenshotOutput,
                    sources: artifact.items,
                    width: artifact.width,
                  }),
                );
              }}
              onSelectedChange={(selected) => {
                setTool(selected ? "canvas" : null);
              }}
              shortcut="F"
            >
              <ScanSquare size={15} />
            </ScreenshotToolToggle>
            <ScreenshotToolToggle
              isSelected={tool === "crop"}
              label="Crop"
              name="Crop screenshot"
              onReset={() => {
                if (!selectedItem || !selectedOutput) return;
                onOutputChange?.(
                  resetCommittedScreenshotCrop(selectedOutput, selectedItem),
                  selectedItem.id,
                );
              }}
              onSelectedChange={(selected) => {
                if (selectedItemId === null)
                  onSelectedItemChange?.(newestItemId);
                setTool(selected ? "crop" : null);
              }}
              shortcut="C"
            >
              <Crop size={15} />
            </ScreenshotToolToggle>
            <ScreenshotToolToggle
              isSelected={tool === "recenter"}
              label="Recenter"
              name="Recenter screenshot"
              onReset={recenter.reset}
              onSelectedChange={setRecenterSelected}
              shortcut="R"
            >
              <CircleDotDashed size={15} />
            </ScreenshotToolToggle>
          </div>
        }
        zoomPercent={zoomPercent}
      />
      <PreviewViewport
        alt="Screenshot preview"
        artifactId={artifact.id}
        isEditing={tool === "crop"}
        isRecentering={tool === "recenter"}
        isResizingCanvas={tool === "canvas"}
        isSaving={isSaving}
        isSelecting={tool === "select"}
        items={artifact.items}
        naturalHeight={artifact.height}
        naturalWidth={artifact.width}
        onBackgroundRadiusChange={onBackgroundRadiusChange}
        onBackgroundRadiusChangeEnd={onBackgroundRadiusChangeEnd}
        onCanvasResize={onCanvasResize}
        onCropChangeEnd={recenter.refresh}
        onItemSelect={onSelectedItemChange}
        onOutputChange={onOutputChange}
        onPaneFitChange={(fit) => {
          setMaximumZoomPercent(Math.round(maximumZoom(fit) * 100));
        }}
        onRadiusChangeEnd={onRadiusChangeEnd}
        onRecenter={recenter.begin}
        onZoomChange={setZoomPercent}
        screenshotOutput={screenshotOutput}
        selectedItemId={selectedItemId}
        zoomPercent={zoomPercent}
      />
    </div>
  );
}

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
  inspector,
  isPreparingRecordingAudio,
  isPreparingRecordingPreview,
  isSaving,
  onCameraOverlayChange,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onRecordingOutputChange,
  onSelectedTrackChange,
  onVideoTrackOrderChange,
  recordingOutput,
  recordingPreviewError,
  recordingPreviewLayout,
  recordingPreviewTracks,
  resolutionScalePercent,
  selectedTrack,
}: {
  artifact: Extract<ExportArtifact, { kind: "recording" }>;
  audioTrackVolumes?: AudioTrackVolume[];
  bakeCamera?: boolean;
  cameraOverlay?: CameraOverlaySettings;
  cameraResolutionScalePercent?: number;
  cursorEffects?: CursorEffectSettings;
  enabledStreamIndices?: number[];
  enabledVideoTracks?: RecordingVideoTrackId[];
  hasCursorData?: boolean;
  inspector?: ReactNode;
  isPreparingRecordingAudio?: boolean;
  isPreparingRecordingPreview?: boolean;
  isSaving?: boolean;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onEnabledTracksChange?: (streamIndices: number[]) => void;
  onEnabledVideoTracksChange?: (tracks: RecordingVideoTrackId[]) => void;
  onRecordingOutputChange?: (
    trackId: RecordingVideoTrackId,
    settings: RecordingOutputSettings[RecordingVideoTrackId],
  ) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
  onVideoTrackOrderChange?: (tracks: RecordingVideoTrackId[]) => void;
  recordingOutput?: RecordingOutputSettings;
  recordingPreviewError?: string | null;
  recordingPreviewLayout?: RecordingPreviewLayout;
  recordingPreviewTracks?: PreparedAudioTrack[];
  resolutionScalePercent?: number;
  selectedTrack?: RecordingTrackId | null;
}) {
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
        inspector={inspector}
        isPreparingAudio={isPreparingRecordingAudio}
        isPreparingPreview={isPreparingRecordingPreview}
        isSaving={isSaving}
        key={artifact.id}
        onCameraOverlayChange={onCameraOverlayChange}
        onEnabledTracksChange={onEnabledTracksChange}
        onEnabledVideoTracksChange={onEnabledVideoTracksChange}
        onRecordingOutputChange={onRecordingOutputChange}
        onSelectedTrackChange={onSelectedTrackChange}
        onVideoTrackOrderChange={onVideoTrackOrderChange}
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
        selectedTrack={selectedTrack}
      />
    </div>
  );
}
