// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, Check, Pause, Play, Square, Trash2, X } from "lucide-react";
import { useLayoutEffect, useRef } from "react";

import {
  IconButton,
  IconToggleButton,
} from "../../../components/base/button/icon-button";
import { CircularProgress } from "../../../components/base/circular-progress/circular-progress";
import { ContentRotate } from "../../../components/base/content-rotate/content-rotate";
import { Overlay } from "../../../components/base/overlay/overlay";
import { ConfirmActionButton } from "../../../components/shared/confirm-action-button/confirm-action-button";
import { cn } from "../../../lib/styling";
import { AudioMeter } from "../../audio-inputs/components/audio-meter";
import { cameraPreviewFitClassName } from "../../recording-inputs/camera-preview-fit";
import { formatElapsedTime } from "../elapsed-time";
import { RecordingStatus } from "../types";
import { RecordingMonitorSnapshot } from "../use-recording-monitor";

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
  isDisabled: boolean;
  onDiscard?: () => void;
};

/**
 * Two-step, because discarding sits one button away from stopping: the bin
 * swaps in place to a red check, and only pressing that check discards. The
 * swap is the pause button's, so the three controls stay of a piece.
 */
function DiscardButton({ isDisabled, onDiscard }: DiscardButtonProps) {
  return (
    <ConfirmActionButton
      armedClassName="bg-error-surface text-error data-[hovered]:bg-error-surface-hover data-[pressed]:bg-error-surface-pressed"
      armedIcon={<Check />}
      armedLabel="Confirm discarding"
      idleIcon={<Trash2 />}
      idleLabel="Discard recording"
      isDisabled={isDisabled}
      onConfirm={onDiscard}
      size="default"
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
      className="window-surface p-section gap-section relative flex h-full w-max cursor-grab items-center overflow-hidden rounded-window text-content-fg"
      data-tauri-drag-region="deep"
      onPointerUpCapture={onPointerUp}
      ref={dockRef}
    >
      <Overlay
        aria-label={
          status === "starting" ? "Starting recording" : "Finishing recording"
        }
        className="z-60 rounded-window text-content-fg"
        contained
        isOpen={isBusy}
      >
        {status === "starting" && countdownSeconds > 0 ? (
          <ContentRotate
            className="flex h-full items-center justify-center font-mono text-xl font-bold tabular-nums"
            containerClassName="absolute inset-0"
            contentKey={String(countdownSeconds)}
          >
            {countdownSeconds}
          </ContentRotate>
        ) : (
          <div className="gap-control-inset flex items-center justify-center text-sm">
            <CircularProgress isIndeterminate size="compact" />
            {/*
             * Fixed-width label, so the centred row is a whole number of pixels
             * wide and the spinner lands on a whole pixel. WebKit re-rasterises
             * a rotating element sitting on a fractional pixel once per frame,
             * and the snapping makes it wobble by about half a pixel; Chromium
             * does not, which is why this only ever showed up in the app. The
             * width also has to stay independent of the text, since measured
             * glyph widths differ per engine.
             */}
            <span className="w-14 text-center">
              {status === "starting" ? "Starting" : "Finishing"}
            </span>
          </div>
        )}
        <IconButton
          aria-label={
            status === "starting" && countdownSeconds > 0
              ? "Cancel recording countdown"
              : status === "starting"
                ? "Cancel starting recording"
                : "Cancel finishing recording"
          }
          className="right-section absolute top-1/2 -translate-y-1/2"
          onPress={onDiscard}
        >
          <X />
        </IconButton>
      </Overlay>
      {hasConfidenceChecks && (
        <div className="gap-control flex h-full shrink-0 items-center">
          {monitor.hasCamera && (
            <div className="shadow-preview relative flex aspect-video w-12 items-center justify-center overflow-hidden">
              <canvas
                aria-label="Camera confidence preview"
                className={cn(
                  "pointer-events-none block shrink-0 transition-opacity",
                  monitor.cameraFrameSize
                    ? cameraPreviewFitClassName(monitor.cameraFrameSize)
                    : "h-full w-auto max-w-full",
                  (!monitor.hasCameraFrame || confidenceDisabled) &&
                    "opacity-50",
                )}
                ref={monitor.cameraCanvasRef}
              />
              {!monitor.hasCameraFrame && (
                <Camera className="absolute size-icon-compact text-muted" />
              )}
            </div>
          )}
          {(monitor.hasSystemAudio || monitor.hasMicrophone) && (
            <div className="gap-tight flex">
              {monitor.hasSystemAudio && (
                <AudioMeter
                  decibels={monitor.systemAudioDecibels}
                  disabled={confidenceDisabled}
                  height={24}
                  hidePeakTick
                  hideTicks
                  orientation="vertical"
                  width={4}
                />
              )}
              {monitor.hasMicrophone && (
                <AudioMeter
                  decibels={monitor.microphoneDecibels}
                  disabled={confidenceDisabled}
                  height={24}
                  hidePeakTick
                  hideTicks
                  orientation="vertical"
                  width={4}
                />
              )}
            </div>
          )}
        </div>
      )}

      <div className="flex w-16 justify-center font-mono text-sm tabular-nums">
        <div className={cn("flex transition-colors", isPaused && "text-muted")}>
          <RotatingDigits value={hours} />:<RotatingDigits value={minutes} />:
          <RotatingDigits value={seconds} />
        </div>
      </div>

      <div className="gap-control flex items-center">
        <IconToggleButton
          aria-label={isPaused ? "Resume recording" : "Pause recording"}
          isDisabled={isBusy}
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
          isDisabled={isBusy}
          onPress={onStop}
        >
          <Square />
        </IconButton>

        <DiscardButton
          isDisabled={isBusy}
          key={sessionKey}
          onDiscard={onDiscard}
        />
      </div>
    </main>
  );
}
