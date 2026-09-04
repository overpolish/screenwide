// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Camera,
  CameraOff,
  CircleX,
  Keyboard,
  KeyboardOff,
  Lock,
  Mic,
  MicOff,
  MousePointer2,
  MousePointer2Off,
  Volume2,
  VolumeOff,
} from "lucide-react";
import { ReactNode, useRef, useState } from "react";

import { Button } from "../../../components/base/button/button";
import { IconButton } from "../../../components/base/button/icon-button";
import { ButtonGroup } from "../../../components/base/button-group/button-group";
import { Overlay } from "../../../components/base/overlay/overlay";
import { cn } from "../../../lib/styling";
import { RecordingFps, RecordingInputs } from "../../recording-inputs/types";
import { RecordingMode } from "../../recording-sources/types";
import { canStartRecording } from "../can-record";
import { RecordingStatus, ScreenshotAction, ScreenshotState } from "../types";

import { RecordingBarInputToggle as InputToggle } from "./recording-bar-input-toggle";
import { RecordingBarRecordAction } from "./recording-bar-record-action";
import { RecordingBarScreenshotActions } from "./recording-bar-screenshot-actions";
import { RecordingModePicker } from "./recording-mode-picker";

type Anchor = Pick<DOMRect, "height" | "width" | "x" | "y">;

type RecordingBarProps = {
  fps?: RecordingFps;
  hasCameraWarning?: boolean;
  hasMicrophoneWarning?: boolean;
  hasSelectedMonitor?: boolean;
  hasSelectedWindow?: boolean;
  hasSystemAudioWarning?: boolean;
  initialFps?: RecordingFps;
  initialInputs?: Partial<RecordingInputs>;
  initialMode?: RecordingMode;
  inputs?: RecordingInputs;
  isCameraLocked?: boolean;
  isLocked?: boolean;
  isMicrophoneLocked?: boolean;
  isScreenshotLocked?: boolean;
  mode?: RecordingMode;
  onCameraLockedPress?: () => void;
  onCancel?: () => void;
  onFocusPendingExport?: () => void;
  onFpsChange?: (fps: RecordingFps) => void;
  onInputChange?: (input: keyof RecordingInputs, selected: boolean) => void;
  onInteract?: () => void;
  onMicrophoneLockedPress?: () => void;
  onModeChange?: (mode: RecordingMode) => void;
  onOptions?: (anchor: Anchor, focusContents: boolean) => void;
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
  pendingExports?: { recording: boolean; screenshot: boolean };
  screenshotAction?: ScreenshotAction;
  screenshotState?: ScreenshotState;
  sourceSelector?: ReactNode;
  status?: RecordingStatus;
};

const defaultInputs: RecordingInputs = {
  camera: false,
  keyboardShortcuts: false,
  microphone: false,
  showCursor: true,
  systemAudio: false,
};

