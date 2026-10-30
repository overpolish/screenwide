// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  presetDrawing,
  type ScenePreset,
} from "../../glyphs/scene-layout-drawing";
import { presetName } from "../../island/scene/scene-names";
import { type Span } from "../timeline-scale";

import { type TimelineScene } from "./scene-clip";

// Scenes behind the scene stories: a 16:10 screen and a round 16:9 camera
// on a 16:9 video.

const SHAPE = {
  aspect: 16 / 9,
  cameraAspect: 16 / 9,
  cameraRadius: 50,
  screen: { height: 0.88, width: 0.792, x: 0.104, y: 0.06 },
  screenAspect: 16 / 10,
};

export const storyScene = (
  id: string,
  preset: ScenePreset,
  span: Span,
): TimelineScene => ({
  ...span,
  drawing: presetDrawing(preset, {}, SHAPE),
  id,
  name: presetName(preset),
});

/** The scenes on a rail, and the recording's length they keep inside. */
type Rail = { duration: number; scenes: TimelineScene[] };

/** The room between the neighbours of a moment, leaving out `except`. */
function roomAt({ duration, scenes }: Rail, moment: number, except?: string) {
  const others = scenes.filter((other) => other.id !== except);
  return {
    from: Math.max(
      0,
      ...others.filter(({ end }) => end <= moment).map(({ end }) => end),
    ),
    to: Math.min(
      duration,
      ...others
        .filter(({ start }) => start >= moment)
        .map(({ start }) => start),
    ),
  };
}

/** The scene a drag from `from` to `to` draws, fitted between its
 * neighbours as the editor fits it, or null for a drag too short. */
export function sceneDrawn(rail: Rail, from: number, to: number) {
  const start = Math.min(from, to);
  const end = Math.max(from, to);
  if (end - start < 1) return null;
  const room = roomAt(rail, from);
  return storyScene(
    `scene-${String(rail.scenes.length + 1)}`,
    "picture-in-picture",
    {
      end: Math.min(end, room.to),
      start: Math.max(start, room.from),
    },
  );
}

/** The rail's scenes with `id` retimed to `span`, kept a second long at
 * least and between its neighbours, as the editor keeps it. */
export function keptScene(rail: Rail, id: string, { end, start }: Span) {
  const own = rail.scenes.find((scene) => scene.id === id);
  if (!own) return rail.scenes;
  const room = roomAt(rail, (own.start + own.end) / 2, id);
  return rail.scenes.map((scene) =>
    scene.id === id
      ? {
          ...scene,
          end: Math.min(Math.max(end, start + 1), room.to),
          start: Math.max(Math.min(start, end - 1), room.from),
        }
      : scene,
  );
}
