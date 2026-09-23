// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  withAnnotationColor,
  withoutAnnotationColor,
} from "../../../components/shared/annotation-style/palette";
import { BackgroundPreset } from "../../../components/shared/background-picker/background";
import { useEditableGeneralSettings } from "../../settings/use-general-settings";
import {
  applyAnnotationAngle,
  applyAnnotationAnimated,
  applyAnnotationReverse,
  applyAnnotationStyle,
  useAnnotationSelection,
} from "../annotation-channel";
import { useRestoreRecordingKeyboardShortcuts } from "../components/use-restore-recording-keyboard-shortcuts";
import { keyboardMaximumSizePercent } from "../keyboard-effect-geometry";
import {
  applyKeyboardShortcutToAll,
  placeKeyboardShortcut,
  resetKeyboardShortcut,
  useKeyboardShortcutSelection,
} from "../keyboard-shortcut-channel";
import { RecordingTimelineEdit } from "../recording-timeline-edit";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  EditorArtifact,
  EditorKind,
  KeyboardEffectSettings,
  RecordingVideoTrackId,
} from "../types";

import { cropPanelHandlers, editorCropTarget } from "./crop-target";
import { editorFrameTarget, framePanelHandlers } from "./frame-target";
import {
  audioPanelHandlers,
  editorAudioSelectionTarget,
  editorSelectionTarget,
  selectionPanelHandlers,
} from "./selection-target";
import { useToolPanelBridge } from "./tool-panel-bridge";
import { DEFAULT_TOOL_PANEL_SNAPSHOT } from "./tool-panel-store";

type EditorToolPanelInputs = {
  artifact: EditorArtifact | null;
  audioTrackVolumes: AudioTrackVolume[];
  bakeCamera: boolean;
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
  // Which layer the selection panel is placing, and the handler that commits
  // a new placement for it. Both workspaces already own one; this only says
  // which of them the current selection means.
  const selectionTarget = editorSelectionTarget({
    artifact,
    bakeCamera,
    cameraOverlay,
    enabledVideoTracks,
    onBakeCameraChange,
    onCameraOverlayChange,
    onRecordingOutputChange,
    onScreenshotOutputChange,
    recordingOutput,
    screenshotOutput,
    selectedScreenshotItemId,
    selectedTrack,
  });
  // An audio track is heard rather than placed, so it is its own selection:
  // the level the editor holds for it, and the editor's own way of setting it.
  const audioTarget = editorAudioSelectionTarget({
    artifact,
    audioTrackVolumes,
    onSelectedTrackVolumeChange,
    selectedTrack,
  });
  // The source rectangle the crop panel cuts, and the workspace's own commit
  // path for it: the selected recording track's output, or the selected
  // screenshot layer's.
  const cropTarget = editorCropTarget({
    artifact,
    bakeCamera,
    enabledVideoTracks,
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
    enabledVideoTracks,
    onCanvasResize,
    onRecordingOutputChange,
    recordingOutput,
    screenshotOutput,
    selectedScreenshotItemId,
    selectedTrack,
  });
  // The shortcut the preview has in hand, and the edits that move it. Both
  // live with the preview, which is the only place that knows which fragment
  // is on screen; the panel reaches them the way the padding controls reach
  // the recentre calls.
  const shortcutSelection = useKeyboardShortcutSelection(workspace);
  // The annotation the preview has in hand. It belongs to the tools that
  // hit-test arrows rather than to the document, so it reaches the panel the
  // way the selected shortcut does rather than through the workspace's
  // settings.
  const annotation = useAnnotationSelection(workspace);
  // Restoring and resetting every shortcut is an edit to the timeline the
  // editor already owns, so the panel asks for it here rather than through the
  // preview.
  const shortcuts = useRestoreRecordingKeyboardShortcuts(
    recordingTimelineEdit,
    onRecordingTimelineEditChange,
  );
  const keyboardOutput = recordingOutput
    ? screenshotOutputDimensions(recordingOutput.primary)
    : null;
  useToolPanelBridge(
    workspace,
    {
      annotation,
      annotationColors,
      background:
        frameTarget?.background ?? DEFAULT_TOOL_PANEL_SNAPSHOT.background,
      backgroundPresets,
      canRestoreShortcuts: shortcuts.canRestore,
      crop: cropTarget?.snapshot ?? null,
      cursorEffects,
      frame: frameTarget?.snapshot ?? null,
      hasCursorData: artifact?.kind === "recording" && artifact.hasCursorData,
      hasKeyboardData:
        artifact?.kind === "recording" && artifact.hasKeyboardData,
      isLocked,
      keyboardEffects,
      keyboardMaximum:
        artifact?.kind === "recording" && keyboardOutput
          ? keyboardMaximumSizePercent({
              ...keyboardOutput,
              maximumWidthUnits: artifact.keyboardMaximumWidthUnits,
            })
          : DEFAULT_TOOL_PANEL_SNAPSHOT.keyboardMaximum,
      // A selected shortcut is what the Select tool has in hand, so it is the
      // selection the panel shows rather than the layer underneath it.
      selection:
        shortcutSelection ??
        audioTarget?.selection ??
        selectionTarget?.selection ??
        null,
    },
    {
      onAnnotationAngleChange: (angle) => {
        applyAnnotationAngle(workspace, angle);
      },
      onAnnotationAnimatedChange: (animated) => {
        applyAnnotationAnimated(workspace, animated);
      },
      onAnnotationColorRemove: (color) => {
        applyGeneralSettings({
          annotationColors: withoutAnnotationColor(annotationColors, color),
        });
      },
      onAnnotationColorSave: (color) => {
        applyGeneralSettings({
          annotationColors: withAnnotationColor(annotationColors, color),
        });
      },
      onAnnotationReverse: () => {
        applyAnnotationReverse(workspace);
      },
      onAnnotationStyleChange: (style) => {
        applyAnnotationStyle(workspace, style);
      },
      onBackgroundPresetRemove: (id) => {
        setBackgroundPresets(
          backgroundPresets.filter((preset) => preset.id !== id),
        );
      },
      onBackgroundPresetSave: (preset) => {
        setBackgroundPresets([
          ...backgroundPresets.filter(
            (saved) => saved.id !== preset.id && saved.name !== preset.name,
          ),
          preset,
        ]);
      },
      onCursorEffectsChange,
      onKeyboardEffectsChange: (settings) => {
        onKeyboardEffectsChange?.({ ...keyboardEffects, ...settings });
      },
      // The default place is the absence of a position: the bridge is JSON,
      // which cannot carry an undefined, so the reset is its own request.
      onKeyboardPositionReset: () => {
        const {
          positionXPercent: _x,
          positionYPercent: _y,
          ...rest
        } = keyboardEffects;
        onKeyboardEffectsChange?.(rest);
      },
      onKeyboardShortcutsResetAll: shortcuts.reset,
      onKeyboardShortcutsRestore: shortcuts.restore,
      onShortcutApplyToAll: () => {
        applyKeyboardShortcutToAll(workspace);
      },
      onShortcutPlacementChange: (placement) => {
        placeKeyboardShortcut(workspace, placement);
      },
      onShortcutReset: () => {
        resetKeyboardShortcut(workspace);
      },
      ...audioPanelHandlers(audioTarget),
      ...cropPanelHandlers(cropTarget),
      ...framePanelHandlers(frameTarget),
      ...selectionPanelHandlers(selectionTarget),
    },
  );
}