export function RecordingBar({
  fps: controlledFps,
  hasCameraWarning = false,
  hasMicrophoneWarning = false,
  hasSelectedMonitor = false,
  hasSelectedWindow = false,
  hasSystemAudioWarning = false,
  initialFps = 60,
  initialInputs,
  initialMode = "screen",
  inputs: controlledInputs,
  isCameraLocked,
  isLocked,
  isMicrophoneLocked,
  isScreenshotLocked,
  mode: controlledMode,
  onCameraLockedPress,
  onCancel,
  onFocusPendingExport,
  onFpsChange,
  onInputChange,
  onInteract,
  onMicrophoneLockedPress,
  onModeChange,
  onOptions,
  onPointerUp,
  onRecord,
  onRequiredPermissionsPress,
  onScreenshot,
  onScreenshotToClipboard,
  onScrollingScreenshot,
  pendingExports = { recording: false, screenshot: false },
  screenshotAction = "export",
  screenshotState = "idle",
  sourceSelector,
  status = "idle",
}: RecordingBarProps) {
  const [uncontrolledMode, setUncontrolledMode] =
    useState<RecordingMode>(initialMode);
  const [uncontrolledFps, setUncontrolledFps] =
    useState<RecordingFps>(initialFps);
  const [uncontrolledInputs, setUncontrolledInputs] = useState<RecordingInputs>(
    {
      ...defaultInputs,
      ...initialInputs,
    },
  );
  const optionsButtonRef = useRef<HTMLButtonElement>(null);
  const sourceSelectorRef = useRef<HTMLDivElement>(null);

  const mode = controlledMode ?? uncontrolledMode;
  const fps = controlledFps ?? uncontrolledFps;
  const inputs = controlledInputs ?? uncontrolledInputs;

  const setInput = (input: keyof RecordingInputs, selected: boolean) => {
    if (controlledInputs === undefined) {
      setUncontrolledInputs((current) => ({
        ...current,
        [input]: selected,
      }));
    }
    onInputChange?.(input, selected);
  };

  const isAudioOnly = mode === "audio";
  const isScreenCapture = ["screen", "region", "window"].includes(mode);
  // The bar is hidden by Rust while a recording runs; disabling it as well
  // keeps a stale window from starting a second one.
  const isRecordingActive = status !== "idle";
  const isRecordingWorkspaceOpen = pendingExports.recording;
  const isCapturingStill = screenshotState === "pending";
  const exportScreenshotState =
    screenshotAction === "export" ? screenshotState : "idle";
  const clipboardScreenshotState =
    screenshotAction === "clipboard" ? screenshotState : "idle";
  const scrollingScreenshotState =
    screenshotAction === "scrolling" ? screenshotState : "idle";
  const hasScreenshotSource =
    mode === "window" ? hasSelectedWindow : hasSelectedMonitor;
  const canCaptureStill =
    isScreenCapture &&
    hasScreenshotSource &&
    !isScreenshotLocked &&
    !isRecordingActive;
  // A pending recording no longer stands in a screenshot's way: it waits in
  // its own window while the screenshot workspace opens beside it.
  const canExportScreenshot = canCaptureStill;
  const canCopyScreenshot = canCaptureStill;
  const canCaptureScrollingScreenshot =
    canCaptureStill && mode === "region" && onScrollingScreenshot !== undefined;
  const canRecordIgnoringExport =
    !isRecordingActive &&
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
  const canRecord = canRecordIgnoringExport && !isRecordingWorkspaceOpen;
  // Recording that only the pending recording stands in the way of: the button
  // stays pressable and brings that export forward rather than going dead,
  // which is the same escape hatch the global shortcuts take.
  const isRecordBlockedByExport =
    canRecordIgnoringExport && isRecordingWorkspaceOpen;

  return (
    <main
      className="window-surface px-section pt-section pb-control flex h-full min-h-[120px] w-full min-w-[680px] flex-col overflow-hidden text-content-fg"
      data-tauri-drag-region="deep"
      onKeyDownCapture={(event) => {
        if (
          optionsButtonRef.current?.contains(event.target as Node) ||
          sourceSelectorRef.current?.contains(event.target as Node)
        ) {
          return;
        }
        onInteract?.();
      }}
      onPointerDownCapture={(event) => {
        if (
          optionsButtonRef.current?.contains(event.target as Node) ||
          sourceSelectorRef.current?.contains(event.target as Node)
        ) {
          return;
        }
        onInteract?.();
      }}
      onPointerUpCapture={(event) => {
        if (sourceSelectorRef.current?.contains(event.target as Node)) return;
        onPointerUp?.();
      }}
    >
      <Overlay
        blur="sm"
        isOpen={Boolean(isScreenshotLocked) && isScreenCapture}
      >
        <IconButton
          aria-label="Open permissions"
          className="group"
          onPress={onRequiredPermissionsPress}
        >
          <Lock className="transition-transform group-data-[hovered]:scale-110" />
        </IconButton>
      </Overlay>

      <div
        className={cn(
          "gap-section relative flex min-h-0 w-full grow items-center justify-center",
          sourceSelector &&
            "pt-[calc(var(--spacing-window-inset)+var(--spacing-control))]",
        )}
      >
        <RecordingModePicker
          isDisabled={isRecordingActive}
          mode={mode}
          onChange={(nextMode) => {
            if (controlledMode === undefined) {
              setUncontrolledMode(nextMode);
            }
            onModeChange?.(nextMode);
          }}
        />

        {sourceSelector ? (
          <div
            className="gap-control absolute inset-x-0 top-0 flex h-6"
            ref={sourceSelectorRef}
          >
            {sourceSelector}
          </div>
        ) : null}

        <IconButton
          aria-label="Cancel"
          className="order-first"
          iconSize="prominent"
          onPress={onCancel}
        >
          <CircleX />
        </IconButton>

        <div className="gap-tight flex min-w-[120px] flex-col">
          <ButtonGroup
            aria-label="Recording inputs"
            className="gap-tight justify-between"
          >
            <InputToggle
              hasWarning={hasSystemAudioWarning}
              isDisabled={isRecordingActive}
              isSelected={inputs.systemAudio}
              label="System audio"
              off={<VolumeOff />}
              on={<Volume2 />}
              onChange={(selected) => {
                setInput("systemAudio", selected);
              }}
              warningLabel="One or more selected system audio applications are not detected"
            />
            <InputToggle
              hasWarning={hasMicrophoneWarning}
              isDisabled={isRecordingActive}
              isLocked={isMicrophoneLocked}
              isSelected={inputs.microphone}
              label="Microphone"
              off={<MicOff />}
              on={<Mic />}
              onChange={(selected) => {
                setInput("microphone", selected);
              }}
              onLockedPress={onMicrophoneLockedPress}
              warningLabel="Selected microphone is not detected"
            />
            <InputToggle
              hasWarning={hasCameraWarning}
              isDisabled={isAudioOnly || isRecordingActive}
              isLocked={isCameraLocked}
              isReadOnly={mode === "camera"}
              isSelected={mode === "camera" || (!isAudioOnly && inputs.camera)}
              label="Camera"
              off={<CameraOff />}
              on={<Camera />}
              onChange={(selected) => {
                setInput("camera", selected);
              }}
              onLockedPress={onCameraLockedPress}
              warningLabel="Selected camera is not detected"
            />
            <InputToggle
              isDisabled={!isScreenCapture || isRecordingActive}
              isSelected={isScreenCapture && inputs.showCursor}
              label="Show cursor"
              off={<MousePointer2Off />}
              on={<MousePointer2 />}
              onChange={(selected) => {
                setInput("showCursor", selected);
              }}
            />
            <InputToggle
              isDisabled={!isScreenCapture || isRecordingActive}
              isSelected={isScreenCapture && inputs.keyboardShortcuts}
              label="Keyboard shortcuts"
              off={<KeyboardOff />}
              on={<Keyboard />}
              onChange={(selected) => {
                setInput("keyboardShortcuts", selected);
              }}
            />
          </ButtonGroup>

          <div
            className="flex justify-center"
            onPointerDown={(event) => {
              event.stopPropagation();
            }}
          >
            <Button
              isDisabled={isRecordingActive}
              onPress={(event) => {
                const bounds =
                  optionsButtonRef.current?.getBoundingClientRect();
                if (bounds) {
                  onOptions?.(
                    bounds.toJSON() as Anchor,
                    ["keyboard", "virtual"].includes(event.pointerType),
                  );
                }
              }}
              ref={optionsButtonRef}
              size="compact"
              variant="ghost"
            >
              Options
            </Button>
          </div>
        </div>

        <RecordingBarScreenshotActions
          canCaptureScrollingScreenshot={canCaptureScrollingScreenshot}
          canCopyScreenshot={canCopyScreenshot}
          canExportScreenshot={canExportScreenshot}
          clipboardScreenshotState={clipboardScreenshotState}
          exportScreenshotState={exportScreenshotState}
          isCapturingStill={isCapturingStill}
          onScreenshot={onScreenshot}
          onScreenshotToClipboard={onScreenshotToClipboard}
          onScrollingScreenshot={onScrollingScreenshot}
          scrollingScreenshotState={scrollingScreenshotState}
        />

        <div className="gap-tight flex flex-col items-center justify-center self-stretch">
          <RecordingBarRecordAction
            canRecord={canRecord}
            isLocked={Boolean(isLocked)}
            isRecordBlockedByExport={isRecordBlockedByExport}
            onFocusPendingExport={onFocusPendingExport}
            onRecord={onRecord}
            onRequiredPermissionsPress={onRequiredPermissionsPress}
          />

          <InputToggle
            isDisabled={isRecordingActive}
            isSelected={fps === 60}
            label="Frames per second"
            off={<span className="text-xs tabular-nums">30</span>}
            on={<span className="text-xs tabular-nums">60</span>}
            onChange={(smooth) => {
              const nextFps: RecordingFps = smooth ? 60 : 30;
              if (controlledFps === undefined) setUncontrolledFps(nextFps);
              onFpsChange?.(nextFps);
            }}
          />
        </div>
      </div>
    </main>
  );
}
