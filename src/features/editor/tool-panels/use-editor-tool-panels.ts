// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BackgroundPreset } from "../../../composites/background-picker/background";
import { useEditableGeneralSettings } from "../../settings/use-general-settings";
import {
  useAnnotationSelection,
  useAnnotationSelectionCount,
  useCanClearDrawings,
} from "../annotations/annotation-channel";
import { CameraOutput, composedVideoTracks } from "../export/camera-output";
import { useCaptionCanvasEdits } from "../recording/captions/use-caption-canvas-edits";
import { useCaptionTools } from "../recording/captions/use-caption-tools";
import { useMicrophoneTools } from "../recording/microphone/use-microphone-tools";
import { useRecordingSceneSelection } from "../recording/scenes/recording-scene-channel";
import { useTranscriptTools } from "../recording/transcript/use-transcript-tools";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import { useRestoreRecordingKeyboardShortcuts } from "../timeline/keyboard/use-restore-recording-keyboard-shortcuts";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  EditorArtifact,
  EditorKind,
  KeyboardEffectSettings,
  RecordingVideoTrackId,
} from "../types";

import { editorCropTarget } from "./crop/crop-target";
import { editorToolPanelHandlers } from "./editor-tool-panel-handlers";
import { editorFrameTarget } from "./frame/frame-target";
import { editorAudioSelectionTarget } from "./selection/selection-target";
import { useLayerSelectionTarget } from "./selection/use-layer-selection-target";
import { useToolPanelBridge } from "./tool-panel-bridge";
import { DEFAULT_TOOL_PANEL_SNAPSHOT } from "./tool-panel-store";

type EditorToolPanelInputs = {
  artifact: EditorArtifact | null;
  audioTrackVolumes: AudioTrackVolume[];
  bakeCamera: boolean;
  cameraOutput: CameraOutput | null;
  cameraOverlay: CameraOverlaySettings;
  cursorEffects: CursorEffectSettings;
  enabledVideoTracks: RecordingVideoTrackId[];
  isLocked: boolean;
  keyboardEffects: KeyboardEffectSettings;
  recordingOutput: RecordingOutputSettings | null | undefined;
  recordingTimelineEdit: RecordingTimelineEdit | null | undefined;
  screenshotOutput: ScreenshotWorkspaceOutputSettings | null | undefined;
  selectedScreenshotItemId: number | null;
  selectedTrack: string | null;
  workspace: EditorKind;
  onBakeCameraChange?: (bake: boolean) => void;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
  onRecordingOutputChange?: (
    track: RecordingVideoTrackId,
    next: ScreenshotOutputSettings,
  ) => void;
  onRecordingTimelineEditChange?: (edit: RecordingTimelineEdit) => void;
  onScreenshotOutputChange?: (
    next: ScreenshotOutputSettings,
    itemId: number,
  ) => void;
  onSelectedTrackVolumeChange?: (decibels: number) => void;
};

/**
 * What the editor shows the tool panels, and what it lets them change.
 *
 * Each panel acts on one thing the editor already owns - the selected layer,
 * the output canvas - through the very handler the preview's own gesture uses,
 * so a number typed in a panel and a drag on the picture are the same edit.
 */
