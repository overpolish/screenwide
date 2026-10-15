// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { movedSceneBox } from "./recording-scene-custom";
import {
  framingFromWindow,
  reframed,
  reframeWindow,
} from "./recording-scene-framing";
import { SceneRect } from "./recording-scene-geometry";
import {
  RecordingSceneClip,
  SceneBoxes,
  SceneFraming,
} from "./recording-scenes";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";

/** One pane of the scene at the playhead: its picture's aspect, the box it is
 * drawn in, in the canvas's output pixels, and the framing it shows there. */
export type ScenePane = {
  aspect: number;
  box: SceneRect;
  framing: SceneFraming;
};

export type PaneName = "camera" | "screen";

/** A custom scene's box for `paneName` moved or resized by `event`, whose
 * travel is in shares of the canvas, as the boxes are. */
export const boxEdit =
  (boxes: SceneBoxes, paneName: PaneName) =>
  (clip: RecordingSceneClip, event: RecordingSelectionGestureEvent) => {
    const box = boxes[paneName];
    if (!box) return null;
    return {
      ...clip,
      boxes: {
        ...boxes,
        [paneName]: movedSceneBox(box, {
          delta: { x: event.deltaX, y: event.deltaY },
          scale: event.operation === "resize" ? event.scale : 1,
        }),
      },
    };
  };

/** `pane`'s framing reframed by `event`, whose travel is in output pixels:
 * the crop tool's window over its unzoomed picture moved or resized, or with
 * the select tool its picture panned inside its box or zoomed. */
export const framingEdit = (
  pane: ScenePane,
  {
    delta,
    event,
  }: {
    delta: { x: number; y: number };
    event: RecordingSelectionGestureEvent;
  },
) => {
  const { window } = reframeWindow(pane);
  if (event.operation === "cropMove")
    return framingFromWindow({
      ...pane,
      window: { ...window, x: window.x + delta.x, y: window.y + delta.y },
    });
  // A framed crop window reports its corner's travel and how far it grew, its
  // shape kept.
  if (event.operation === "cropResize")
    return framingFromWindow({
      ...pane,
      window: {
        height: window.height * event.scale,
        width: window.width * event.scale,
        x: window.x + delta.x,
        y: window.y + delta.y,
      },
    });
  return reframed({
    ...pane,
    delta,
    frame: pane.box,
    scale: event.operation === "resize" ? event.scale : 1,
  });
};
