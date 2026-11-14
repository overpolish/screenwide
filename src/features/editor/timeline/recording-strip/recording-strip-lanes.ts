// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type SceneDrawing,
  type ScenePreset,
} from "../../glyphs/scene-layout-drawing";
import { presetName } from "../../island/scene/scene-names";
import { sceneLandingMs } from "../../recording/scenes/recording-scene-arrangement";
import { LEAST_SCENE_MS } from "../../recording/scenes/recording-scene-gaps";
import { sceneCustomLabel } from "../../recording/scenes/recording-scene-labels";
import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import { type TimelineScene } from "../scene-rail/scene-clip";
import { drawnSceneSpan, LEAST_DRAWN_S } from "../scene-rail/scene-draw";
import { type Playhead } from "../scrub-playhead";
import { baseOf } from "../strip/strip-part";
import {
  isPlayheadOutside,
  type StripSelection,
} from "../strip/strip-selection";
import { type Span, type TimeMap } from "../timeline-scale";

import { type RecordingStripSteps } from "./use-recording-strip-edits";

import type { RecordingStripLanes } from "./use-recording-strip";
import type { RecordingAnnotationClip } from "../../../../bindings/RecordingAnnotationClip";
import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";

// The strip's lanes over the recording's own annotations and scenes, which
// the edit times in the edited recording's milliseconds and the strip lays
// out where they play, in played seconds (`strip-lane-time.ts`), each one
// piece wherever it runs. Retimed, restacked or drawn on the strip, they
// change through the hooks that own them, as their lanes did; chosen, an
// annotation takes up Select, whose island follows it to its pen, and a
// scene takes up the Scene tool, whose island is its mode.

type TimedSpan = { endMs: number; startMs: number };

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
  /** The gaps on the timeline, in ms, which a scene stops at the edges of
   * as at another scene's. */
  gaps: readonly TimedSpan[];
  /** How the recording plays, which brings the playhead to a moment. */
  map: TimeMap;
  /** Opens the Captions mode. */
  openCaptions: () => void;
  /** Where the playhead stands, a share of what plays. */
  playhead: Playhead;
  scenes: {
    /** Whether a scene can place the camera: a drawn scene starts as a
     * picture in picture only where it can, else as the screen alone. */
    canPlaceCamera: boolean;
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

/** The room a scene has between its neighbours and the gaps, leaving out
 * `except`. */
function roomAround(
  obstacles: readonly (TimedSpan & { id?: string })[],
  atMs: number,
  except?: string,
) {
  const others = obstacles.filter(
    ({ id }) => except === undefined || id !== except,
  );
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
  const { annotations, map, scenes, steps } = source;
  const obstacles = [...scenes.clips, ...source.gaps];
  /** A clip where it plays, in played seconds: one piece, as it plays. */
  const playedOf = (clip: { endMs: number; startMs: number }) => ({
    end: clip.endMs / 1000,
    start: clip.startMs / 1000,
  });
  /** A span the strip tells, in played seconds, in edited ms. */
  const timedMs = ({ end, start }: Span) => ({
    endMs: Math.round(end * 1000),
    startMs: Math.round(start * 1000),
  });
  // Later clips are drawn in front, on the top lane.
  const laneOfClip = (index: number) => annotations.clips.length - 1 - index;
  const timelineAnnotations: TimelineAnnotation[] = annotations.clips.map(
    (clip, index) => ({
      ...playedOf(clip),
      id: clip.annotation.id,
      kind: clip.annotation.shape.kind,
      lane: laneOfClip(index),
    }),
  );
  const timelineScenes: TimelineScene[] = scenes.clips.map((clip) => {
    const preset: ScenePreset =
      clip.preset === "full" ? "screen-only" : clip.preset;
    return {
      ...playedOf(clip),
      drawing: scenes.drawingOf(clip),
      id: clip.id,
      landsAt: sceneLandingMs(clip) / 1000,
      name: clip.boxes ? sceneCustomLabel() : presetName(preset),
    };
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
        if (id in laneOf && laneOf[id] !== was) laneOfBase.set(id, laneOf[id]);
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
          ? { ...clip, ...timedMs({ end, start }) }
          : clip,
      );
      steps.step(() => {
        annotations.onClipsChange(next);
      });
    },
    onCaptionPress: source.openCaptions,
    onSceneDraw: (from, to) => {
      if (!scenes.isAvailable) return;
      // In the order drawn, so a drag too short grows from where it was
      // pressed, as the rail showed it, rather than vanishing.
      const fromMs = from * 1000;
      const toMs = to * 1000;
      const room = roomAround(obstacles, fromMs);
      const span = drawnSceneSpan(
        { from: fromMs, to: toMs },
        { from: room.from, to: Math.min(room.to, map.duration * 1000) },
        LEAST_DRAWN_S * 1000,
      );
      const clip: RecordingSceneClip = {
        endMs: Math.round(span.end),
        id: crypto.randomUUID(),
        preset: scenes.canPlaceCamera ? "picture-in-picture" : "screen-only",
        startMs: Math.round(span.start),
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
      const played = source.playhead.ratio() * map.duration;
      if (isPlayheadOutside(played, playedOf(clip)))
        source.seek(map.toRecorded(sceneLandingMs(clip) / 1000));
    },
    onSceneRetime: (id, start, end) => {
      const own = scenes.clips.find((clip) => clip.id === baseOf(id));
      if (!own) return;
      const room = roomAround(obstacles, (own.startMs + own.endMs) / 2, own.id);
      const pulled = timedMs({ end, start });
      // Moved whole, it keeps its length and stops at a gap as at a scene,
      // rather than being squeezed against it.
      const length = own.endMs - own.startMs;
      const isMoved =
        pulled.startMs !== own.startMs &&
        Math.abs(pulled.endMs - pulled.startMs - length) < 2;
      const startMs = isMoved
        ? Math.min(
            Math.max(pulled.startMs, room.from),
            Math.max(room.from, room.to - length),
          )
        : Math.max(pulled.startMs, room.from);
      const endMs = isMoved
        ? startMs + length
        : Math.max(Math.min(pulled.endMs, room.to), startMs + LEAST_SCENE_MS);
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
