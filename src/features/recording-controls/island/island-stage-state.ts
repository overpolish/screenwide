// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createElement, useEffect, useState } from "react";

import { SamplePicture } from "./island-stage-picture";

import type { IslandReplay } from "./island-types";

// The stage's stand-ins for what the app would drive: the camera's picture,
// which follows Flip, and the replay buffer, off unless a story says
// otherwise, which turned on starts for a moment before it is on, as the
// real buffer does while it sets up its capture.

const STARTING_MS = 900;

/** The buffer the island is shown, none where a story says it cannot run,
 * and what its switch does to it. */
export function useStageReplay(initial: IslandReplay | null | undefined) {
  const [replay, setReplay] = useState<IslandReplay | null>(
    initial === undefined ? { lengthSeconds: 30, status: "off" } : initial,
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
 * picture follows. */
export function useStageCamera(isFlippedAtFirst = false) {
  const [isCameraFlipped, setIsCameraFlipped] = useState(isFlippedAtFirst);
  return {
    cameraPreview: createElement(SamplePicture, { isFlipped: isCameraFlipped }),
    isCameraFlipped,
    onCameraFlipChange: setIsCameraFlipped,
  };
}
