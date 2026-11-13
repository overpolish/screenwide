// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEditableGeneralSettings } from "../../settings/use-general-settings";
import {
  useAnnotationSelection,
  useAnnotationSelectionCount,
} from "../annotations/annotation-channel";
import { composedVideoTracks } from "../export/camera-output";
import { useCaptionCanvasEdits } from "../recording/captions/use-caption-canvas-edits";
import { useCaptionTools } from "../recording/captions/use-caption-tools";
import { useMicrophoneTools } from "../recording/microphone/use-microphone-tools";
import {
  recordingSceneControls,
  useRecordingSceneSelection,
} from "../recording/scenes/recording-scene-channel";
import { useTranscriptTools } from "../recording/transcript/use-transcript-tools";
import { useRestoreRecordingKeyboardShortcuts } from "../timeline/keyboard/use-restore-recording-keyboard-shortcuts";
import {
  stripIslandControls,
  useStripIslandChoice,
} from "../timeline/recording-strip/strip-island-channel";
import { recordingAudioStreamIndex, RecordingTrackId } from "../types";

import { useCropAspectStore } from "./crop/crop-aspect-store";
import { editorCropTarget } from "./crop/crop-target";
import { editorToolPanelHandlers } from "./editor-tool-panel-handlers";
import { EditorToolPanelInputs } from "./editor-tool-panel-inputs";
import { BackgroundPreset } from "./frame/background/background";
import { editorFrameTarget } from "./frame/frame-target";
import {
  editorAudioSelection,
  editorSoundTracks,
} from "./selection/selection-target";
import { useLayerSelectionTarget } from "./selection/use-layer-selection-target";
import { useToolPanelBridge } from "./tool-panel-bridge";
import { DEFAULT_TOOL_PANEL_SNAPSHOT } from "./tool-panel-store";
import { editorTracks } from "./tracks/editor-tracks";

import type { AudioClipKind } from "../../../bindings/AudioClipKind";

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
  exportIsland,
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
  trackInclusion,
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
  // The microphone's speech tools, made once: the Sound and Script islands
  // share one state and one listen.
  const microphone = useMicrophoneTools({
    artifact,
    edit: recordingTimelineEdit,
    onEditChange: onRecordingTimelineEditChange,
  });
  // An audio track is heard rather than placed, so it is its own selection,
  // with the speech tools where it is the microphone.
  const audioSelection = editorAudioSelection({
    artifact,
    microphone,
    selectedTrack,
  });
  // The scene under the playhead lives with the preview and reaches the
  // panel through its own channel, as what is chosen on the strip does.
  const scene = useRecordingSceneSelection(workspace);
  const sceneControls = recordingSceneControls(workspace);
  // The source rectangle the crop panel cuts, and the workspace's own commit
  // path for it: the selected recording track's output, the screen's own
  // crop in the scene under the playhead, or the selected screenshot layer's.
  const cropAspects = useCropAspectStore((state) => state.aspects);
  const cropTarget = editorCropTarget({
    artifact,
    aspects: cropAspects,
    bakeCamera,
    composedVideoTracks: composedTracks,
    onRecordingOutputChange,
    onScreenshotOutputChange,
    recordingOutput,
    scene,
    sceneControls,
    screenshotOutput,
    selectedScreenshotItemId,
    selectedTrack,
  });
  // The output canvas the frame panel sizes, and the workspace's own resize
  // path for it: the recording's primary output, or the screenshot canvas.
  const frameTarget = editorFrameTarget({
    artifact,
    bakeCamera,
    cameraOverlay,
    composedVideoTracks: composedTracks,
    onCameraOverlayChange,
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
  const strip = useStripIslandChoice(workspace);
  const tracks = {
    ...trackInclusion,
    artifact,
    enabledVideoTracks,
    onSoundsHeard: (kind: AudioClipKind, isHeard: boolean) => {
      stripIslandControls(workspace)?.hearSounds(kind, isHeard);
    },
    sounds: strip.sounds,
  };
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
      addingSounds: strip.adding,
      annotation,
      annotationColors,
      annotationCount,
      background:
        frameTarget?.background ?? DEFAULT_TOOL_PANEL_SNAPSHOT.background,
      backgroundPresets,
      cameraOutput,
      canRestoreShortcuts: shortcuts.canRestore,
      captions: captions.snapshot,
      chosenShortcuts: strip.shortcuts,
      crop: cropTarget?.snapshot ?? null,
      cursorEffects,
      export: exportIsland.snapshot && {
        ...exportIsland.snapshot,
        revealWhenDone: general?.openLocationAfterExport ?? false,
      },
      frame: frameTarget?.snapshot ?? null,
      hasCursorData: artifact?.kind === "recording" && artifact.hasCursorData,
      isLocked,
      keyboardEffects,
      scene,
      sceneTemplates,
      selection: audioSelection ?? selectionTarget?.selection ?? null,
      soundTracks: editorSoundTracks({
        artifact,
        audioTrackVolumes,
        microphone,
      }),
      stripItem: strip.item,
      textTemplates: captions.templates,
      tracks: editorTracks(tracks),
      transcript: transcript.snapshot,
    },
    {
      ...editorToolPanelHandlers({
        annotationColors,
        backgroundPresets,
        cropTarget,
        frameTarget,
        keyboardEffects,
        microphone,
        onCaptionDefaultsChange: captions.changeDefaults,
        onCaptionsChange: captions.change,
        onCursorEffectsChange,
        onKeyboardEffectsChange,
        // The Sound island names each track by its id, as the editor selects
        // it, and the editor holds each stream's level.
        onTrackVolumeChange: (trackId, decibels) => {
          const streamIndex = recordingAudioStreamIndex(
            trackId as RecordingTrackId,
          );
          if (streamIndex !== null)
            onTrackVolumeChange?.(streamIndex, decibels);
        },
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
        tracks,
        workspace,
      }),
      onExportPatch: (patch) => {
        if (patch.revealWhenDone !== undefined)
          applyGeneralSettings({
            openLocationAfterExport: patch.revealWhenDone,
          });
        exportIsland.onPatch(patch);
      },
    },
  );
  // Subtitles are made of the transcript, so the export offers them once
  // there is one.
  return { hasTranscript: transcript.snapshot?.status === "ready" };
}