export function useEditorToolPanels({
  artifact,
  audioTrackVolumes,
  bakeCamera,
  cameraOutput,
  cameraOverlay,
  cursorEffects,
  enabledVideoTracks,
  isLocked,
  keyboardEffects,
  onBakeCameraChange,
  onCameraOverlayChange,
  onCanvasResize,
  onCursorEffectsChange,
  onKeyboardEffectsChange,
  onRecordingOutputChange,
  onRecordingTimelineEditChange,
  onScreenshotOutputChange,
  onSelectedTrackVolumeChange,
  recordingOutput,
  recordingTimelineEdit,
  screenshotOutput,
  selectedScreenshotItemId,
  selectedTrack,
  workspace,
}: EditorToolPanelInputs) {
  // The saved backgrounds are a preference rather than a property of this
  // capture, so they are read and written where every window can see them.
  const [general, applyGeneralSettings] = useEditableGeneralSettings();
  const annotationColors = general?.annotationColors ?? [];
  const backgroundPresets = general?.backgroundPresets ?? [];
  const setBackgroundPresets = (presets: BackgroundPreset[]) => {
    applyGeneralSettings({ backgroundPresets: presets });
  };
  const sceneTemplates = general?.sceneTemplates ?? [];
  // The targets act on what the workspace draws, so a camera saved as a
  // separate file, which it leaves out, has nothing to place, crop or frame.
  const composedTracks = composedVideoTracks(enabledVideoTracks, bakeCamera);
  // Which layer the selection panel is placing, and the handler that commits
  // a new placement for it. Both workspaces already own one; this only says
  // which of them the current selection means.
  const selectionTarget = useLayerSelectionTarget({
    artifact,
    bakeCamera,
    cameraOverlay,
    composedVideoTracks: composedTracks,
    onCameraOverlayChange,
    onRecordingOutputChange,
    onScreenshotOutputChange,
    recordingOutput,
    screenshotOutput,
    selectedScreenshotItemId,
    selectedTrack,
  });
  // The microphone's speech tools, made once: Remove silences shows both in
  // the microphone's Select panel and in the Transcript panel, and the two
  // must share one state and one listen.
  const microphone = useMicrophoneTools({
    artifact,
    edit: recordingTimelineEdit,
    onEditChange: onRecordingTimelineEditChange,
  });
  // An audio track is heard rather than placed, so it is its own selection,
  // with the speech tools where it is the microphone.
  const audioTarget = editorAudioSelectionTarget({
    artifact,
    audioTrackVolumes,
    microphone,
    onSelectedTrackVolumeChange,
    selectedTrack,
  });
  // The source rectangle the crop panel cuts, and the workspace's own commit
  // path for it: the selected recording track's output, or the selected
  // screenshot layer's.
  const cropTarget = editorCropTarget({
    artifact,
    bakeCamera,
    composedVideoTracks: composedTracks,
    onRecordingOutputChange,
    onScreenshotOutputChange,
    recordingOutput,
    screenshotOutput,
    selectedScreenshotItemId,
    selectedTrack,
  });
  // The output canvas the frame panel sizes, and the workspace's own resize
  // path for it: the recording's primary output, or the screenshot canvas.
  const frameTarget = editorFrameTarget({
    artifact,
    bakeCamera,
    composedVideoTracks: composedTracks,
    onCanvasResize,
    onRecordingOutputChange,
    recordingOutput,
    screenshotOutput,
    selectedScreenshotItemId,
    selectedTrack,
  });
  // The annotation the preview has in hand. It belongs to the tools that
  // hit-test arrows rather than to the document, so it reaches the panel
  // through its own channel rather than through the workspace's settings.
  const annotation = useAnnotationSelection(workspace);
  const annotationCount = useAnnotationSelectionCount(workspace);
  const canClearDrawings = useCanClearDrawings(workspace);
  // The scene under the playhead lives with the preview, which knows where
  // the playhead is; it reaches the panel through its own channel too.
  const scene = useRecordingSceneSelection(workspace);
  // Restoring every shortcut is an edit to the timeline the editor already
  // owns, so the panel asks for it here rather than through the preview.
  const shortcuts = useRestoreRecordingKeyboardShortcuts(
    recordingTimelineEdit,
    onRecordingTimelineEditChange,
  );
  // The transcript's place and its asks, where the recording has a microphone
  // to transcribe. The panel's words and playhead travel apart, through their
  // own channel, so they never ride this snapshot.
  const transcript = useTranscriptTools({
    artifact,
    edit: recordingTimelineEdit,
    microphone,
    onEditChange: onRecordingTimelineEditChange,
    workspace,
  });
  const captions = useCaptionTools({
    artifact,
    edit: recordingTimelineEdit,
    general,
    onEditChange: onRecordingTimelineEditChange,
    onGeneralChange: applyGeneralSettings,
  });
  useCaptionCanvasEdits(captions, transcript);
  useToolPanelBridge(
    workspace,
    {
      annotation,
      annotationColors,
      annotationCount,
      background:
        frameTarget?.background ?? DEFAULT_TOOL_PANEL_SNAPSHOT.background,
      backgroundPresets,
      cameraOutput,
      canClearDrawings,
      canRestoreShortcuts: shortcuts.canRestore,
      captions: captions.snapshot,
      crop: cropTarget?.snapshot ?? null,
      cursorEffects,
      frame: frameTarget?.snapshot ?? null,
      hasCursorData: artifact?.kind === "recording" && artifact.hasCursorData,
      hasKeyboardData:
        artifact?.kind === "recording" && artifact.hasKeyboardData,
      isLocked,
      keyboardEffects,
      scene,
      sceneTemplates,
      selection: audioTarget?.selection ?? selectionTarget?.selection ?? null,
      textTemplates: captions.templates,
      transcript: transcript.snapshot,
    },
    editorToolPanelHandlers({
      annotationColors,
      audioTarget,
      backgroundPresets,
      cropTarget,
      frameTarget,
      keyboardEffects,
      onBakeCameraChange,
      onCaptionDefaultsChange: captions.changeDefaults,
      onCaptionsChange: captions.change,
      onCursorEffectsChange,
      onKeyboardEffectsChange,
      onTranscriptRequest: transcript.handle,
      restoreShortcuts: shortcuts.restore,
      saveSceneTemplates: (templates) => {
        applyGeneralSettings({ sceneTemplates: templates });
      },
      sceneTemplates,
      selectionTarget,
      setAnnotationColors: (colors) => {
        applyGeneralSettings({ annotationColors: colors });
      },
      setBackgroundPresets,
      workspace,
    }),
  );
  // Subtitles are made of the transcript, so the export offers them once
  // there is one.
  return { hasTranscript: transcript.snapshot?.status === "ready" };
}
