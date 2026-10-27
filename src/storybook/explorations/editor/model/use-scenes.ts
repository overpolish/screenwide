// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { type SceneClip, type ScenePreset } from "../editor-data";
import { type Beat, type Recording } from "../recording";

// The scenes over the recording, auto zoom's to start with. Scenes never
// overlap, as the app's own never do: a scene is retimed only up to its
// neighbours, and one added fits the room between them.

/** The shortest a scene can be, in seconds. */
const MIN_SCENE_S = 0.5;
/** Round a beat, how far a scene added for it reaches either side. */
const AROUND_S = 0.4;
/** How long a scene added where nothing is done lasts, in seconds. */
const NEW_SCENE_S = 3;

/** The room a scene at a moment can have: from the end of the one before
 * to the start of the one after, within the recording's `duration`. */
export function sceneRoom(
  { duration, scenes }: { duration: number; scenes: SceneClip[] },
  at: number,
  except?: string,
) {
  const others = scenes.filter(({ id }) => id !== except);
  return {
    from: Math.max(
      0,
      ...others.filter(({ end }) => end <= at).map(({ end }) => end),
    ),
    to: Math.min(
      duration,
      ...others.filter(({ start }) => start >= at).map(({ start }) => start),
    ),
  };
}

export function useScenes(recording: Recording) {
  const [scenes, setScenes] = useState<SceneClip[]>(recording.scenes);
  const roomFor = (at: number, except?: string) =>
    sceneRoom({ duration: recording.duration, scenes }, at, except);
  const update = (id: string, change: Partial<SceneClip>) => {
    setScenes((current) =>
      current.map((clip) => (clip.id === id ? { ...clip, ...change } : clip)),
    );
  };
  return {
    /** A new scene at a moment: over the beat there, or a few seconds from
     * it, within the room between its neighbours. Inside a scene already,
     * that scene is given the preset instead. Gives the scene's id. */
    addScene: (preset: ScenePreset, at: number, beats: Beat[]) => {
      const inside = scenes.find(({ end, start }) => at >= start && at < end);
      if (inside) {
        update(inside.id, { preset });
        return inside.id;
      }
      const beat = beats.find(({ end, start }) => at >= start && at < end);
      const room = roomFor(at);
      const start = Math.max(room.from, beat ? beat.start - AROUND_S : at);
      const end = Math.min(
        room.to,
        beat ? beat.end + AROUND_S : at + NEW_SCENE_S,
      );
      const id = `scene-${String(Date.now())}`;
      setScenes((current) =>
        [...current, { end, id, preset, start, zoom: 2 }].sort(
          (first, second) => first.start - second.start,
        ),
      );
      return id;
    },
    /** A zoom drawn along the rail, from one moment to another, within the
     * room it starts in. Gives its id, or null where there is no room. */
    drawScene: (from: number, to: number) => {
      const room = roomFor(from);
      const start = Math.max(room.from, Math.min(from, to));
      const end = Math.min(room.to, Math.max(from, to));
      if (end - start < MIN_SCENE_S) return null;
      const id = `scene-${String(Date.now())}`;
      setScenes((current) =>
        [...current, { end, id, preset: "zoom" as const, start, zoom: 2 }].sort(
          (first, second) => first.start - second.start,
        ),
      );
      return id;
    },
    removeScene: (id: string) => {
      setScenes((current) => current.filter((clip) => clip.id !== id));
    },
    /** Moves a scene's ends, no shorter than a little and no further than
     * its neighbours. */
    retimeScene: (id: string, start: number, end: number) => {
      const room = roomFor((start + end) / 2, id);
      const from = Math.max(room.from, Math.min(start, end - MIN_SCENE_S));
      update(id, {
        end: Math.min(room.to, Math.max(end, from + MIN_SCENE_S)),
        start: from,
      });
    },
    scenes,
    /** Puts the scenes back as they were, for undo. */
    setScenes,
    updateScene: update,
  };
}
