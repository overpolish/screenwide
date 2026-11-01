// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { WindowShell } from "../../../composites/window-shell/window-shell";
import { t } from "../../../i18n/i18n";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { cameraOutputChoice } from "../export/camera-output";
import { useExportOptionsBridge } from "../export/options-window/use-export-options-bridge";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
} from "../export/recording-export-settings";
import { EditorTitleModeBar } from "../title-bar/editor-mode-bar";
import { EditorTitleBar } from "../title-bar/editor-title-bar";
import { useEditorToolPanels } from "../tool-panels/use-editor-tool-panels";
import { currentEditorKind } from "../window-kind";

import { EditorPanelProps } from "./editor-panel-props";
import { ScreenshotSection } from "./editor-preview-section";
import { RecordingSection } from "./recording-section";
import { useEditorSheets } from "./use-editor-sheets";

/**
 * The editor window: the title bar with its modes, then the recording's or
 * the screenshot's picture with its tool column, islands and, for a
 * recording, the timeline strip. The islands and the export options window
 * are fed the editor's settings from here and hand changes back through
 * these props.
 */
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
  deleteProjectAfterExport = false,
  directory,
  enabledAudioTrackCount,
  enabledStreamIndices,
  enabledVideoTracks = [],
  estimatedSizeBytes,
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
  onDeleteProject,
  onDeleteProjectAfterExportChange,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onFileStemChange,
  onKeyboardEffectsChange,
  onMinimize,
  onRecordingOutputChange,
  onRecordingTimelineEditChange,
  onRenameProject,
  onResolutionScaleChange,
  onSave,
  onScreenshotOutputChange,
  onSelectedScreenshotItemChange,
  onSelectedTrackChange,
  onSubtitlesChange,
  onToggleMaximize,
  onTrackVolumeChange,
  recordingOutput,
  recordingPreviewError,
  recordingPreviewLayout,
  recordingPreviewTracks,
  recordingTimelineEdit,
  resolutionScalePercent,
  screenshotOutput,
  selectedScreenshotItemId = null,
  selectedTrack = null,
  subtitles = "none",
}: EditorPanelProps) {
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
  const { isSheetOpen, openExportOptions } = useEditorSheets(
    workspace,
    Boolean(isSaving),
  );
  // A save or a sheet over the editor puts its tools out of reach, in the title
  // bar and on the islands alike, while leaving them in view.
  const isLocked = Boolean(isSaving) || isSheetOpen;
  const cameraOutput = cameraOutputChoice({
    artifact,
    bakeCamera,
    enabledVideoTracks,
  });
  // The islands work as the options window does: settings the editor owns,
  // shown in a window of their own over the picture and changed from there.
  const { hasTranscript } = useEditorToolPanels({
    artifact,
    audioTrackVolumes,
    bakeCamera,
    cameraOutput,
    cameraOverlay,
    cursorEffects,
    enabledVideoTracks,
    isLocked,
    keyboardEffects,
    onCameraOverlayChange,
    onCanvasResize,
    onCursorEffectsChange,
    onKeyboardEffectsChange,
    onRecordingOutputChange,
    onRecordingTimelineEditChange,
    onScreenshotOutputChange,
    onTrackVolumeChange,
    recordingOutput,
    recordingTimelineEdit,
    screenshotOutput,
    selectedScreenshotItemId,
    selectedTrack,
    trackInclusion: {
      enabledStreamIndices,
      onEnabledTracksChange,
      onEnabledVideoTracksChange,
    },
    workspace,
  });
  useExportOptionsBridge(
    workspace,
    {
      cameraCompression,
      cameraOutput,
      cameraResolutionScalePercent,
      canExport,
      collapseAudio,
      compression,
      deleteProjectAfterExport,
      directory,
      enabledAudioTrackCount: enabledAudioTrackCount ?? 0,
      estimatedSizeBytes: estimatedSizeBytes ?? null,
      extension: isAudioOnly ? "m4a" : (artifact?.extension ?? ""),
      fileStem,
      hasTranscript,
      includeCamera: enabledVideoTracks.includes("camera"),
      isAudioOnly,
      isCancelingSave,
      isEstimatingSize: Boolean(isEstimatingSize),
      isSaving: Boolean(isSaving),
      recordingOutput: recordingOutput ?? null,
      resolutionScalePercent: resolutionScalePercent ?? 100,
      subtitles,
    },
    {
      onBrowse,
      onCameraCompressionChange,
      onCameraOutputChange: (output) => {
        onBakeCameraChange?.(output === "combined");
      },
      onCameraResolutionScaleChange,
      onCancelSave,
      onCollapseAudioChange,
      onCompressionChange,
      onDeleteProjectAfterExportChange,
      onExport: onSave,
      onFileStemChange,
      onResolutionScaleChange,
      onSubtitlesChange,
    },
  );
  return (
    <WindowShell
      className="relative"
      header={
        <EditorTitleBar
          artifact={artifact}
          canExport={canExport}
          isSaving={isSaving}
          modes={<EditorTitleModeBar isDisabled={isSaving || isSheetOpen} />}
          onClose={onCancel}
          onCopy={onCopy}
          onDelete={onDeleteProject}
          onExport={openExportOptions}
          onMinimize={onMinimize}
          onRename={onRenameProject}
          onToggleMaximize={onToggleMaximize}
        />
      }
    >
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 -z-10"
        data-preview-backdrop
        data-preview-window-backdrop
      />
      {/* The export sheet is modal to the content the way a native sheet is:
        nothing below the title bar responds, and nothing dims. The title
        bar stays live so the window can still be moved. The picture runs
        to the window's bottom edge; the recording's strip keeps the
        window's inset from it. */}
      <div className="flex min-h-0 grow flex-col" inert={isSheetOpen}>
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
            isPreparingRecordingAudio={isPreparingRecordingAudio}
            isPreparingRecordingPreview={isPreparingRecordingPreview}
            isSaving={isSaving}
            isSheetOpen={isSheetOpen}
            key={artifact.id}
            keyboardEffects={keyboardEffects}
            onCameraOverlayChange={onCameraOverlayChange}
            onEnabledTracksChange={onEnabledTracksChange}
            onEnabledVideoTracksChange={onEnabledVideoTracksChange}
            onRecordingOutputChange={onRecordingOutputChange}
            onRecordingTimelineEditChange={onRecordingTimelineEditChange}
            onSelectedTrackChange={onSelectedTrackChange}
            recordingOutput={recordingOutput}
            recordingPreviewError={recordingPreviewError}
            recordingPreviewLayout={recordingPreviewLayout}
            recordingPreviewTracks={recordingPreviewTracks}
            recordingTimelineEdit={recordingTimelineEdit}
            resolutionScalePercent={resolutionScalePercent}
            selectedTrack={selectedTrack}
          />
        ) : artifact ? (
          <section className="flex min-h-0 grow flex-col">
            <ScreenshotSection
              artifact={artifact}
              isSaving={isSaving}
              isSheetOpen={isSheetOpen}
              onCanvasResize={onCanvasResize}
              onOutputChange={onScreenshotOutputChange}
              onSelectedItemChange={onSelectedScreenshotItemChange}
              screenshotOutput={screenshotOutput}
              selectedItemId={selectedScreenshotItemId}
            />
          </section>
        ) : (
          // The window is only shown once Rust holds a capture for it, so
          // no capture here means its snapshot has not arrived yet.
          <div className="flex min-h-0 grow items-center justify-center">
            <CircularProgress
              aria-label={t("editor-opening")}
              isIndeterminate
            />
          </div>
        )}
      </div>
    </WindowShell>
  );
}
