// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type SceneDrawing,
  type ScenePreset,
} from "../../glyphs/scene-layout-drawing";
import { presetName } from "../../island/scene/scene-names";
import { sceneLandingMs } from "../../recording/scenes/recording-scene-arrangement";
import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import { type TimelineScene } from "../scene-rail/scene-clip";
import { type Playhead } from "../scrub-playhead";
import { baseOf, partId } from "../strip/strip-part";
import {
  isPlayheadOutside,
  type StripSelection,
} from "../strip/strip-selection";
import { SLIVER_S } from "../strip/strip-split";
import { type Span, type TimeMap } from "../timeline-scale";

import { type RecordingStripSteps } from "./use-recording-strip-edits";

import type { RecordingStripLanes } from "./use-recording-strip";
import type { RecordingAnnotationClip } from "../../../../bindings/RecordingAnnotationClip";
import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";

// The strip's lanes over the recording's own annotations and scenes, which
// the edit times in the edited recording's milliseconds and the strip in
// recorded seconds. Retimed, restacked or drawn on the strip, they change
// through the hooks that own them, as their lanes did; chosen, an
// annotation takes up Select, whose island follows it to its pen, and a
// scene takes up the Scene tool, whose island is its mode.

/** Shorter than this, in ms, a scene would spend its whole life arriving. */
const LEAST_SCENE_MS = 500;
/** Shorter than this, in ms, a drag on the scene rail draws nothing. */
const LEAST_DRAWN_MS = 1000;

/** A stretch's span in recorded seconds, from its span in played seconds,
 * inside one run. */
function recordedSpan(map: TimeMap, from: number, to: number) {
  const start = map.toRecorded(from);
  // Its end is the last moment it shows, which is in what plays before it.
  const end = map.toRecorded(Math.max(to - 1e-4, from)) + 1e-4;
  return { end: Math.max(end, start), start };
}

/** A clip's parts in recorded seconds, from its span in edited ms: one for
 * each run it plays across, so a clip played across a jump is drawn each
 * side of it. One span from its start to its end would take in what plays
 * elsewhere between them (a stretch moved away from inside it, drawn where
 * it plays now), or run backwards (one moved into it). */
function recordedParts(
  map: TimeMap,
  runs: Span[],
  { endMs, startMs }: { endMs: number; startMs: number },
) {
  return runs.flatMap((run) => {
    const from = Math.max(startMs / 1000, run.start);
    const to = Math.min(endMs / 1000, run.end);
    return to - from >= SLIVER_S ? [recordedSpan(map, from, to)] : [];
  });
}

/** Where a recorded moment plays, in edited ms. */
const playedMs = (map: TimeMap, seconds: number, isEnd?: boolean) =>
  Math.round(map.toPlayed(seconds, isEnd) * 1000);

export type RecordingStripLaneSource = {
  annotations: {
    clearSelection: () => void;
    clips: RecordingAnnotationClip[];
    onClipsChange: (clips: RecordingAnnotationClip[]) => void;
    onSelect: (id: string, toggle: boolean) => void;
    selectedIds: ReadonlySet<string>;
  };
  /** Takes up a tool, as choosing an annotation or a scene does. */
  changeCanvasTool: (tool: "scene" | "select") => void;
  map: TimeMap;
  /** Opens the Captions mode. */
  openCaptions: () => void;
  /** Where the playhead stands, a share of what plays. */
  playhead: Playhead;
  /** What plays on without a jump, in played seconds (`RecordingStripTime`). */
  runs: Span[];
  scenes: {
    clips: RecordingSceneClip[];
    /** How a scene's layout is drawn, as the Scene island draws it. */
    drawingOf: (clip: RecordingSceneClip) => SceneDrawing;
    isAvailable: boolean;
    onClipsChange: (clips: RecordingSceneClip[]) => void;
    selection: {
      ids: ReadonlySet<string>;
      onClear: () => void;
      onSelect: (id: string, toggle: boolean) => void;
    };
  };
  /** Brings the playhead to a moment, in recorded seconds. */
  seek: (seconds: number) => void;
  steps: RecordingStripSteps;
};

/** The room a scene has between its neighbours, leaving out `except`. */
function roomAround(
  clips: RecordingSceneClip[],
  atMs: number,
  except?: string,
) {
  const others = clips.filter(({ id }) => id !== except);
  return {
    from: Math.max(
      0,
      ...others.filter(({ endMs }) => endMs <= atMs).map(({ endMs }) => endMs),
    ),
    to: Math.min(
      Number.POSITIVE_INFINITY,
      ...others
        .filter(({ startMs }) => startMs >= atMs)
        .map(({ startMs }) => startMs),
    ),
  };
}

