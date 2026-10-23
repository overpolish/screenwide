// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createElement, useEffect, useState } from "react";

import { t } from "../../../i18n/i18n";
import { CameraThumbnail } from "../../recording-inputs/camera-thumbnail";

import { SamplePicture } from "./island-stage-picture";

import type {
  IslandInput,
  IslandInputKind,
  IslandReplay,
} from "./island-types";

// The stage's stand-ins for what the app would drive: the camera's picture,
// which follows Flip, and the replay buffer, off unless a story says
// otherwise, which turned on starts for a moment before it is on, as the
// real buffer does while it sets up its capture.

const STARTING_MS = 900;

/** The buffer the island is shown, none where a story says it cannot run,
 * and what its switch does to it. */
export function useStageReplay(initial: IslandReplay | null | undefined) {
  const [replay, setReplay] = useState<IslandReplay | null>(
    initial === undefined ? { status: "off" } : initial,
  );
  // A story that opens on the buffer starting holds it there to be looked
  // at, until the switch is used.
  const [holdsStart, setHoldsStart] = useState(initial?.status === "starting");
  useEffect(() => {
    if (replay?.status !== "starting" || holdsStart) return;
    const timer = window.setTimeout(() => {
      setReplay((current) => current && { ...current, status: "on" });
    }, STARTING_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [replay?.status, holdsStart]);

  const changeReplay = (isOn: boolean) => {
    setHoldsStart(false);
    setReplay(
      (current) => current && { ...current, status: isOn ? "starting" : "off" },
    );
  };
  return { changeReplay, replay: replay ?? undefined };
}

/** The island's camera props on the stage: its picture, and Flip, which the
 * picture follows. Loading, it is the app's own picture before the camera's
 * first frame: the preview starting, or, live, the recording's yet to come. */
export function useStageCamera(
  isFlippedAtFirst = false,
  isLoading = false,
  isLive = false,
) {
  const [isCameraFlipped, setIsCameraFlipped] = useState(isFlippedAtFirst);
  return {
    cameraPreview: isLoading
      ? createElement(CameraThumbnail, {
          canvasRef: null,
          // Filling the island's picture box, as `use-island-inputs.ts` does.
          className: "size-full rounded-none",
          frameSize: null,
          hasFrame: false,
          isStarting: !isLive,
        })
      : createElement(SamplePicture, { isFlipped: isCameraFlipped }),
    isCameraFlipped,
    onCameraFlipChange: setIsCameraFlipped,
  };
}

/** The stage's microphone, as a story names its problem: missing, as the
 * app finds before recording. The levels the inputs report follow, nothing
 * from an input that is off. */
export function stageMicrophone({
  inputs,
  problem,
}: {
  inputs: Record<IslandInputKind, IslandInput>;
  problem?: "missing";
}) {
  return {
    level: (kind: "microphone" | "systemAudio", value = 0) =>
      inputs[kind].isOn ? value : 0,
    warning:
      problem === "missing"
        ? t("recording-controls-microphone-missing")
        : inputs.microphone.warning,
  };
}
