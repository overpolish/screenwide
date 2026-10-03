// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

import { RecordingSceneClip } from "./recording-scenes";

/** The part of the screen's picture the composition shows, as shares of the
 * picture: what a scene's screen framing is measured against. */
export type VisibleScreenArea = {
  height: number;
  width: number;
  x: number;
  y: number;
};

/** Shorter than this, what is left of an auto zoom beside a scene of your
 * own would spend its whole life arriving and leaving. */
const AUTO_ZOOM_MINIMUM_MS = 1_500;

/** The auto zooms the recording's cursor and keys call for, planned by
 * `src-tauri/src/editor/auto_zoom.rs`. */
export const planRecordingAutoZoom = (
  artifactId: number,
  visible: VisibleScreenArea,
) =>
  invoke<RecordingSceneClip[]>("plan_recording_auto_zoom", {
    artifactId,
    visible,
  });

/** What part of a `source` sized picture `output` shows. A composition
 * without a measured placement shows the whole picture. */
export const visibleScreenArea = (
  output: ScreenshotOutputSettings,
  source: { height: number; width: number } | undefined,
): VisibleScreenArea => {
  if (
    !source ||
    output.imageWidth <= 0 ||
    output.cropWidth <= 0 ||
    output.cropHeight <= 0
  )
    return { height: 1, width: 1, x: 0, y: 0 };
  const imageHeight = (output.imageWidth * source.height) / source.width;
  return {
    height: output.cropHeight / imageHeight,
    width: output.cropWidth / output.imageWidth,
    x: (output.cropX - output.imageX) / output.imageWidth,
    y: (output.cropY - output.imageY) / imageHeight,
  };
};

/**
 * `next` with every auto zoom an edit has touched made your own, so making
 * the auto zooms again leaves it alone. A zoom counts as touched when it is
 * not exactly the clip it was before, which includes one a split or a copy
 * made from an auto zoom.
 */
export const ownEditedAutoZooms = (
  previous: readonly RecordingSceneClip[],
  next: RecordingSceneClip[],
): RecordingSceneClip[] => {
  const before = new Map(
    previous.map((clip) => [clip.id, JSON.stringify(clip)]),
  );
  return next.map((clip) => {
    if (!clip.auto || before.get(clip.id) === JSON.stringify(clip)) return clip;
    const { auto: _auto, ...own } = clip;
    return own;
  });
};

/**
 * `clips` with their auto zooms swapped for `planned`. Scenes of your own
 * stay as they are; a planned zoom that runs into one keeps the longest
 * stretch beside it, or is left out where that would be too short to play.
 */
export const withAutoZooms = (
  clips: readonly RecordingSceneClip[],
  planned: readonly RecordingSceneClip[],
) => {
  const own = clips.filter((clip) => !clip.auto);
  const fitted = planned.flatMap((zoom) => {
    let free = [{ endMs: zoom.endMs, startMs: zoom.startMs }];
    for (const scene of own)
      free = free.flatMap((stretch) =>
        [
          {
            endMs: Math.min(stretch.endMs, scene.startMs),
            startMs: stretch.startMs,
          },
          {
            endMs: stretch.endMs,
            startMs: Math.max(stretch.startMs, scene.endMs),
          },
        ].filter(({ endMs, startMs }) => endMs > startMs),
      );
    const longest = free.reduce<(typeof free)[number] | null>(
      (best, stretch) =>
        !best || stretch.endMs - stretch.startMs > best.endMs - best.startMs
          ? stretch
          : best,
      null,
    );
    return longest && longest.endMs - longest.startMs >= AUTO_ZOOM_MINIMUM_MS
      ? [{ ...zoom, ...longest }]
      : [];
  });
  return [...own, ...fitted].sort((a, b) => a.startMs - b.startMs);
};
