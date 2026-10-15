// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { withRotation } from "../../layer-rotation";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { recordingTimelineLengthMs } from "../../timeline/editing/recording-timeline-time";

import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";
import type { RecordingScenePreset } from "../../../../bindings/RecordingScenePreset";
import type { SceneBox } from "../../../../bindings/SceneBox";
import type { SceneBoxes } from "../../../../bindings/SceneBoxes";
import type { SceneFraming } from "../../../../bindings/SceneFraming";
import type { SceneRotation } from "../../../../bindings/SceneRotation";

export type {
  RecordingSceneClip,
  RecordingScenePreset,
  SceneBox,
  SceneBoxes,
  SceneFraming,
  SceneRotation,
};

/** The presets the Scene panel offers beside Custom. */
export type ArrangedScenePreset = Exclude<RecordingScenePreset, "full">;

/** The whole picture, centred: what a scene shows until it is reframed. */
export const WHOLE_FRAMING: SceneFraming = {
  focusX: 0.5,
  focusY: 0.5,
  zoom: 1,
};

/** The furthest a scene zooms into a picture. */
export const MAX_SCENE_ZOOM = 8;

/** `clip` turning the screen by `rotation`, each angle held to what a scene
 * can turn it; one that leaves it upright keeps no turn at all. */
export const rotatedSceneClip = (
  clip: RecordingSceneClip,
  rotation: SceneRotation,
): RecordingSceneClip => withRotation(clip, rotation);

/** Whether a scene places a camera, which it can only do where the camera is
 * drawn into the picture. A custom scene places one where it has a box for
 * it. */
export const sceneNeedsCamera = ({
  boxes,
  preset,
}: {
  preset: RecordingScenePreset;
  boxes?: SceneBoxes;
}) =>
  boxes
    ? boxes.camera !== undefined
    : preset !== "full" && preset !== "screen-only";

/** How long a scene added at the playhead lasts, where there is room. */
const RECORDING_SCENE_DEFAULT_MS = 5000;
/** Shorter than this, a scene would spend its whole life arriving. */
export const RECORDING_SCENE_MINIMUM_MS = 500;

/** The clip playing `timelineMs` into the edited video, or null between
 * clips. */
export const recordingSceneClipAt = (
  clips: readonly RecordingSceneClip[],
  timelineMs: number,
) =>
  clips.find((clip) => clip.startMs <= timelineMs && timelineMs < clip.endMs) ??
  null;

/** Where the clip `id` may reach without running into its neighbours, on a
 * timeline `lengthMs` long. */
const roomAround = (
  clips: readonly RecordingSceneClip[],
  id: string,
  lengthMs: number,
) => {
  const index = clips.findIndex((clip) => clip.id === id);
  return {
    clip: index < 0 ? null : clips[index],
    from: clips[index - 1]?.endMs ?? 0,
    to: clips[index + 1]?.startMs ?? lengthMs,
  };
};

/**
 * A new clip at `atMs` on a timeline `lengthMs` long, as long as the default
 * where the free stretch around the playhead allows: it runs forward first
 * and, near the end of the video or the next clip, reaches back for the
 * rest. Null where the playhead is inside a clip or the stretch is too short
 * to hold one.
 */
export const insertRecordingSceneClip = ({
  atMs,
  clips,
  id,
  lengthMs,
  preset,
}: {
  atMs: number;
  clips: readonly RecordingSceneClip[];
  id: string;
  lengthMs: number;
  preset: RecordingScenePreset;
}) => {
  if (recordingSceneClipAt(clips, atMs)) return null;
  const length = Math.round(lengthMs);
  const at = Math.max(0, Math.min(length, Math.round(atMs)));
  const from = Math.max(
    0,
    ...clips.flatMap((clip) => (clip.endMs <= at ? [clip.endMs] : [])),
  );
  const after = clips.find((clip) => clip.startMs >= at);
  const end = Math.min(
    at + RECORDING_SCENE_DEFAULT_MS,
    after?.startMs ?? length,
  );
  const start = Math.max(from, Math.min(at, end - RECORDING_SCENE_DEFAULT_MS));
  if (end - start < RECORDING_SCENE_MINIMUM_MS) return null;
  const clip: RecordingSceneClip = { endMs: end, id, preset, startMs: start };
  return {
    clip,
    clips: [...clips, clip].sort((a, b) => a.startMs - b.startMs),
  };
};

