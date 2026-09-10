// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useExportOptionsBridge } from "../export-options/use-export-options-bridge";
import { useExportOptionsOpen } from "../export-options/use-export-options-open";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
} from "../recording-export-settings";
import {
  resizeScreenshotWorkspaceCentered,
  screenshotWorkspaceItemOutput,
} from "../screenshot-output";
import { currentEditorKind } from "../window-kind";

import { EditorInspector } from "./editor-inspector";
import { EditorPanelProps } from "./editor-panel-props";
import { RecordingSection, ScreenshotSection } from "./editor-preview-section";
import { EditorTitlebar } from "./editor-titlebar";
import { ScreenshotInspector } from "./screenshot-inspector";
import { selectedTrackVolume } from "./selected-track-volume";
import { useRestoreRecordingKeyboardShortcuts } from "./use-restore-recording-keyboard-shortcuts";

export function EditorPanel({
  artifact,
  audioTrackVolumes = [],
  bakeCamera = false,
  cameraCompression = 0,
  cameraOverlay = defaultCameraOverlay(),
  cameraResolutionScalePercent = 100,
  collapseAudio = false,
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
  isPreparingRecordingAudio,
  isPreparingRecordingPreview,
  isPreviewPreparing,
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
}: EditorPanelProps) {
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
  const isAudioOnly = isRecording && enabledVideoTrackCount === 0;
  const hasContent =
    !isRecording || enabledVideoTrackCount + (enabledAudioTrackCount ?? 0) > 0;
  // The titlebar button, its shortcut and the form all gate on this one test.
  const canExport =
    Boolean(artifact) &&
    hasContent &&
    fileStem.trim().length > 0 &&
    !isPreviewPreparing &&
    !isSaving;
  // The options window is a separate webview owning none of this: it is shown
  // what the editor holds and asks the editor to change it, keyed on this
  // window's own workspace rather than the artifact's, so an editor with
  // nothing in it cannot publish over the other workspace's settings.
  const workspace = currentEditorKind() ?? artifact?.kind ?? "recording";
  const { isExportOpen, open: openExportOptions } =
    useExportOptionsOpen(workspace);
  useExportOptionsBridge(
    workspace,
    {
      bakeCamera,
      cameraCompression,
      cameraResolutionScalePercent,
      canExport,
      collapseAudio,
      compression,
      directory,
      enabledAudioTrackCount: enabledAudioTrackCount ?? 0,
      estimatedSizeBytes: estimatedSizeBytes ?? null,
      // Rounded before it crosses: the mirror is a `localStorage` write per
      // change, and progress arrives per encoded frame. A whole percent and a
      // whole second are all the ring and the estimate ever show.
      etaSeconds: etaSeconds === null ? null : Math.round(etaSeconds),
      extension: isAudioOnly ? "m4a" : (artifact?.extension ?? ""),
      fileStem,
      includeCamera: enabledVideoTracks.includes("camera"),
      isAudioOnly,
      isCancelingSave,
      isEstimatingSize: Boolean(isEstimatingSize),
      isSaving: Boolean(isSaving),
      recordingOutput: recordingOutput ?? null,
      resolutionScalePercent: resolutionScalePercent ?? 100,
      savePhase,
      saveProgress: saveProgress === null ? null : Math.round(saveProgress),
    },
    {
      onBrowse,
      onCameraCompressionChange,
      onCameraResolutionScaleChange,
      onCancelSave,
      onCollapseAudioChange,
      onCompressionChange,
      onExport: onSave,
      onFileStemChange,
      onResolutionScaleChange,
    },
  );
  const inspector =
    artifact?.kind === "recording" ? (
      <EditorInspector
        artifact={artifact}
        bakeCamera={bakeCamera}
        cameraOverlay={cameraOverlay}
        canRestoreKeyboardShortcuts={canRestoreKeyboardShortcuts}
        cursorEffects={cursorEffects}
        enabledAudioTrackCount={enabledAudioTrackCount}
        enabledVideoTracks={enabledVideoTracks}
        error={error}
        isSaving={isSaving}
        keyboardEffects={keyboardEffects}
        onBakeCameraChange={onBakeCameraChange}
        onCameraOverlayChange={onCameraOverlayChange}
        onCursorEffectsChange={onCursorEffectsChange}
        onKeyboardEffectsChange={onKeyboardEffectsChange}
        onRecordingOutputChange={onRecordingOutputChange}
        onResetKeyboardShortcuts={resetKeyboardShortcuts}
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
    <main className="window-surface relative flex h-screen w-screen flex-col gap-section overflow-hidden rounded-[10px] text-content-fg">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 -z-10"
        data-preview-backdrop
        data-preview-window-backdrop
      />
      <EditorTitlebar
        artifact={artifact}
        canExport={canExport}
        fileStem={fileStem}
        isSaving={isSaving}
        onClose={onCancel}
        onCopy={onCopy}
        onExport={openExportOptions}
        onFileStemChange={onFileStemChange}
        onMinimize={onMinimize}
        onToggleMaximize={onToggleMaximize}
      />
      {/* The export sheet is modal to the content the way a native sheet is:
          nothing below the title bar responds, and nothing dims. The title
          bar stays live so the window can still be moved. */}
      <div
        className="flex min-h-0 grow flex-col gap-section"
        inert={isExportOpen}
      >
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
            isExportOpen={isExportOpen}
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
              isExportOpen={isExportOpen}
              isSaving={isSaving}
              onBackgroundRadiusChange={onScreenshotBackgroundRadiusChange}
              onBackgroundRadiusChangeEnd={
                onScreenshotBackgroundRadiusChangeEnd
              }
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
      </div>
    </main>
  );
}
