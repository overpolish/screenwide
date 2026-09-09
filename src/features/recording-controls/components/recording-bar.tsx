// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { X } from "lucide-react";
import { Ref, useState } from "react";

import { IconButton } from "../../../components/base/button/icon-button";
import { RecordingInputs } from "../../recording-inputs/types";
import {
  MonitorDetails,
  RecordingMode,
  WindowDetails,
} from "../../recording-sources/types";
import { canStartRecording } from "../can-record";
import { RecordingStatus, ScreenshotAction, ScreenshotState } from "../types";

import { RecordingBarCaptureActions } from "./recording-bar-capture-actions";
import { RecordingBarInputs } from "./recording-bar-inputs";
import { RecordingTypePicker } from "./recording-type-picker";

type RecordingBarProps = {
  hasCameraWarning?: boolean;
  hasMicrophoneWarning?: boolean;
  hasSelectedMonitor?: boolean;
  hasSelectedWindow?: boolean;
  hasSystemAudioWarning?: boolean;
  initialInputs?: Partial<RecordingInputs>;
  initialMode?: RecordingMode;
  inputs?: RecordingInputs;
  isCameraLocked?: boolean;
  isLocked?: boolean;
  isMicrophoneLocked?: boolean;
  /** Whether the bar is on screen: its camera and audio previews are native
   * streams, and a dismissed bar stays mounted. */
  isPreviewActive?: boolean;
  isScreenshotLocked?: boolean;
  mode?: RecordingMode;
  /** A still of each attached display by its id, drawn as the Screen
   * segment's icon for whichever display is chosen. */
  monitorThumbnails?: Record<number, string>;
  onCameraLockedPress?: () => void;
  onCancel?: () => void;
  onChooseMonitor?: (anchor: DOMRect, fromKeyboard: boolean) => void;
  onChooseWindow?: (anchor: DOMRect, fromKeyboard: boolean) => void;
  onFocusPendingEditor?: () => void;
  onInputChange?: (input: keyof RecordingInputs, selected: boolean) => void;
  onInteract?: () => void;
  onMicrophoneLockedPress?: () => void;
  onModeChange?: (mode: RecordingMode) => void;
  onPointerUp?: () => void;
  onRecord?: () => void;
  onRequiredPermissionsPress?: () => void;
  onScreenshot?: () => void;
  onScreenshotToClipboard?: () => void;
  onScrollingScreenshot?: () => void;
  /**
   * Which workspaces are holding unsaved work. Each has a window of its own,
   * so only a pending recording stands in the way of starting another.
   */
  pendingEditors?: { recording: boolean; screenshot: boolean };
  /** The bar's root, which a window can measure to size itself to it. */
  ref?: Ref<HTMLElement>;
  screenshotAction?: ScreenshotAction;
  screenshotState?: ScreenshotState;
  selectedMonitor?: MonitorDetails | null;
  selectedWindow?: WindowDetails | null;
  status?: RecordingStatus;
};

const defaultInputs: RecordingInputs = {
  camera: false,
  microphone: false,
  systemAudio: false,
};

/**
 * The recording bar is being rebuilt one section at a time on the native
 * primitives. The props are the full contract the window and stories drive;
 * sections return as they are redesigned.
 */
