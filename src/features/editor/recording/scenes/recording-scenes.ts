// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineSourceToOutput,
} from "../../timeline/editing/recording-timeline-edit";

import type { SceneVariant } from "./recording-scene-variant";

/** The twin of `RecordingScenePreset` in `src-tauri/src/editor/scenes/model.rs`.
 * `full` keeps the screen and the camera where the recording's own
 * composition puts them. It is never offered: it is what a custom scene made
 * from the recording's own composition starts from and resets to. */
export type RecordingScenePreset =
  | "camera-only"
  | "full"
  | "picture-in-picture"
  | "screen-only"
  | "split-two-thirds"
  | "stacked";

/** The presets the Scene panel offers beside Custom. */
export type ArrangedScenePreset = Exclude<RecordingScenePreset, "full">;

/**
 * What part of a picture fills its box: the point of the picture, as shares
 * of its width and height, that sits in the box's middle, and how far it is
 * zoomed past only just covering the box. The twin of `SceneFraming` in
 * `src-tauri/src/editor/scenes/framing.rs`.
 */
export type SceneFraming = { focusX: number; focusY: number; zoom: number };

/** The whole picture, centred: what a scene shows until it is reframed. */
export const WHOLE_FRAMING: SceneFraming = {
  focusX: 0.5,
  focusY: 0.5,
  zoom: 1,
};

/** The furthest a scene zooms into a picture. */
export const MAX_SCENE_ZOOM = 8;

/**
 * One scene over a stretch of the recording, in source time like an
 * annotation clip: an arrangement of the screen and the camera, and what part
 * of each fills its box. A recording's clips are kept sorted by start and
 * never overlap: only one scene can be on screen at a time. Bounds are half
 * open, so one may end where the next begins.
 */
export type RecordingSceneClip = {
  endMs: number;
  id: string;
  preset: RecordingScenePreset;
  startMs: number;
  /** Set, a custom scene: the panes sit in these boxes instead of the
   * preset's, which a reset returns them to. */
  boxes?: SceneBoxes;
  /** Unset, a full scene keeps the recording's own camera crop and an
   * arrangement shows the whole camera. */
  camera?: SceneFraming;
  /** The panes' corner radii in this scene, which a reset takes away. */
  radius?: SceneRadius;
  /** Unset, the whole screen. */
  screen?: SceneFraming;
  /** How the preset is laid out, each option its default where unset. */
  variant?: SceneVariant;
};

/** One pane's box in a custom scene, as shares of the canvas's width and
 * height. */
export type SceneBox = { height: number; width: number; x: number; y: number };

/** Where a custom scene puts its panes. The camera keeps the recording's own
 * box where it has none. The twin of `SceneBoxes` in
 * `src-tauri/src/editor/scenes/model.rs`. */
export type SceneBoxes = { screen: SceneBox; camera?: SceneBox };

/** The corner radius a scene gives each pane, as a share of the pane's
 * shorter side in percent, like the recording's own. A pane without one
 * keeps the recording's radius. The twin of `SceneRadius` in
 * `src-tauri/src/editor/scenes/model.rs`. */
export type SceneRadius = { camera?: number; screen?: number };

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
const RECORDING_SCENE_MINIMUM_MS = 500;

/** The clip playing at `sourceMs`, or null between clips. */
export const recordingSceneClipAt = (
  clips: readonly RecordingSceneClip[],
  sourceMs: number,
) =>
  clips.find((clip) => clip.startMs <= sourceMs && sourceMs < clip.endMs) ??
  null;

/** Where the clip `id` may reach without running into its neighbours, in
 * source milliseconds. */
const roomAround = (
  clips: readonly RecordingSceneClip[],
  id: string,
  sourceDurationMs: number,
) => {
  const index = clips.findIndex((clip) => clip.id === id);
  return {
    clip: index < 0 ? null : clips[index],
    from: clips[index - 1]?.endMs ?? 0,
    to: clips[index + 1]?.startMs ?? sourceDurationMs,
  };
};

