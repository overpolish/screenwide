// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Pause, Play, Square, Trash2, X } from "lucide-react";
import { useLayoutEffect, useRef } from "react";

import {
  IconButton,
  IconToggleButton,
} from "../../../components/base/button/icon-button";
import { CircularProgress } from "../../../components/base/circular-progress/circular-progress";
import { ContentRotate } from "../../../components/base/content-rotate/content-rotate";
import { ConfirmActionButton } from "../../../components/shared/confirm-action-button/confirm-action-button";
import { cn } from "../../../lib/styling";
import { AudioMeter } from "../../audio-inputs/components/audio-meter";
import { CameraThumbnail } from "../../recording-inputs/camera-thumbnail";
import { formatElapsedTime } from "../elapsed-time";
import { RecordingStatus } from "../types";
import { RecordingMonitorSnapshot } from "../use-recording-monitor";

/**
 * "00:00:08" in 13px SF Pro tabular figures measures about 58px, so 64px holds
 * the longest timer with a little slack and the pill never breathes as the
 * digits change. It also gives the starting and finishing slot, which takes
 * the same width, room for its spinner and label.
 */
const TIME_SLOT_CLASS = "w-16";

/**
 * The starting and finishing slot takes the timer's width as its floor, so a
 * countdown digit sits where the timer was; the spinner and its fixed-width
 * label are wider than that and the slot grows to hold them rather than
 * spilling out of the pill.
 */
const BUSY_SLOT_CLASS = "min-w-16";

/**
 * Rotates each digit on its own, so a tick only animates what actually
 * changed: 58 to 59 moves the units alone, while 59 to 00 moves both. Rotating
 * the pair as one unit would swing the tens digit on every single second.
 */
function RotatingDigits({ value }: { value: string }) {
  const leading = value.slice(0, -1);
  const last = value.slice(-1);

  return (
    <>
      <ContentRotate contentKey={leading}>{leading}</ContentRotate>
      <ContentRotate contentKey={last}>{last}</ContentRotate>
    </>
  );
}

type DiscardButtonProps = {
  onDiscard?: () => void;
};

/**
 * Two-step, because discarding sits one button away from stopping: the bin
 * swaps in place to a check, and only pressing that check discards. The armed
 * colour is the button's own, so it reads destructive the same way everywhere.
 */
function DiscardButton({ onDiscard }: DiscardButtonProps) {
  return (
    <ConfirmActionButton
      armedIcon={<Check />}
      armedLabel="Confirm discarding"
      idleIcon={<Trash2 />}
      idleLabel="Discard recording"
      onConfirm={onDiscard}
      variant="icon"
    />
  );
}

type RecordingDockProps = {
  countdownSeconds?: number;
  elapsedMs?: number;
  monitor?: RecordingMonitorSnapshot;
  onDiscard?: () => void;
  onPauseChange?: (isPaused: boolean) => void;
  onPointerUp?: () => void;
  onStop?: () => void;
  onWidthChange?: (width: number) => void;
  status?: RecordingStatus;
};

