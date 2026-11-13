// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { WindowShell } from "../../../composites/window-shell/window-shell";
import { t } from "../../../i18n/i18n";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { cameraOutputChoice } from "../export/camera-output";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
} from "../export/recording-export-settings";
import { useEditorExportIsland } from "../export/use-editor-export-island";
import { EditorTitleModeBar } from "../title-bar/editor-mode-bar";
import { EditorTitleBar } from "../title-bar/editor-title-bar";
import { useEditorToolPanels } from "../tools/use-editor-tool-panels";
import { currentEditorKind } from "../window-kind";

import { ChromeSetBack } from "./chrome-set-back";
import { EditorPanelProps } from "./editor-panel-props";
import { ScreenshotSection } from "./editor-preview-section";
import { RecordingSection } from "./recording-section";

// The islands, the Export island among them, are fed the editor's settings
// from here and hand changes back through these props.
/** The editor window: title bar, picture, tool column, islands and, for a
 * recording, the timeline. */
export function EditorPanel(props: EditorPanelProps) {
  const {
    artifact,
    audioTrackVolumes = [],
    bakeCamera = false,
    cameraOverlay = defaultCameraOverlay(),
    cameraResolutionScalePercent = 100,
    cursorEffects = DEFAULT_CURSOR_EFFECTS,
    enabledAudioTrackCount,
    enabledStreamIndices,
    enabledVideoTracks = [],
    isPreparingRecordingAudio,
    isPreparingRecordingPreview,
    isPreviewPreparing,
    isSaving,
    keyboardEffects = DEFAULT_KEYBOARD_EFFECTS,
    onCameraOverlayChange,
    onCancel,
    onCanvasResize,
    onCopy,
    onCursorEffectsChange,
    onDeleteProject,
    onEnabledTracksChange,
    onEnabledVideoTracksChange,
    onKeyboardEffectsChange,
    onMinimize,
    onRecordingOutputChange,
    onRecordingTimelineEditChange,
    onRenameProject,
    onScreenshotOutputChange,
    onSelectedScreenshotItemChange,
    onSelectedTrackChange,
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
  } = props;
  const isRecording = artifact?.kind === "recording";
  const enabledVideoTrackCount = enabledVideoTracks.length;
  const hasContent =
    !isRecording || enabledVideoTrackCount + (enabledAudioTrackCount ?? 0) > 0;
  // The title bar's button brings the Export island out; its own Export
  // button is what checks the name and the rest.
  const canOpenExport = Boolean(artifact) && hasContent && !isPreviewPreparing;
  const workspace = currentEditorKind() ?? artifact?.kind ?? "recording";
  const cameraOutput = cameraOutputChoice({
    artifact,
    bakeCamera,
    enabledVideoTracks,
  });
  const exportIsland = useEditorExportIsland(props, {
    cameraOutput,
    canOpen: canOpenExport,
    isAudioOnly: isRecording && enabledVideoTrackCount === 0,
  });
  const { isExportOpen } = exportIsland;
  // The islands work as the Export island does: settings the editor owns,
  // shown in a window of their own over the picture and changed from there.
  const { hasTranscript } = useEditorToolPanels({
    artifact,
    audioTrackVolumes,
    bakeCamera,
    cameraOutput,
    cameraOverlay,
    cursorEffects,
    enabledVideoTracks,
    exportIsland: exportIsland.island,
    isLocked: isExportOpen,
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
  const { setHasTranscript } = exportIsland;
  useEffect(() => {
    setHasTranscript(hasTranscript);
  }, [hasTranscript, setHasTranscript]);
  return (
    <WindowShell
      className="relative"
      header={
        <EditorTitleBar
          artifact={artifact}
          canExport={canOpenExport}
          exportProgress={exportIsland.saveProgress}
          isExportOpen={exportIsland.isOpen}
          isSaving={isSaving}
          modes={
            <ChromeSetBack isSetBack={isExportOpen}>
              <EditorTitleModeBar isDisabled={isExportOpen} />
            </ChromeSetBack>
          }
          onClose={onCancel}
          onCopy={onCopy}
          onDelete={onDeleteProject}
          onExport={exportIsland.toggle}
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
      {/* The island window covers this (`island-window-place.ts`). */}
      <div className="flex min-h-0 grow flex-col" data-editor-content>
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
              isExportOpen={isExportOpen}
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