export function RecordingBar(props: RecordingBarProps) {
  const {
    hasCameraWarning = false,
    hasMicrophoneWarning = false,
    hasSelectedMonitor = false,
    hasSelectedWindow = false,
    hasSystemAudioWarning = false,
    initialInputs,
    initialMode = "screen",
    inputs: controlledInputs,
    isCameraLocked,
    isLocked,
    isMicrophoneLocked,
    isPreviewActive = true,
    isScreenshotLocked,
    mode: controlledMode,
    monitorThumbnails = {},
    onCameraLockedPress,
    onCancel,
    onChooseMonitor,
    onChooseWindow,
    onFocusPendingEditor,
    onInputChange,
    onInteract,
    onMicrophoneLockedPress,
    onModeChange,
    onPointerUp,
    onRecord,
    onRequiredPermissionsPress,
    onScreenshot,
    onScreenshotToClipboard,
    onScrollingScreenshot,
    pendingEditors = { recording: false, screenshot: false },
    ref,
    screenshotState = "idle",
    selectedMonitor = null,
    selectedWindow = null,
    status = "idle",
  } = props;
  const [uncontrolledInputs, setUncontrolledInputs] = useState<RecordingInputs>(
    { ...defaultInputs, ...initialInputs },
  );
  const inputs = controlledInputs ?? uncontrolledInputs;
  const changeInput = (input: keyof RecordingInputs, selected: boolean) => {
    if (controlledInputs === undefined) {
      setUncontrolledInputs((current) => ({ ...current, [input]: selected }));
    }
    onInputChange?.(input, selected);
  };
  const [uncontrolledMode, setUncontrolledMode] =
    useState<RecordingMode>(initialMode);
  const mode = controlledMode ?? uncontrolledMode;
  const selectMode = (nextMode: RecordingMode) => {
    if (nextMode === mode) return;
    if (controlledMode === undefined) {
      setUncontrolledMode(nextMode);
    }
    onModeChange?.(nextMode);
  };

  const isScreenCapture = ["region", "screen", "window"].includes(mode);
  const hasSource = mode === "window" ? hasSelectedWindow : hasSelectedMonitor;
  const canScreenshot =
    isScreenCapture && hasSource && !isScreenshotLocked && status === "idle";
  const canScrollingScreenshot = canScreenshot && mode === "region";
  const isCapturing = screenshotState === "pending";
  // The bar is hidden by Rust while a recording runs; disabling it as well
  // keeps a stale window from starting a second one.
  const canRecordIgnoringEditor =
    status === "idle" &&
    canStartRecording({
      hasCameraWarning,
      hasMicrophoneWarning,
      hasSelectedMonitor,
      hasSelectedWindow,
      hasSystemAudioWarning,
      inputs,
      isCameraLocked: Boolean(isCameraLocked),
      isMicrophoneLocked: Boolean(isMicrophoneLocked),
      isScreenLocked: Boolean(isLocked),
      mode,
    });
  const isRecordingWorkspaceOpen = pendingEditors.recording;
  const canRecord = canRecordIgnoringEditor && !isRecordingWorkspaceOpen;
  // Recording that only the pending recording stands in the way of: the button
  // stays pressable and brings that editor forward rather than going dead,
  // which is the same escape hatch the global shortcuts take.
  const isRecordBlockedByEditor =
    canRecordIgnoringEditor && isRecordingWorkspaceOpen;

  return (
    <main
      // One row of 40px controls with 8px on every side, which makes the bar
      // 56pt tall. The four groups are set apart by the section gap, half
      // again the gap inside a group, which is how a toolbar shows grouping
      // without separators. The row is as wide as its controls and the
      // window follows it, so no state leaves slack.
      className="window-surface flex h-full w-max items-center gap-section overflow-hidden p-control-inset text-content-fg"
      data-tauri-drag-region="deep"
      onKeyDownCapture={() => {
        onInteract?.();
      }}
      onPointerDownCapture={() => {
        onInteract?.();
      }}
      onPointerUpCapture={() => {
        onPointerUp?.();
      }}
      ref={ref}
    >
      <IconButton aria-label="Cancel" onPress={onCancel} size="capture">
        <X />
      </IconButton>

      <RecordingTypePicker
        isDisabled={status !== "idle" || Boolean(isLocked)}
        mode={mode}
        monitorThumbnails={monitorThumbnails}
        // Pressing a segment that already owns the selection opens its source
        // list; pressing it from another mode selects it first, because the
        // choice only makes sense in its own mode.
        onChooseMonitor={(anchor, fromKeyboard) => {
          selectMode("screen");
          onChooseMonitor?.(anchor, fromKeyboard);
        }}
        onChooseWindow={(anchor, fromKeyboard) => {
          selectMode("window");
          onChooseWindow?.(anchor, fromKeyboard);
        }}
        onModeChange={(nextMode) => {
          selectMode(nextMode);
        }}
        selectedMonitor={selectedMonitor}
        selectedWindow={selectedWindow}
      />

      <RecordingBarInputs
        hasCameraWarning={hasCameraWarning}
        hasMicrophoneWarning={hasMicrophoneWarning}
        hasSystemAudioWarning={hasSystemAudioWarning}
        inputs={inputs}
        isCameraLocked={Boolean(isCameraLocked)}
        isDisabled={status !== "idle" || Boolean(isLocked)}
        isMicrophoneLocked={Boolean(isMicrophoneLocked)}
        isPreviewActive={isPreviewActive}
        mode={mode}
        onCameraLockedPress={onCameraLockedPress}
        onInputChange={changeInput}
        onMicrophoneLockedPress={onMicrophoneLockedPress}
      />

      <RecordingBarCaptureActions
        canRecord={canRecord}
        canScreenshot={canScreenshot}
        canScrollingScreenshot={canScrollingScreenshot}
        isCapturing={isCapturing}
        isLocked={Boolean(isLocked)}
        isRecordBlockedByEditor={isRecordBlockedByEditor}
        onFocusPendingEditor={onFocusPendingEditor}
        onRecord={onRecord}
        onRequiredPermissionsPress={onRequiredPermissionsPress}
        onScreenshot={onScreenshot}
        onScreenshotToClipboard={onScreenshotToClipboard}
        onScrollingScreenshot={onScrollingScreenshot}
        screenshotState={screenshotState}
      />
    </main>
  );
}

export type { RecordingBarProps };