/**
 * A new clip at `atMs`, as long as the default where the free stretch around
 * the playhead allows: it runs forward first and, near the end of the
 * recording or the next clip, reaches back for the rest. Null where the
 * playhead is inside a clip or the stretch is too short to hold one.
 */
export const insertRecordingSceneClip = ({
  atMs,
  clips,
  id,
  preset,
  sourceDurationMs,
}: {
  atMs: number;
  clips: readonly RecordingSceneClip[];
  id: string;
  preset: RecordingScenePreset;
  sourceDurationMs: number;
}) => {
  if (recordingSceneClipAt(clips, atMs)) return null;
  const at = Math.max(0, Math.min(sourceDurationMs, Math.round(atMs)));
  const from = Math.max(
    0,
    ...clips.flatMap((clip) => (clip.endMs <= at ? [clip.endMs] : [])),
  );
  const after = clips.find((clip) => clip.startMs >= at);
  const end = Math.min(
    at + RECORDING_SCENE_DEFAULT_MS,
    after?.startMs ?? sourceDurationMs,
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
 * The clip `id` slid `deltaOutput` along the output timeline. The other clips
 * stay where they are and the moved one passes over them: it lands in the
 * free stretch nearest to where its middle was carried, keeping how long it
 * plays for, and where that stretch is shorter it fills the stretch instead.
 * A stretch too short to hold a scene at all is passed over.
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
  if (!clip || sourceDurationMs <= 0) return [...clips];
  const output = (ms: number) =>
    recordingTimelineSourceToOutput(edit, ms / sourceDurationMs);
  const source = (position: number) =>
    Math.round(
      recordingTimelineOutputToSource(edit, position) * sourceDurationMs,
    );
  const others = clips.filter((item) => item.id !== id);
  const length = output(clip.endMs) - output(clip.startMs);
  const middle = Math.max(
    length / 2,
    Math.min(
      1 - length / 2,
      (output(clip.startMs) + output(clip.endMs)) / 2 + deltaOutput,
    ),
  );
  // The free stretch nearest the carried middle, measured on the output
  // timeline; the first of two equally near wins.
  let gap: { distance: number; from: number; to: number } | null = null;
  let from = 0;
  for (const [start, end] of [
    ...others.map((item) => [item.startMs, item.endMs]),
    [sourceDurationMs, sourceDurationMs],
  ]) {
    if (start - from >= RECORDING_SCENE_MINIMUM_MS) {
      const low = output(from);
      const high = output(start);
      const distance = Math.max(0, low - middle, middle - high);
      if (!gap || distance < gap.distance) gap = { distance, from, to: start };
    }
    from = end;
  }
  if (!gap) return [...clips];
  const low = output(gap.from);
  const high = output(gap.to);
  let placed = { endMs: gap.to, startMs: gap.from };
  if (high - low > length) {
    const start = Math.max(low, Math.min(high - length, middle - length / 2));
    const startMs = Math.max(gap.from, source(start));
    placed = {
      endMs: Math.min(gap.to, Math.max(startMs + 1, source(start + length))),
      startMs,
    };
  }
  return [...others, { ...clip, ...placed }].sort(
    (a, b) => a.startMs - b.startMs,
  );
};

/**
 * The clip `id` with one edge taken to the output position `output`, held
 * between its neighbour and the shortest a scene may be.
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
  const { clip, from, to } = roomAround(clips, id, sourceDurationMs);
  if (!clip) return [...clips];
  const source = Math.round(
    recordingTimelineOutputToSource(edit, Math.max(0, Math.min(1, output))) *
      sourceDurationMs,
  );
  const minimum = Math.min(
    RECORDING_SCENE_MINIMUM_MS,
    clip.endMs - clip.startMs,
  );
  const next =
    edge === "startMs"
      ? { startMs: Math.max(from, Math.min(source, clip.endMs - minimum)) }
      : { endMs: Math.min(to, Math.max(source, clip.startMs + minimum)) };
  return clips.map((item) => (item.id === id ? { ...item, ...next } : item));
};