export function RecordingDock({
  countdownSeconds = 0,
  elapsedMs = 0,
  monitor,
  onDiscard,
  onPauseChange,
  onPointerUp,
  onStop,
  onWidthChange,
  status = "recording",
}: RecordingDockProps) {
  const dockRef = useRef<HTMLElement>(null);
  const isBusy = status === "starting" || status === "stopping";
  // Remounting between sessions drops any half-armed discard, so a recording
  // never inherits an armed button from the one before it.
  const sessionKey = status === "idle" ? "idle" : "session";
  const isPaused = status === "paused";
  const isRecording = status === "recording";
  const { hours, minutes, seconds } = formatElapsedTime(elapsedMs);
  const confidenceDisabled = !isRecording;
  const hasConfidenceChecks =
    monitor?.hasCamera === true ||
    monitor?.hasSystemAudio === true ||
    monitor?.hasMicrophone === true;
  const isCameraPortrait =
    monitor?.cameraFrameSize != null &&
    monitor.cameraFrameSize.height > monitor.cameraFrameSize.width;

  useLayoutEffect(() => {
    const dock = dockRef.current;
    if (!dock || !onWidthChange) return;

    const reportWidth = () => {
      onWidthChange(Math.ceil(dock.getBoundingClientRect().width));
    };
    const observer = new ResizeObserver(reportWidth);
    observer.observe(dock);
    reportWidth();

    return () => {
      observer.disconnect();
    };
  }, [onWidthChange]);

  return (
    <main
      className="window-surface flex h-full w-max items-center gap-control-inset rounded-window p-control-inset text-content-fg windows:inset-ring windows:inset-ring-popover-stroke"
      data-tauri-drag-region="deep"
      onPointerUpCapture={onPointerUp}
      ref={dockRef}
    >
      {isBusy ? (
        <>
          <div
            aria-label={
              status === "starting"
                ? "Starting recording"
                : "Finishing recording"
            }
            className={cn("flex items-center justify-center", BUSY_SLOT_CLASS)}
            role="status"
          >
            {status === "starting" && countdownSeconds > 0 ? (
              <ContentRotate
                className="flex items-center justify-center text-title tabular-nums"
                contentKey={String(countdownSeconds)}
              >
                {countdownSeconds}
              </ContentRotate>
            ) : (
              <div className="flex items-center justify-center gap-control-inset text-body">
                <CircularProgress isIndeterminate size="small" />
                {/*
                 * Fixed-width label, so the centred row is a whole number of
                 * pixels wide and the spinner lands on a whole pixel. WebKit
                 * re-rasterises a rotating element sitting on a fractional
                 * pixel once per frame, and the snapping makes it wobble by
                 * about half a pixel; Chromium does not, which is why this
                 * only ever showed up in the app. The width also has to stay
                 * independent of the text, since measured glyph widths differ
                 * per engine.
                 */}
                <span className="w-14 text-center">
                  {status === "starting" ? "Starting" : "Finishing"}
                </span>
              </div>
            )}
          </div>

          <IconButton
            aria-label={
              status === "starting" && countdownSeconds > 0
                ? "Cancel recording countdown"
                : status === "starting"
                  ? "Cancel starting recording"
                  : "Cancel finishing recording"
            }
            onPress={onDiscard}
          >
            <X />
          </IconButton>
        </>
      ) : (
        <>
          {hasConfidenceChecks && (
            <div className="flex shrink-0 items-center gap-control">
              {monitor.hasCamera && (
                <div className="flex h-control-height items-center">
                  <CameraThumbnail
                    aria-label="Camera confidence preview"
                    canvasRef={monitor.cameraCanvasRef}
                    // A portrait camera is held to the row's height; a
                    // landscape one takes a 40px width, which 16:9 leaves
                    // shorter than the row.
                    className={isCameraPortrait ? "h-control-height" : "w-10"}
                    frameSize={monitor.cameraFrameSize}
                    hasFrame={monitor.hasCameraFrame}
                    isDimmed={confidenceDisabled}
                  />
                </div>
              )}
              {(monitor.hasSystemAudio || monitor.hasMicrophone) && (
                <div className="flex gap-tight">
                  {monitor.hasSystemAudio && (
                    <AudioMeter
                      decibels={monitor.systemAudioDecibels}
                      disabled={confidenceDisabled}
                      height={16}
                      hidePeakTick
                      hideTicks
                      orientation="vertical"
                      radius={1}
                      width={2}
                    />
                  )}
                  {monitor.hasMicrophone && (
                    <AudioMeter
                      decibels={monitor.microphoneDecibels}
                      disabled={confidenceDisabled}
                      height={16}
                      hidePeakTick
                      hideTicks
                      orientation="vertical"
                      radius={1}
                      width={2}
                    />
                  )}
                </div>
              )}
            </div>
          )}

          <div
            className={cn(
              "flex justify-center text-body tabular-nums",
              TIME_SLOT_CLASS,
            )}
          >
            <div
              className={cn(
                "flex transition-colors",
                isPaused && "text-content-fg-secondary",
              )}
            >
              <RotatingDigits value={hours} />:
              <RotatingDigits value={minutes} />:
              <RotatingDigits value={seconds} />
            </div>
          </div>

          <div className="flex items-center gap-control">
            <IconToggleButton
              aria-label={isPaused ? "Resume recording" : "Pause recording"}
              isSelected={isPaused}
              off={<Pause />}
              onChange={(selected) => {
                onPauseChange?.(selected);
              }}
            >
              <Play />
            </IconToggleButton>

            <IconButton
              aria-label="Stop recording"
              color="primary"
              onPress={onStop}
            >
              <Square />
            </IconButton>

            <DiscardButton key={sessionKey} onDiscard={onDiscard} />
          </div>
        </>
      )}
    </main>
  );
}
