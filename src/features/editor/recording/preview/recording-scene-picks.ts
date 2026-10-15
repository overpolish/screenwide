// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingSceneArrangement } from "../scenes/recording-scene-arrangement";

import type { RecordingPreviewSelection } from "../use-recording-preview-surface";

const layerOf = (selection: RecordingPreviewSelection) =>
  selection.layerId ?? selection.paneIndex;

/**
 * What the select tool may pick as a scene draws the panes: the pane in hand
 * and every pane a press may land on. A pane the scene hides is neither drawn
 * nor picked. A custom scene's box moves and resizes like any layer, but the
 * canvas it sits on is the scene's to size, so dragging it never grows the
 * canvas. The screen carries how the scene turns it, which its chrome is
 * drawn through and the rotation gizmo turns. `freeScreen` frees the screen
 * of a scene that otherwise fixes its panes, a full scene's, which its first
 * move makes custom. While the crop tool reframes a pane, `reframe` stands in
 * for it. A press over both panes finds the last target, the one drawn in
 * front, so a scene that puts the camera behind lists the screen after it.
 */
export function scenePanePicks({
  arrangement,
  canPreviewBakedCamera,
  freeScreen,
  isFramed,
  overlay,
  reframe,
  targets,
}: {
  arrangement: RecordingSceneArrangement | null | undefined;
  canPreviewBakedCamera: boolean;
  freeScreen: boolean;
  isFramed: boolean;
  overlay: RecordingPreviewSelection | null;
  reframe: RecordingPreviewSelection | undefined;
  targets: RecordingPreviewSelection[] | null;
}) {
  const placedFreely = (given: RecordingPreviewSelection) => {
    const selection =
      freeScreen && layerOf(given) === 0 && !given.cropMode
        ? { ...given, framed: false }
        : given;
    const placed =
      isFramed &&
      !selection.framed &&
      !selection.cropMode &&
      layerOf(selection) <= 1
        ? { ...selection, inScene: true }
        : selection;
    return arrangement && layerOf(placed) === 0 && !placed.cropMode
      ? { ...placed, rotation: arrangement.rotation }
      : placed;
  };
  const opacity = arrangement?.opacity;
  const isHidden = (selection: RecordingPreviewSelection) =>
    opacity !== undefined &&
    ((layerOf(selection) === 0 && opacity.screen === 0) ||
      (layerOf(selection) === 1 &&
        canPreviewBakedCamera &&
        opacity.camera === 0));
  const pane = reframe ?? overlay;
  const shown = targets
    ?.filter((target) => !isHidden(target))
    .map((target) =>
      reframe && target.layerId === reframe.layerId
        ? reframe
        : placedFreely(target),
    );
  const isScreen = (target: RecordingPreviewSelection) => layerOf(target) === 0;
  return {
    overlay: pane && !isHidden(pane) ? placedFreely(pane) : null,
    targets:
      shown && arrangement?.cameraInFront === false
        ? [
            ...shown.filter((target) => !isScreen(target)),
            ...shown.filter(isScreen),
          ]
        : (shown ?? null),
  };
}
