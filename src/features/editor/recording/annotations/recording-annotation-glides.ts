// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotationShape } from "../../annotations/annotations";
import { TRANSITION_MS } from "../scenes/recording-scene-arrangement";

import { annotationInScene, sameGeometry } from "./recording-annotation-scenes";

import type { RecordingAnnotationClip } from "./recording-annotations";
import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";

// When an annotation travels between its places in the scenes: the twin of
// `scene_shape/glide.rs`, so the strip marks the moves the picture makes. A
// glide sets off as its scene arrives and takes as long as the distance
// asks, from the scene's own arrival up to a limit; it leaves early enough
// to be home as the scene ends.

/** The longest a glide takes, in ms. */
const LONGEST_GLIDE_MS = 1_500;
/** The share of the picture's diagonal a glide crosses in the scene's own
 * arrival time. */
const BRISK_SHARE = 0.25;

type Point = { x: number; y: number };
type Size = { height: number; width: number };

const farthest = (pairs: [Point, Point][]) =>
  Math.max(0, ...pairs.map(([a, b]) => Math.hypot(b.x - a.x, b.y - a.y)));

/** How far `from`'s geometry is from `to`'s, in source pixels; the twin of
 * `travel` in `glide.rs`. */
function travel(from: AnnotationShape, to: AnnotationShape): number {
  if (from.kind !== to.kind) return 0;
  const [a, b] = [from, to] as Record<string, unknown>[] as Record<
    string,
    Point
  >[];
  switch (from.kind) {
    case "arrow":
      return farthest([
        [a.start, b.start],
        [a.control, b.control],
        [a.end, b.end],
      ]);
    case "counter":
      return farthest([[a.center, b.center]]);
    case "text":
      return farthest([[a.origin, b.origin]]);
    case "draw": {
      const [p0, p1] = [from.points, (to as typeof from).points];
      if (p0.length === p1.length)
        return farthest(p0.map((point, index) => [point, p1[index] ?? point]));
      const ends = (points: Point[]): [Point, Point] | null =>
        points.length > 0 ? [points[0], points[points.length - 1]] : null;
      const [e0, e1] = [ends(p0), ends(p1)];
      return e0 && e1
        ? farthest([
            [e0[0], e1[0]],
            [e0[1], e1[1]],
          ])
        : 0;
    }
    case "image": {
      const other = to as typeof from;
      return (
        farthest([[from.center, other.center]]) +
        Math.abs(other.size - from.size) / 2
      );
    }
    default:
      return farthest([
        [a.start, b.start],
        [a.end, b.end],
      ]);
  }
}

/** How long a glide from `from` to `to` takes on `picture`, in ms; the twin
 * of `glide_ms` in `glide.rs`. */
export function glideMs(
  from: AnnotationShape,
  to: AnnotationShape,
  picture: Size | undefined,
) {
  const diagonal = picture ? Math.hypot(picture.width, picture.height) : 0;
  if (diagonal <= 0) return TRANSITION_MS;
  const share = travel(from, to) / diagonal;
  return Math.min(
    LONGEST_GLIDE_MS,
    Math.max(TRANSITION_MS, TRANSITION_MS * Math.sqrt(share / BRISK_SHARE)),
  );
}

/** Where, in timeline ms, `clip`'s annotation travels between its places in
 * `scenes` (sorted by start), on a video `videoMs` long drawn on `picture`:
 * arriving as each scene it has a place in arrives, or as one it is handed
 * on from ends, where it was showing already; and leaving before such a
 * scene ends, unless it hands on or is gone by then. One that appears or
 * goes inside a scene does so in its place there. Kept to the clip's own
 * time. A pinned annotation keeps its own. */
export function annotationGlides({
  clip,
  picture,
  scenes,
  videoMs,
}: {
  clip: RecordingAnnotationClip;
  picture: Size | undefined;
  scenes: readonly RecordingSceneClip[];
  videoMs: number | undefined;
}): { endMs: number; startMs: number }[] {
  if (clip.pin || !clip.sceneShapes?.length) return [];
  const own = clip.annotation.shape;
  const place = (scene: RecordingSceneClip | undefined) =>
    scene && clip.sceneShapes?.some(({ sceneId }) => sceneId === scene.id)
      ? annotationInScene(clip, scene.id).shape
      : null;
  const glides = scenes.flatMap((scene, index) => {
    const previous = index > 0 ? scenes[index - 1] : undefined;
    const before = previous?.endMs === scene.startMs ? place(previous) : null;
    const here = place(scene);
    if (!here && !before) return [];
    const length = scene.endMs - scene.startMs;
    if (length <= 0) return [];
    const startsVideo = videoMs !== undefined && scene.startMs <= 1;
    const endsVideo = videoMs !== undefined && scene.endMs + 1 >= videoMs;
    const room = startsVideo || endsVideo ? length : length / 2;
    const to = here ?? own;
    const moves: { endMs: number; startMs: number }[] = [];
    const from = before ?? own;
    if (
      !startsVideo &&
      clip.startMs < scene.startMs &&
      !sameGeometry(from, to)
    ) {
      const window = Math.min(glideMs(from, to, picture), room);
      moves.push({ endMs: scene.startMs + window, startMs: scene.startMs });
    }
    const next = index + 1 < scenes.length ? scenes[index + 1] : undefined;
    const handsOn = next?.startMs === scene.endMs;
    if (here && !handsOn && !endsVideo && clip.endMs > scene.endMs) {
      const window = Math.min(glideMs(here, own, picture), room);
      moves.push({ endMs: scene.endMs, startMs: scene.endMs - window });
    }
    return moves;
  });
  return glides.flatMap(({ endMs, startMs }) => {
    const start = Math.max(startMs, clip.startMs);
    const end = Math.min(endMs, clip.endMs);
    return end > start ? [{ endMs: end, startMs: start }] : [];
  });
}