/**
 * The clip `id` slid `deltaOutput`, a share of the timeline, along it. The
 * other clips stay where they are and the moved one passes over them: it
 * lands in the free stretch nearest to where its middle was carried, keeping
 * how long it plays for, and where that stretch is shorter it fills the
 * stretch instead. A stretch too short to hold a scene at all is passed over.
 */
export const moveRecordingSceneClip = ({
  clips,
  deltaOutput,
  edit,
  id,
  sourceDurationMs,
}: {
  clips: readonly RecordingSceneClip[];
  deltaOutput: number;
  edit: RecordingTimelineEdit;
  id: string;
  sourceDurationMs: number;
}): RecordingSceneClip[] => {
  const clip = clips.find((item) => item.id === id);
  const lengthMs = Math.round(
    recordingTimelineLengthMs(edit, sourceDurationMs),
  );
  if (!clip || lengthMs <= 0) return [...clips];
  const others = clips.filter((item) => item.id !== id);
  const length = clip.endMs - clip.startMs;
  const middle = Math.max(
    length / 2,
    Math.min(
      lengthMs - length / 2,
      (clip.startMs + clip.endMs) / 2 + deltaOutput * lengthMs,
    ),
  );
  // The free stretch nearest the carried middle; the first of two equally
  // near wins.
  let gap: { distance: number; from: number; to: number } | null = null;
  let from = 0;
  for (const [start, end] of [
    ...others.map((item) => [item.startMs, item.endMs]),
    [lengthMs, lengthMs],
  ]) {
    if (start - from >= RECORDING_SCENE_MINIMUM_MS) {
      const distance = Math.max(0, from - middle, middle - start);
      if (!gap || distance < gap.distance) gap = { distance, from, to: start };
    }
    from = end;
  }
  if (!gap) return [...clips];
  let placed = { endMs: gap.to, startMs: gap.from };
  if (gap.to - gap.from > length) {
    const startMs = Math.round(
      Math.max(gap.from, Math.min(gap.to - length, middle - length / 2)),
    );
    placed = { endMs: startMs + length, startMs };
  }
  return [...others, { ...clip, ...placed }].sort(
    (a, b) => a.startMs - b.startMs,
  );
};

/**
 * The clip `id` with one edge taken to `output`, a share of the timeline,
 * held between its neighbour and the shortest a scene may be.
 */
export const resizeRecordingSceneClip = ({
  clips,
  edge,
  edit,
  id,
  output,
  sourceDurationMs,
}: {
  clips: readonly RecordingSceneClip[];
  edge: "endMs" | "startMs";
  edit: RecordingTimelineEdit;
  id: string;
  output: number;
  sourceDurationMs: number;
}): RecordingSceneClip[] => {
  const lengthMs = Math.round(
    recordingTimelineLengthMs(edit, sourceDurationMs),
  );
  const { clip, from, to } = roomAround(clips, id, lengthMs);
  if (!clip) return [...clips];
  const at = Math.round(Math.max(0, Math.min(1, output)) * lengthMs);
  const minimum = Math.min(
    RECORDING_SCENE_MINIMUM_MS,
    clip.endMs - clip.startMs,
  );
  const next =
    edge === "startMs"
      ? { startMs: Math.max(from, Math.min(at, clip.endMs - minimum)) }
      : { endMs: Math.min(to, Math.max(at, clip.startMs + minimum)) };
  return clips.map((item) => (item.id === id ? { ...item, ...next } : item));
};
