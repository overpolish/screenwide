// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RECORDING_SCENE_MINIMUM_MS } from "./recording-scenes";

import type { SceneTemplate } from "./recording-scene-template";
import type { RecordingScenePreset } from "./recording-scenes";

/** How a scene is laid out when it is made or changed from outside the Scene
 * panel: as Custom, a preset, or one of your templates. */
export type SceneChoice =
  | { preset: RecordingScenePreset; type: "preset" }
  | { template: SceneTemplate; type: "template" }
  | { type: "custom" };

type Timed = { endMs: number; startMs: number };

/** What a choice made for a stretch of the edited video acts on: a new scene
 * over `[startMs, endMs)`, or the scene `id` already there, moved to cover
 * `[startMs, endMs)` instead. */
export type SceneSpanTarget = Timed &
  ({ id: string; type: "change" } | { type: "add" });

/** `span` held to `[from, to)` in whole milliseconds, grown to the shortest
 * scene around its middle where it is shorter; null where the gap cannot
 * hold one. */
const fitted = (span: Timed, from: number, to: number): Timed | null => {
  if (to - from < RECORDING_SCENE_MINIMUM_MS) return null;
  const startMs = Math.max(from, Math.round(span.startMs));
  const endMs = Math.min(to, Math.round(span.endMs));
  if (endMs - startMs >= RECORDING_SCENE_MINIMUM_MS) return { endMs, startMs };
  const middle = (span.startMs + span.endMs) / 2;
  const grown = Math.round(
    Math.min(
      Math.max(from, middle - RECORDING_SCENE_MINIMUM_MS / 2),
      to - RECORDING_SCENE_MINIMUM_MS,
    ),
  );
  return { endMs: grown + RECORDING_SCENE_MINIMUM_MS, startMs: grown };
};

/** How long `clip` and `span` play together, 0 where they do not meet. */
const overlapMs = (clip: Timed, span: Timed) =>
  Math.max(
    0,
    Math.min(clip.endMs, span.endMs) - Math.max(clip.startMs, span.startMs),
  );

/**
 * What choosing a scene for `span`, a stretch of the edited video `lengthMs`
 * long, acts on, or null where it can act on nothing. The scene the span
 * overlaps most is changed where they share at least half of the shorter of
 * the two: words inside a scene, or words around all of it. It is moved to
 * cover the span, so fewer or more words than it covered trim or stretch it.
 * Otherwise a scene is added over the span. Either way the span is held
 * between the scenes on each side, so words timed a few milliseconds into a
 * neighbour still give a scene that starts where it ends, and a span too
 * short for a scene grows to the shortest one, centred where the gap allows.
 * Times are whole milliseconds, as the clips' are.
 */
export function sceneSpanTarget({
  clips,
  lengthMs,
  span,
}: {
  clips: readonly (Timed & { id: string })[];
  lengthMs: number;
  span: Timed;
}): SceneSpanTarget | null {
  const middle = (span.startMs + span.endMs) / 2;
  // Only a scene that shares enough with the span is a candidate. The one
  // under the span's middle always is, so an added scene never lands on it.
  const under = clips
    .filter(
      (clip) =>
        (clip.startMs <= middle && middle < clip.endMs) ||
        (overlapMs(clip, span) > 0 &&
          overlapMs(clip, span) * 2 >=
            Math.min(clip.endMs - clip.startMs, span.endMs - span.startMs)),
    )
    .reduce<(typeof clips)[number] | undefined>(
      (best, clip) =>
        !best || overlapMs(clip, span) > overlapMs(best, span) ? clip : best,
      undefined,
    );
  const others = under ? clips.filter((clip) => clip !== under) : clips;
  const from = Math.max(
    0,
    ...others.flatMap((clip) =>
      clip.endMs <= (under?.startMs ?? middle) ? [clip.endMs] : [],
    ),
  );
  const to = Math.min(
    Math.floor(lengthMs),
    ...others.flatMap((clip) =>
      clip.startMs >= (under?.endMs ?? middle) ? [clip.startMs] : [],
    ),
  );
  const timed = fitted(span, from, to);
  if (!under) return timed ? { ...timed, type: "add" } : null;
  // A scene with no room to be the shortest one keeps where it is.
  const { endMs, startMs } = timed ?? under;
  return { endMs, id: under.id, startMs, type: "change" };
}
