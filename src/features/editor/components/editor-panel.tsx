// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { WindowShell } from "../../../components/shared/window-shell/window-shell";
import { useConfirmSheetModal } from "../../confirm-sheet/use-confirm-sheet-modal";
import { useExportOptionsBridge } from "../export-options/use-export-options-bridge";
import { useExportOptionsOpen } from "../export-options/use-export-options-open";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
} from "../recording-export-settings";
import { useToolPanelBridge } from "../tool-panels/tool-panel-bridge";
import { ToolPanelPlacement } from "../tool-panels/tool-panel-placement";
import { currentEditorKind } from "../window-kind";

import { EditorPanelProps } from "./editor-panel-props";
import { RecordingSection, ScreenshotSection } from "./editor-preview-section";
import { EditorTitlebar } from "./editor-titlebar";
import { PreviewFitProvider } from "./preview-fit";

/**
 * `EditorPanelProps` still carries what the inspectors used to read - the bake
 * camera flag, the keyboard restore handlers, track volume and the per-track
 * output settings. The editor window keeps sending them: the tool panels take
 * those controls over, so the panel itself reads only what it still shows.
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
  directory,
  enabledAudioTrackCount,
  enabledStreamIndices,
  enabledVideoTracks = [],
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
  const isConfirmSheetModal = useConfirmSheetModal();
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
  // The tool panels are the same arrangement one step further out: settings
  // the editor owns, shown in a window of their own and changed from there.
  useToolPanelBridge(
    workspace,
    {
      cursorEffects,
      hasCursorData: isRecording && artifact.hasCursorData,
      isSaving: Boolean(isSaving),
    },
    { onCursorEffectsChange },
  );
  return (
    <WindowShell
      className="relative h-screen w-screen"
      header={
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
          bar stays live so the window can still be moved. */}
      <div
        className="flex min-h-0 grow flex-col gap-section"
        inert={isExportOpen || isConfirmSheetModal}
      >
        {/* The preview hands its fit to the toolbar above it: one per editor
            window, around whichever workspace this one is showing. */}
        <PreviewFitProvider>
          <ToolPanelPlacement />
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
            <section className="flex min-h-0 grow flex-col">
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
        </PreviewFitProvider>
      </div>
    </WindowShell>
  );
}
