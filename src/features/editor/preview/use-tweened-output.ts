// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { useLayoutEffect, useRef, useState } from "react";

import { motionDurations } from "../../../lib/motion";
import { EditorKind } from "../types";

import { cameraOverlayBetween, outputBetween } from "./picture-between";
import {
  easeOut,
  PicturePart,
  pictureTweenAsked,
  takePictureTween,
} from "./picture-tween";

import type { CameraOverlaySettings } from "../../../bindings/CameraOverlaySettings";
import type { RecordingOutputSettings } from "../../../bindings/RecordingOutputSettings";
import type { ScreenshotWorkspaceOutputSettings } from "../../../bindings/ScreenshotWorkspaceOutputSettings";

/** Every layer and the canvas of a screenshot drawn between `from` and
 * `to`. A layer that is in only one of them is drawn as `to` has it. */
export const screenshotBetween = (
  from: ScreenshotWorkspaceOutputSettings,
  to: ScreenshotWorkspaceOutputSettings,
  share: number,
): ScreenshotWorkspaceOutputSettings => ({
  ...outputBetween(from, to, share),
  items: to.items.map((item) => {
    const was = from.items.find((candidate) => candidate.id === item.id);
    return was
      ? { ...item, output: outputBetween(was.output, item.output, share) }
      : item;
  }),
});

/** A recording's composition as the preview draws it: its screen and camera
 * settings, and where the camera is drawn into the screen's picture. */
export type RecordingComposition = {
  cameraOverlay: CameraOverlaySettings;
  recordingOutput: RecordingOutputSettings;
};

/** A recording's screen and camera drawn between `from` and `to`. */
export const recordingBetween = (
  from: RecordingComposition,
  to: RecordingComposition,
  share: number,
): RecordingComposition => ({
  cameraOverlay: cameraOverlayBetween(
    from.cameraOverlay,
    to.cameraOverlay,
    share,
  ),
  recordingOutput: {
    camera: outputBetween(
      from.recordingOutput.camera,
      to.recordingOutput.camera,
      share,
    ),
    primary: outputBetween(
      from.recordingOutput.primary,
      to.recordingOutput.primary,
      share,
    ),
  },
});

/** The settings as the picture draws them: `target` at once, or, where an
 * island asked for the change (`askPictureTween`), moving there from what
 * was drawn, through `between`. `part` names which of `workspace`'s
 * pictures `target` is, each taking the ask for itself.
 *
 * The render that brings an asked change still draws what was drawn: the
 * native preview is handed the drawn settings from passive effects, which
 * run for that render before the tween's first step re-renders, so drawing
 * `target` there showed the new look for a frame before the tween started
 * from the old one. Outside a tween `target` is returned as it is, with no
 * state of its own: a caller that builds `target` afresh each render (a
 * default output) would otherwise set state from every render and never
 * settle. */
export function useTweenedOutput<T>(
  target: T,
  { part, workspace }: { part: PicturePart; workspace: EditorKind },
  between: (from: NonNullable<T>, to: NonNullable<T>, share: number) => T,
) {
  // Boxed, so a tween's value is never mistaken for "no tween".
  const [tweened, setTweened] = useState<{ value: T } | null>(null);
  // What was last drawn, and the target the effect last moved towards.
  const shownRef = useRef(target);
  const targetRef = useRef(target);
  const betweenRef = useRef(between);
  betweenRef.current = between;
  // Asked for with reduced motion, a change is drawn whole on its first step.
  const reducedMotion = useReducedMotion() ?? false;
  useLayoutEffect(() => {
    targetRef.current = target;
    const from = shownRef.current;
    if (target == null || from == null || !takePictureTween(workspace, part)) {
      shownRef.current = target;
      // Drawn in the same frame, before paint: a drag is never behind.
      setTweened(null);
      return;
    }
    const start = performance.now();
    const duration = reducedMotion ? 0 : motionDurations.reshape * 1000;
    let frame = 0;
    const step = () => {
      const elapsed = performance.now() - start;
      const progress = duration > 0 ? Math.min(1, elapsed / duration) : 1;
      const value =
        progress >= 1
          ? target
          : betweenRef.current(from, target, easeOut(progress));
      shownRef.current = value;
      // The first step runs in the layout effect, so it is drawn before paint.
      // eslint-disable-next-line @eslint-react/set-state-in-effect
      setTweened(progress >= 1 ? null : { value });
      if (progress < 1) frame = requestAnimationFrame(step);
    };
    step();
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [part, reducedMotion, target, workspace]);
  if (target !== targetRef.current)
    return target != null &&
      shownRef.current != null &&
      pictureTweenAsked(workspace, part)
      ? shownRef.current
      : target;
  return tweened ? tweened.value : target;
}
