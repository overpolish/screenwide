// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingSceneClip,
  RecordingScenePreset,
  sceneNeedsCamera,
  sceneShowsScreen,
} from "./recording-scenes";

type Pane = "camera" | "screen";
type Shown = { camera: boolean; screen: boolean };

/** Whether `clip` shows `pane`: a custom scene unless its boxes hide it, a
 * preset as it lays the panes out, the full one showing the camera where
 * the recording does. */
export const scenePaneIsShown = (clip: RecordingSceneClip, pane: Pane) => {
  if (pane === "screen") return sceneShowsScreen(clip);
  return clip.boxes
    ? clip.boxes.cameraHidden !== true
    : clip.preset === "full" || sceneNeedsCamera(clip);
};

/** Which panes a preset shows. */
const presetShows = (preset: RecordingScenePreset): Shown =>
  scenesShown({ endMs: 0, id: "", preset, startMs: 0 });

const scenesShown = (clip: RecordingSceneClip): Shown => ({
  camera: scenePaneIsShown(clip, "camera"),
  screen: scenePaneIsShown(clip, "screen"),
});

/** The layout a preset scene takes to show the panes `shown`: the one it had
 * before a pane was turned off, where that shows them, else the layout of
 * those panes alone, both where the recording puts them. */
const presetShowing = (
  shown: Shown,
  before: RecordingScenePreset | undefined,
): RecordingScenePreset => {
  const kept = before ? presetShows(before) : null;
  if (before && kept?.camera === shown.camera && kept.screen === shown.screen)
    return before;
  if (shown.camera && shown.screen) return "full";
  if (shown.camera) return "camera-only";
  return shown.screen ? "screen-only" : "blank";
};

/**
 * `clip` showing or hiding `pane`. A preset's scene takes the layout of the
 * panes left: turning the camera off a picture in picture gives Screen only,
 * the screen back in the recording's own place, and turning it on again
 * goes back to the picture in picture, which `layoutBefore` keeps. A custom
 * scene keeps each pane where it was put, hidden in its place, so it comes
 * back there. One that already shows or hides the pane as asked is left as
 * it is.
 */
export const sceneWithPaneShown = (
  clip: RecordingSceneClip,
  { pane, shown }: { pane: Pane; shown: boolean },
): RecordingSceneClip => {
  if (scenePaneIsShown(clip, pane) === shown) return clip;
  if (!clip.boxes) {
    const { layoutBefore, ...rest } = clip;
    const preset = presetShowing(
      { ...scenesShown(clip), [pane]: shown },
      layoutBefore,
    );
    // Turning a pane off remembers the layout it left, or the one before
    // that, so a second pane off still leads back to the fullest; turning
    // one on forgets it once it is reached.
    const before = shown
      ? preset === layoutBefore
        ? undefined
        : layoutBefore
      : (layoutBefore ?? clip.preset);
    return { ...rest, preset, ...(before ? { layoutBefore: before } : {}) };
  }
  const { cameraHidden, screenHidden, ...boxes } = clip.boxes;
  const hidden = {
    camera: pane === "camera" ? !shown : cameraHidden === true,
    screen: pane === "screen" ? !shown : screenHidden === true,
  };
  return {
    ...clip,
    boxes: {
      ...boxes,
      ...(hidden.camera ? { cameraHidden: true } : {}),
      ...(hidden.screen ? { screenHidden: true } : {}),
    },
  };
};
