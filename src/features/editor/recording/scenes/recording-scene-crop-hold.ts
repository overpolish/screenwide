// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useSyncExternalStore } from "react";

import { RecordingSceneClip } from "./recording-scenes";

// A preset's pane cropped to a new shape is laid out afresh at it, which
// moves the pane. While the crop tool stays on the scene the preview holds
// the panes where they stood instead, each crop cutting its pane where it
// stands, and the preset makes room for the new shapes once the crop has
// closed after the tool is put away, or the playhead leaves the scene (user
// asked). Only the drawing is held: the scene itself is the preset with its
// new shapes from the first crop, so undo, saving and export never see the
// held boxes.

/** The scene as the preview draws it while held, and the scene it stands
 * for, which the hold lasts only as long as. */
type Hold = { drawn: RecordingSceneClip; scene: RecordingSceneClip };

let hold: Hold | null = null;
const listeners = new Set<() => void>();

const setHold = (next: Hold | null) => {
  if (next === hold) return;
  hold = next;
  for (const listener of listeners) listener();
};

/** The scene without whether the auto zoom made it, which an edit by hand
 * takes off on its way into the timeline (`ownEditedAutoZooms`). */
const sceneKey = ({ auto: _auto, ...scene }: RecordingSceneClip) =>
  JSON.stringify(scene);

/** Holds `scene` drawn as `drawn`: its boxes where the crop left them. */
export const holdSceneCrop = (
  scene: RecordingSceneClip,
  drawn: RecordingSceneClip,
) => {
  setHold({ drawn, scene });
};

/** Lets the held scene be laid out as it is. */
export const releaseSceneCrop = () => {
  setHold(null);
};

/** How the held scene is drawn while `scene` is still the one it stands
 * for, else null. */
export const heldSceneCrop = (scene: RecordingSceneClip | undefined) =>
  hold &&
  scene &&
  hold.scene.id === scene.id &&
  sceneKey(hold.scene) === sceneKey(scene)
    ? hold.drawn
    : null;

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

/** The scene the crop tool holds where its panes stood, while it still
 * stands for its scene in `clips`, else null. */
export function useSceneCropHold(
  clips: RecordingSceneClip[] | undefined,
): RecordingSceneClip | null {
  const current = useSyncExternalStore(subscribe, () => hold);
  return useMemo(
    () =>
      current && clips
        ? heldSceneCrop(clips.find((clip) => clip.id === current.scene.id))
        : null,
    [clips, current],
  );
}

/** `clips` with `held` in place of the scene it stands for. */
export const withHeldScene = (
  clips: RecordingSceneClip[],
  held: RecordingSceneClip | null,
) => (held ? clips.map((clip) => (clip.id === held.id ? held : clip)) : clips);
