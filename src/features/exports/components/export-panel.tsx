// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../components/base/button/button";
import { CircularProgressBar } from "../../../components/base/circular-progress-bar/circular-progress-bar";
import { Overlay } from "../../../components/base/overlay/overlay";
import { formatEta } from "../duration";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
} from "../recording-export-settings";
import {
  resizeScreenshotWorkspaceCentered,
  screenshotWorkspaceItemOutput,
} from "../screenshot-output";

import { ExportInspector } from "./export-inspector";
import { ExportPanelProps } from "./export-panel-props";
import { RecordingSection, ScreenshotSection } from "./export-preview-section";
import { ExportTitlebar } from "./export-titlebar";
import { ScreenshotInspector } from "./screenshot-inspector";
import { selectedTrackVolume } from "./selected-track-volume";
import { useRestoreRecordingKeyboardShortcuts } from "./use-restore-recording-keyboard-shortcuts";

export function ExportPanel({
  artifact,
  audioTrackVolumes = [],
  bakeCamera = false,
  cameraCompression = 0,
  cameraOverlay = defaultCameraOverlay(),
  cameraResolutionScalePercent = 100,
  collapseAudio,
  compression = 0,
  cursorEffects = DEFAULT_CURSOR_EFFECTS,
  directory,
  enabledAudioTrackCount,
  enabledStreamIndices,
  enabledVideoTracks = [],
  error,
  estimatedSizeBytes,
  etaSeconds = null,
  fileStem,
  isCancelingSave = false,
  isEstimatingSize,
  isExportPreparationPending,
  isPreparingRecordingAudio,
  isPreparingRecordingPreview,
  isSaving,
  keyboardEffects = DEFAULT_KEYBOARD_EFFECTS,
  onBakeCameraChange,
  onBrowse,
  onCameraCompressionChange,
  onCameraOverlayChange,
  onCameraResolutionScaleChange,
  onCancel,
  onCancelSave,
  onCanvasResize,
  onCollapseAudioChange,
  onCompressionChange,
  onCopy,
  onCursorEffectsChange,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onFileStemChange,
  onKeyboardEffectsChange,
  onMinimize,
  onRecordingOutputChange,
  onRecordingTimelineEditChange,
  onResolutionScaleChange,
  onSave,
  onScreenshotBackgroundRadiusChange,
  onScreenshotBackgroundRadiusChangeEnd,
  onScreenshotOutputChange,
  onScreenshotRadiusChangeEnd,
  onSelectedScreenshotItemChange,
  onSelectedTrackChange,
  onSelectedTrackVolumeChange,
  onToggleMaximize,
  onVideoTrackOrderChange,
  recordingOutput,
  recordingPreviewError,
  recordingPreviewLayout,
  recordingPreviewTracks,
  recordingTimelineEdit,
  resolutionScalePercent,
  savePhase = "recording",
  saveProgress = null,
  screenshotOutput,
  selectedScreenshotItemId = null,
  selectedTrack = null,
}: ExportPanelProps) {
  const {
    canRestore: canRestoreKeyboardShortcuts,
    reset: resetKeyboardShortcuts,
    restore: restoreKeyboardShortcuts,
  } = useRestoreRecordingKeyboardShortcuts(
    recordingTimelineEdit,
    onRecordingTimelineEditChange,
  );
  const isRecording = artifact?.kind === "recording";
  const enabledVideoTrackCount = enabledVideoTracks.length;
  const isAudioExport = isRecording && enabledVideoTrackCount === 0;
  const hasContent =
    !isRecording || enabledVideoTrackCount + (enabledAudioTrackCount ?? 0) > 0;
  const inspector =
    artifact?.kind === "recording" ? (
      <ExportInspector
        artifact={artifact}
        bakeCamera={bakeCamera}
        cameraCompression={cameraCompression}
        cameraOverlay={cameraOverlay}
        cameraResolutionScalePercent={cameraResolutionScalePercent}
        canRestoreKeyboardShortcuts={canRestoreKeyboardShortcuts}
        collapseAudio={collapseAudio}
        compression={compression}
        cursorEffects={cursorEffects}
        enabledAudioTrackCount={enabledAudioTrackCount}
        enabledVideoTracks={enabledVideoTracks}
        error={error}
        estimatedSizeBytes={estimatedSizeBytes}
        isEstimatingSize={isEstimatingSize}
        isSaving={isSaving}
        keyboardEffects={keyboardEffects}
        onBakeCameraChange={onBakeCameraChange}
        onCameraCompressionChange={onCameraCompressionChange}
        onCameraOverlayChange={onCameraOverlayChange}
        onCameraResolutionScaleChange={onCameraResolutionScaleChange}
        onCollapseAudioChange={onCollapseAudioChange}
        onCompressionChange={onCompressionChange}
        onCursorEffectsChange={onCursorEffectsChange}
        onKeyboardEffectsChange={onKeyboardEffectsChange}
        onRecordingOutputChange={onRecordingOutputChange}
        onResetKeyboardShortcuts={resetKeyboardShortcuts}
        onResolutionScaleChange={onResolutionScaleChange}
        onRestoreKeyboardShortcuts={restoreKeyboardShortcuts}
        onSelectedTrackChange={onSelectedTrackChange}
        onSelectedTrackVolumeChange={onSelectedTrackVolumeChange}
        recordingOutput={recordingOutput}
        resolutionScalePercent={resolutionScalePercent}
        selectedTrack={selectedTrack}
        selectedTrackVolume={selectedTrackVolume(
          audioTrackVolumes,
          selectedTrack,
        )}
      />
    ) : null;
  return (
    <main className="window-surface relative flex h-screen w-screen flex-col overflow-hidden rounded-[10px] text-content-fg">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 -z-10 bg-content/92"
        data-preview-backdrop
        data-preview-window-backdrop
      />
      <Overlay blur="lg" contained isOpen={isSaving}>
        <div className="flex flex-col items-center gap-3">
          <CircularProgressBar
            aria-label="Save progress"
            isIndeterminate={saveProgress === null}
            renderLabel={(percentage) =>
              percentage === undefined ? null : (
                <span className="absolute inset-0 flex items-center justify-center text-lg font-semibold text-content-fg tabular-nums">
                  {percentage.toFixed(0)}%
                </span>
              )
            }
            size={96}
            strokeWidth={8}
            value={saveProgress ?? undefined}
          />
          <div className="flex flex-col items-center gap-0.5">
            <span className="text-sm text-content-fg">
              {isAudioExport
                ? "Saving audio…"
                : isRecording
                  ? savePhase === "camera"
                    ? "Saving camera…"
                    : savePhase === "finalizing"
                      ? "Finalizing recording…"
                      : "Saving recording…"
                  : "Saving screenshot…"}
            </span>
            {etaSeconds === null ? null : (
              <span className="text-xs text-muted tabular-nums">
                {formatEta(etaSeconds)}
              </span>
            )}
          </div>
          <Button
            isDisabled={isCancelingSave}
            onPress={onCancelSave}
            showFocus={false}
            size="sm"
            variant="soft"
          >
            {isCancelingSave ? "Canceling…" : "Cancel"}
          </Button>
        </div>
      </Overlay>
      <ExportTitlebar
        artifact={artifact}
        directory={directory}
        extension={
          isRecording && enabledVideoTrackCount === 0 ? "m4a" : undefined
        }
        fileStem={fileStem}
        hasExportableContent={hasContent}
        isExportPreparationPending={isExportPreparationPending}
        isSaving={isSaving}
        onBrowse={onBrowse}
        onClose={onCancel}
        onCopy={onCopy}
        onExport={onSave}
        onFileStemChange={onFileStemChange}
        onMinimize={onMinimize}
        onToggleMaximize={onToggleMaximize}
      />

      {artifact?.kind === "recording" ? (
        <RecordingSection
          artifact={artifact}
          audioTrackVolumes={audioTrackVolumes}
          bakeCamera={bakeCamera}
          cameraOverlay={cameraOverlay}
          cameraResolutionScalePercent={cameraResolutionScalePercent}
          cursorEffects={cursorEffects}
          enabledStreamIndices={enabledStreamIndices}
          enabledVideoTracks={enabledVideoTracks}
          hasCursorData={artifact.hasCursorData}
          hasKeyboardData={artifact.hasKeyboardData}
          inspector={inspector}
          isPreparingRecordingAudio={isPreparingRecordingAudio}
          isPreparingRecordingPreview={isPreparingRecordingPreview}
          isSaving={isSaving}
          key={artifact.id}
          keyboardEffects={keyboardEffects}
          onCameraOverlayChange={onCameraOverlayChange}
          onEnabledTracksChange={onEnabledTracksChange}
          onEnabledVideoTracksChange={onEnabledVideoTracksChange}
          onKeyboardEffectsChange={onKeyboardEffectsChange}
          onRecordingOutputChange={onRecordingOutputChange}
          onRecordingTimelineEditChange={onRecordingTimelineEditChange}
          onSelectedTrackChange={onSelectedTrackChange}
          onVideoTrackOrderChange={onVideoTrackOrderChange}
          recordingOutput={recordingOutput}
          recordingPreviewError={recordingPreviewError}
          recordingPreviewLayout={recordingPreviewLayout}
          recordingPreviewTracks={recordingPreviewTracks}
          recordingTimelineEdit={recordingTimelineEdit}
          resolutionScalePercent={resolutionScalePercent}
          selectedTrack={selectedTrack}
        />
      ) : artifact ? (
        <section className="grid min-h-0 grow grid-cols-[clamp(350px,28vw,400px)_minmax(0,1fr)]">
          {screenshotOutput ? (
            <ScreenshotInspector
              isSaving={isSaving}
              onChange={onScreenshotOutputChange}
              onDimensionsChange={(width, height) => {
                onCanvasResize?.(
                  resizeScreenshotWorkspaceCentered({
                    height,
                    settings: screenshotOutput,
                    sources: artifact.items,
                    width,
                  }),
                );
              }}
              settings={screenshotWorkspaceItemOutput(
                screenshotOutput,
                selectedScreenshotItemId ?? -1,
              )}
              sourceHeight={artifact.height}
              sourceWidth={artifact.width}
            />
          ) : null}
          <ScreenshotSection
            artifact={artifact}
            isSaving={isSaving}
            onBackgroundRadiusChange={onScreenshotBackgroundRadiusChange}
            onBackgroundRadiusChangeEnd={onScreenshotBackgroundRadiusChangeEnd}
            onCanvasResize={onCanvasResize}
            onOutputChange={onScreenshotOutputChange}
            onRadiusChangeEnd={onScreenshotRadiusChangeEnd}
            onSelectedItemChange={onSelectedScreenshotItemChange}
            screenshotOutput={screenshotOutput}
            selectedItemId={selectedScreenshotItemId}
          />
        </section>
      ) : (
        <div className="flex min-h-0 grow items-center justify-center text-sm text-muted">
          Nothing to export
        </div>
      )}
    </main>
  );
}