export function recordingStripLanes(source: RecordingStripLaneSource) {
  const { annotations, map, runs, scenes, steps } = source;
  const partsOf = (clip: { endMs: number; startMs: number }) =>
    recordedParts(map, runs, clip);
  /** `clip`'s times once its part `id` is pulled to `start` and `end`.
   * Only the clip's own ends move, the first part's start and the last
   * part's end; where a part meets a jump it ends with the run, not by
   * choice. */
  const retimed = (
    clip: { endMs: number; startMs: number },
    id: string,
    { end, start }: Span,
  ) => {
    const parts = partsOf(clip);
    const index = parts.findIndex((_, each) => partId(baseOf(id), each) === id);
    if (index < 0) return clip;
    const part = parts[index];
    return {
      endMs:
        index === parts.length - 1 && end !== part.end
          ? playedMs(map, end, true)
          : clip.endMs,
      startMs:
        index === 0 && start !== part.start
          ? playedMs(map, start)
          : clip.startMs,
    };
  };
  // Later clips are drawn in front, on the top lane.
  const laneOfClip = (index: number) => annotations.clips.length - 1 - index;
  const timelineAnnotations: TimelineAnnotation[] = annotations.clips.flatMap(
    (clip, index) =>
      partsOf(clip).map((span, part) => ({
        ...span,
        id: partId(clip.annotation.id, part),
        kind: clip.annotation.shape.kind,
        lane: laneOfClip(index),
      })),
  );
  const timelineScenes: TimelineScene[] = scenes.clips.flatMap((clip) => {
    const preset: ScenePreset =
      clip.preset === "full" ? "screen-only" : clip.preset;
    const landsAt = map.toRecorded(sceneLandingMs(clip) / 1000);
    return partsOf(clip).map((span, part) => ({
      ...span,
      drawing: scenes.drawingOf(clip),
      id: partId(clip.id, part),
      landsAt,
      name: presetName(preset),
    }));
  });
  const sortedScenes = (clips: RecordingSceneClip[]) =>
    [...clips].sort((one, other) => one.startMs - other.startMs);

  const lanes: RecordingStripLanes = {
    annotations: timelineAnnotations,
    onAnnotationRestack: (laneOf) => {
      // A clip's parts share its lane; the one moved, if any, says where.
      const laneOfBase = new Map(
        annotations.clips.map((clip, index) => [
          clip.annotation.id,
          laneOfClip(index),
        ]),
      );
      for (const { id, lane: was } of timelineAnnotations) {
        if (id in laneOf && laneOf[id] !== was)
          laneOfBase.set(baseOf(id), laneOf[id]);
      }
      const order = [...annotations.clips].sort(
        (one, other) =>
          (laneOfBase.get(other.annotation.id) ?? 0) -
          (laneOfBase.get(one.annotation.id) ?? 0),
      );
      steps.step(() => {
        annotations.onClipsChange(order);
      });
    },
    onAnnotationRetime: (id, start, end) => {
      const next = annotations.clips.map((clip) =>
        clip.annotation.id === baseOf(id)
          ? {
              ...clip,
              ...retimed(clip, id, { end, start }),
            }
          : clip,
      );
      steps.step(() => {
        annotations.onClipsChange(next);
      });
    },
    onCaptionPress: source.openCaptions,
    onSceneDraw: (from, to) => {
      if (!scenes.isAvailable) return;
      const fromMs = playedMs(map, Math.min(from, to));
      const toMs = playedMs(map, Math.max(from, to), true);
      if (toMs - fromMs < LEAST_DRAWN_MS) return;
      const room = roomAround(scenes.clips, fromMs);
      const clip: RecordingSceneClip = {
        endMs: Math.min(toMs, room.to),
        id: crypto.randomUUID(),
        preset: "picture-in-picture",
        startMs: Math.max(fromMs, room.from),
      };
      if (clip.endMs - clip.startMs < LEAST_SCENE_MS) return;
      steps.step(() => {
        scenes.onClipsChange(sortedScenes([...scenes.clips, clip]));
      });
      scenes.selection.onSelect(clip.id, false);
      source.changeCanvasTool("scene");
      // Drawn, the scene is chosen as one pressed is, so the playhead comes
      // to it, once arrived, where it stands outside it: the picture and the
      // Scene island show the scene under the playhead, which would
      // otherwise be another or none, and a layout chosen there would add a
      // scene.
      const parts = partsOf(clip);
      const at = map.toRecorded(source.playhead.ratio() * map.duration);
      if (parts.every((span) => isPlayheadOutside(at, span)))
        source.seek(map.toRecorded(sceneLandingMs(clip) / 1000));
    },
    onSceneRetime: (id, start, end) => {
      const own = scenes.clips.find((clip) => clip.id === baseOf(id));
      if (!own) return;
      const room = roomAround(
        scenes.clips,
        (own.startMs + own.endMs) / 2,
        own.id,
      );
      const pulled = retimed(own, id, { end, start });
      const startMs = Math.max(pulled.startMs, room.from);
      const endMs = Math.max(
        Math.min(pulled.endMs, room.to),
        startMs + LEAST_SCENE_MS,
      );
      steps.step(() => {
        scenes.onClipsChange(
          scenes.clips.map((clip) =>
            clip.id === own.id ? { ...clip, endMs, startMs } : clip,
          ),
        );
      });
    },
    scenes: timelineScenes,
  };

  /** What the lanes hold chosen, as the strip names it. */
  const chosen = (): StripSelection | null => {
    const [annotation] = annotations.selectedIds;
    if (annotations.selectedIds.size === 1)
      return { id: annotation, kind: "annotation" };
    const [scene] = scenes.selection.ids;
    if (scenes.selection.ids.size === 1) return { id: scene, kind: "scene" };
    return null;
  };
  /** Chooses in the lanes what the strip chose there, letting go of the
   * rest; true where the choice is the lanes' own. */
  const choose = (selection: StripSelection | null) => {
    if (selection?.kind === "annotation") {
      scenes.selection.onClear();
      annotations.onSelect(baseOf(selection.id), false);
      source.changeCanvasTool("select");
      return true;
    }
    if (selection?.kind === "scene") {
      annotations.clearSelection();
      scenes.selection.onSelect(baseOf(selection.id), false);
      source.changeCanvasTool("scene");
      return true;
    }
    annotations.clearSelection();
    scenes.selection.onClear();
    return false;
  };
  return { choose, chosen, lanes };
}
