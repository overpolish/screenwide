// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { movedPaddedSceneBox } from "./recording-scene-custom";
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
 * travel is in shares of the `canvas` of output pixels, as the boxes are. The
 * screen's travel is measured on its box padded by the scene's inset, where
 * its handles are drawn. */
export const boxEdit =
  (
    boxes: SceneBoxes,
    paneName: PaneName,
    canvas: { height: number; width: number },
  ) =>
  (clip: RecordingSceneClip, event: RecordingSelectionGestureEvent) => {
    const box = boxes[paneName];
    if (!box) return null;
    // The scene's inset pads the screen by a share of its box's shorter side.
    const inset =
      paneName === "screen"
        ? (clip.inset ?? 0) *
          Math.min(box.width * canvas.width, box.height * canvas.height)
        : 0;
    return {
      ...clip,
      boxes: {
        ...boxes,
        [paneName]: movedPaddedSceneBox(box, {
          delta: { x: event.deltaX, y: event.deltaY },
          pad: {
            x: inset / Math.max(1, canvas.width),
            y: inset / Math.max(1, canvas.height),
          },
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

/** A crop's `framing` for a camera that follows the face, with the focus of
 * the framing it had kept: following, the focus says where the face sits in
 * the box, which a drag or the Position fields set, and a crop only picks how
 * much of the picture the box shows and its shape. */
export const keepingFaceSpot = (
  framing: SceneFraming,
  kept: SceneFraming,
): SceneFraming => ({
  ...framing,
  focusX: kept.focusX,
  focusY: kept.focusY,
});
