// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Cog, Keyboard, MousePointer2 } from "lucide-react";
import { useState } from "react";

import { Alert } from "../../../components/base/alert/alert";
import { Checkbox } from "../../../components/base/checkbox/checkbox";
import { OverflowShadow } from "../../../components/base/overflow-shadow/overflow-shadow";
import { PillGroup } from "../../../components/base/pill-group/pill-group";
import { Slider } from "../../../components/base/slider/slider";
import { resizeCameraOverlayCentered } from "../camera-overlay-geometry";
import { resolutionScales, scaledDimensions } from "../resolution";
import {
  RecordingOutputSettings,
  resizeScreenshotOutputCentered,
  ScreenshotOutputSettings,
} from "../screenshot-output";
import {
  CameraOverlaySettings,
  ExportArtifact,
  CursorEffectSettings,
  KeyboardEffectSettings,
  recordingAudioStreamIndex,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";

import { CameraTrackSettings } from "./camera-track-settings";
import { CursorEffectControls } from "./cursor-effect-controls";
import { KeyboardEffectControls } from "./keyboard-effect-controls";
import {
  RecordingSizeEstimate,
  VideoExportSettings,
} from "./recording-export-options";
import { recordingTrackTabs } from "./recording-track-tabs";
import { ScreenshotOutputControls } from "./screenshot-inspector";

type RecordingArtifact = Extract<ExportArtifact, { kind: "recording" }>;

const inspectorTabs = [
  { icon: <Cog size={15} />, id: "settings", label: "Settings" },
  { icon: <MousePointer2 size={15} />, id: "cursor", label: "Cursor" },
];

export function ExportInspector({
  artifact,
  bakeCamera,
  cameraCompression,
  cameraOverlay,
  cameraResolutionScalePercent,
  canRestoreKeyboardShortcuts,
  collapseAudio,
  compression,
  cursorEffects,
  enabledAudioTrackCount = 0,
  enabledVideoTracks = [],
  error,
  estimatedSizeBytes,
  isEstimatingSize,
  isSaving,
  keyboardEffects,
  onBakeCameraChange,
  onCameraCompressionChange,
  onCameraOverlayChange,
  onCameraResolutionScaleChange,
  onCollapseAudioChange,
  onCompressionChange,
  onCursorEffectsChange,
  onKeyboardEffectsChange,
  onRecordingOutputChange,
  onResetKeyboardShortcuts,
  onResolutionScaleChange,
  onRestoreKeyboardShortcuts,
  onSelectedTrackChange,
  onSelectedTrackVolumeChange,
  recordingOutput,
  resolutionScalePercent,
  selectedTrack,
  selectedTrackVolume = 0,
}: {
  artifact: RecordingArtifact;
  bakeCamera: boolean;
  cameraCompression: number;
  cameraOverlay: CameraOverlaySettings;
  cameraResolutionScalePercent: number;
  compression: number;
  cursorEffects: CursorEffectSettings;
  keyboardEffects: KeyboardEffectSettings;
  selectedTrack: RecordingTrackId | null;
  canRestoreKeyboardShortcuts?: boolean;
  collapseAudio?: boolean;
  enabledAudioTrackCount?: number;
  enabledVideoTracks?: RecordingVideoTrackId[];
  error?: string | null;
  estimatedSizeBytes?: number | null;
  isEstimatingSize?: boolean;
  isSaving?: boolean;
  onBakeCameraChange?: (bake: boolean) => void;
  onCameraCompressionChange?: (compression: number) => void;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onCameraResolutionScaleChange?: (scale: number) => void;
  onCollapseAudioChange?: (collapse: boolean) => void;
  onCompressionChange?: (compression: number) => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
  onRecordingOutputChange?: (
    trackId: RecordingVideoTrackId,
    settings: ScreenshotOutputSettings,
  ) => void;
  onResetKeyboardShortcuts?: () => void;
  onResolutionScaleChange?: (scale: number) => void;
  onRestoreKeyboardShortcuts?: () => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
  onSelectedTrackVolumeChange?: (decibels: number) => void;
  recordingOutput?: RecordingOutputSettings;
  resolutionScalePercent?: number;
  selectedTrackVolume?: number;
}) {
  const [inspectorTab, setInspectorTab] = useState("settings");
  const availableResolutionScales = resolutionScales(artifact);
  const effectiveResolutionScale =
    resolutionScalePercent ?? availableResolutionScales[0];
  const keyboardOutputDimensions = recordingOutput
    ? recordingOutput.primary
    : scaledDimensions(artifact, effectiveResolutionScale);
  const selectedAudioStreamIndex = recordingAudioStreamIndex(selectedTrack);
  const selectedAudioTrack = artifact.audioTracks.find(
    (track) => track.streamIndex === selectedAudioStreamIndex,
  );
  const videoSelection = new Set(enabledVideoTracks);
  const canBakeCamera =
    videoSelection.has("primary") && videoSelection.has("camera");
  const hasRecordingSettings =
    Boolean(artifact.camera) || artifact.audioTracks.length > 1;
  const tabs = recordingTrackTabs(artifact, recordingOutput);
  const availableInspectorTabs = artifact.hasKeyboardData
    ? [
        ...inspectorTabs,
        { icon: <Keyboard size={15} />, id: "keyboard", label: "Keyboard" },
      ]
    : inspectorTabs;
  const effectiveSelectedTrack = selectedTrack ?? tabs[0].id;
  const resizePrimaryOutput = (width: number, height: number) => {
    if (!recordingOutput) return;
    const next = resizeScreenshotOutputCentered({
      height,
      settings: recordingOutput.primary,
      source: { height: artifact.height, width: artifact.width },
      width,
    });
    if (bakeCamera && canBakeCamera)
      onCameraOverlayChange?.(
        resizeCameraOverlayCentered(
          cameraOverlay,
          recordingOutput.primary,
          next,
        ),
      );
    onRecordingOutputChange?.("primary", next);
  };

  return (
    <aside className="flex min-h-0 min-w-0 flex-col border-r border-muted/15 bg-content/35">
      <OverflowShadow rootClassName="min-h-0 grow">
        <div className="flex flex-col gap-4 p-4">
          <PillGroup
            ariaLabel="Inspector section"
            isDisabled={isSaving}
            items={availableInspectorTabs}
            onSelectionChange={setInspectorTab}
            selected={inspectorTab}
          />

          {inspectorTab === "settings" ? (
            <>
              {artifact.camera ? (
                <Checkbox
                  isDisabled={isSaving || !canBakeCamera}
                  isSelected={bakeCamera && canBakeCamera}
                  onChange={onBakeCameraChange}
                  size="sm"
                >
                  <span className="flex flex-col">
                    <span className="text-xs">Bake camera into recording</span>
                    <span className="text-xs text-muted">
                      Position and crop it directly in the preview.
                    </span>
                  </span>
                </Checkbox>
              ) : null}

              {artifact.audioTracks.length > 1 ? (
                <Checkbox
                  isDisabled={isSaving || enabledAudioTrackCount < 2}
                  isSelected={collapseAudio}
                  onChange={onCollapseAudioChange}
                  size="sm"
                >
                  <span className="flex flex-col">
                    <span className="text-xs">Collapse audio tracks</span>
                    <span className="text-xs text-muted">
                      Mix the selected tracks into one.
                    </span>
                  </span>
                </Checkbox>
              ) : null}

              {!hasRecordingSettings ? (
                <Alert color="neutral" role="status" size="sm">
                  No additional options are available for this recording.
                </Alert>
              ) : null}

              <RecordingSizeEstimate
                estimatedSizeBytes={estimatedSizeBytes}
                isEstimatingSize={isEstimatingSize}
                originalSizeBytes={artifact.originalSizeBytes}
              />
            </>
          ) : null}

          {inspectorTab === "cursor" && artifact.hasCursorData ? (
            <CursorEffectControls
              isSaving={Boolean(isSaving)}
              onChange={onCursorEffectsChange}
              settings={cursorEffects}
            />
          ) : null}

          {inspectorTab === "keyboard" && artifact.hasKeyboardData ? (
            <KeyboardEffectControls
              canRestoreShortcuts={canRestoreKeyboardShortcuts}
              isSaving={Boolean(isSaving)}
              maximumWidthUnits={artifact.keyboardMaximumWidthUnits}
              onChange={onKeyboardEffectsChange}
              onResetShortcuts={onResetKeyboardShortcuts}
              onRestoreShortcuts={onRestoreKeyboardShortcuts}
              outputDimensions={keyboardOutputDimensions}
              settings={keyboardEffects}
            />
          ) : null}

          {tabs.length > 0 ? (
            <div className="flex flex-col gap-3 border-t border-muted/15 pt-4">
              <PillGroup
                ariaLabel="Recording tracks"
                isDisabled={isSaving}
                items={tabs}
                onSelectionChange={(trackId) => {
                  onSelectedTrackChange?.(trackId as RecordingTrackId);
                }}
                selected={effectiveSelectedTrack}
              />

              {effectiveSelectedTrack === "primary" ? (
                <div className="flex flex-col gap-4">
                  <VideoExportSettings
                    compression={compression}
                    isDisabled={!artifact.canCompress || isSaving}
                    onCompressionChange={onCompressionChange}
                    onResolutionScaleChange={onResolutionScaleChange}
                    resolutionDimensions={(scale) =>
                      scaledDimensions(artifact, scale)
                    }
                    resolutionScale={effectiveResolutionScale}
                    resolutionScales={
                      recordingOutput ? [] : availableResolutionScales
                    }
                  />
                  {recordingOutput ? (
                    <ScreenshotOutputControls
                      className=""
                      isSaving={isSaving}
                      onChange={(settings) => {
                        onRecordingOutputChange?.("primary", settings);
                      }}
                      onDimensionsChange={(width, height) => {
                        resizePrimaryOutput(width, height);
                      }}
                      settings={recordingOutput.primary}
                      sourceHeight={artifact.height}
                      sourceWidth={artifact.width}
                    />
                  ) : null}
                </div>
              ) : null}

              {effectiveSelectedTrack === "camera" && artifact.camera ? (
                <CameraTrackSettings
                  artifact={artifact}
                  availableResolutionScales={availableResolutionScales}
                  baked={bakeCamera && canBakeCamera}
                  cameraCompression={cameraCompression}
                  cameraResolutionScalePercent={cameraResolutionScalePercent}
                  compression={compression}
                  effectiveResolutionScale={effectiveResolutionScale}
                  isSaving={isSaving}
                  onCameraCompressionChange={onCameraCompressionChange}
                  onCameraResolutionScaleChange={onCameraResolutionScaleChange}
                  onCompressionChange={onCompressionChange}
                  onRecordingOutputChange={onRecordingOutputChange}
                  onResolutionScaleChange={onResolutionScaleChange}
                  recordingOutput={recordingOutput}
                  resizePrimaryOutput={resizePrimaryOutput}
                />
              ) : null}

              {selectedAudioTrack ? (
                <Slider
                  aria-label={`${selectedAudioTrack.label} volume`}
                  isDisabled={isSaving}
                  label="Volume"
                  maxValue={12}
                  minValue={-60}
                  onChange={onSelectedTrackVolumeChange}
                  renderValue={(value) =>
                    value <= -60
                      ? "Muted"
                      : `${value > 0 ? "+" : ""}${value.toString()} dB`
                  }
                  step={1}
                  value={selectedTrackVolume}
                />
              ) : null}
            </div>
          ) : null}

          {error ? (
            <p className="m-0 text-xs text-error" role="alert">
              {error}
            </p>
          ) : null}
        </div>
      </OverflowShadow>
    </aside>
  );
}
