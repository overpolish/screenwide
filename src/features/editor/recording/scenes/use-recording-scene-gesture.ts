// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import {
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
} from "../../screenshot/screenshot-output";

import { movedSceneBox } from "./recording-scene-custom";
import {
  croppedCustomPane,
  customCropWindow,
} from "./recording-scene-custom-crop";
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

/** The scene at the playhead, while a gesture on its panes can reframe it:
 * every clip, the one under the playhead, and its panes. */
export type RecordingSceneTarget = {
  /** Null where the scene places no camera. */
  camera: ScenePane | null;
  clipId: string;
  clips: RecordingSceneClip[];
  screen: ScenePane;
};

type PaneName = "camera" | "screen";

type ActiveGesture =
  | { kind: "screen" }
  | {
      clipId: string;
      clips: RecordingSceneClip[];
      /** Each event's edit of the clip, from the clip as the gesture found
       * it; null leaves it as it was. */
      edit: (
        clip: RecordingSceneClip,
        event: RecordingSelectionGestureEvent,
      ) => RecordingSceneClip | null;
      kind: "edit";
    };

/** A custom scene's box for `paneName` moved or resized by `event`, whose
 * travel is in shares of the canvas, as the boxes are. */
const boxEdit =
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
const framingEdit = (
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

/**
 * Takes the gestures on the screen and the camera while the playhead is
 * inside a scene. A custom scene's own boxes move and resize with the select
 * tool, and the crop tool crops them in place, freely, the picture staying
 * where it sits. Any other pane is fixed where the scene puts it: with the
 * select tool the screen's moves and resizes are swallowed and the camera's
 * pan its picture inside its box or zoom it, and with the crop tool a window
 * of the box's shape laid over the unzoomed picture picks the part the box
 * shows. A corner radius dragged on either pane is the scene's own. Each
 * edits the clip under the playhead. The canvas's frame is left to the
 * ordinary selection gesture, as is a camera a custom scene has no box for
 * and anything else `applyGesture` answers it did not take.
 */
export function useRecordingSceneGesture({
  editGesture,
  onClipsChange,
  primaryOutput,
  target,
}: {
  editGesture: { beginGesture: () => void; endGesture: () => void };
  onClipsChange: (clips: RecordingSceneClip[]) => void;
  primaryOutput: ScreenshotOutputSettings;
  /** Null outside every scene. */
  target: RecordingSceneTarget | null;
}) {
  const activeRef = useRef<ActiveGesture | null>(null);
  const begin = (
    event: RecordingSelectionGestureEvent,
  ): ActiveGesture | null => {
    if (!target || (event.paneIndex !== 0 && event.paneIndex !== 1))
      return null;
    const paneName: PaneName = event.paneIndex === 1 ? "camera" : "screen";
    const pane = target[paneName];
    if (!pane) return null;
    const { clipId, clips } = target;
    // A pane's corner radius is the scene's own, reported as the radius the
    // drag reaches.
    if (event.operation === "radius")
      return {
        clipId,
        clips,
        edit: (clip, next) => ({
          ...clip,
          radius: {
            ...clip.radius,
            [paneName]: Math.min(50, Math.max(0, next.scale)),
          },
        }),
        kind: "edit",
      };
    const boxes = clips.find((clip) => clip.id === clipId)?.boxes;
    // The native gesture reports its travel as shares of the canvas.
    const canvas = screenshotOutputDimensions(primaryOutput);
    const { operation } = event;
    const isCrop = operation === "cropMove" || operation === "cropResize";
    // A custom scene's own boxes are cropped in place, drawn afresh included.
    if (boxes?.[paneName] && (isCrop || operation === "cropDraw")) {
      const anchor = {
        x: pane.box.x + event.deltaX * canvas.width,
        y: pane.box.y + event.deltaY * canvas.height,
      };
      return {
        clipId,
        clips,
        edit: (clip, next) => {
          if (
            next.operation !== "cropDraw" &&
            next.operation !== "cropMove" &&
            next.operation !== "cropResize"
          )
            return null;
          const cropped = croppedCustomPane(pane, {
            canvas,
            next: customCropWindow(pane.box, {
              anchor,
              delta: {
                x: next.deltaX * canvas.width,
                y: next.deltaY * canvas.height,
              },
              edges: next.edges,
              operation: next.operation,
            }),
            slides: next.operation === "cropMove",
          });
          return {
            ...clip,
            boxes: { ...boxes, [paneName]: cropped.box },
            [paneName]: cropped.framing,
          };
        },
        kind: "edit",
      };
    }
    // With the camera a pane of its own, no scene places it, so its gestures
    // stay its own.
    if (!isCrop && operation !== "move" && operation !== "resize") return null;
    if (!isCrop && boxes) {
      if (!boxes[paneName]) return null;
      return { clipId, clips, edit: boxEdit(boxes, paneName), kind: "edit" };
    }
    if (!isCrop && paneName === "screen") return { kind: "screen" };
    return {
      clipId,
      clips,
      edit: (clip, next) => ({
        ...clip,
        [paneName]: framingEdit(pane, {
          delta: {
            x: next.deltaX * canvas.width,
            y: next.deltaY * canvas.height,
          },
          event: next,
        }),
      }),
      kind: "edit",
    };
  };
  const applyGesture = (event: RecordingSelectionGestureEvent) => {
    if (event.phase === "begin") {
      activeRef.current = begin(event);
      if (activeRef.current?.kind === "edit") editGesture.beginGesture();
      return activeRef.current !== null;
    }
    const active = activeRef.current;
    if (!active) return false;
    if (event.phase === "end" || event.phase === "cancel")
      activeRef.current = null;
    if (active.kind === "screen") return true;
    onClipsChange(
      active.clips.map((clip) =>
        clip.id === active.clipId && event.phase !== "cancel"
          ? (active.edit(clip, event) ?? clip)
          : clip,
      ),
    );
    // Keep the history gesture open through React's commit, as the ordinary
    // gesture does, so a late update cannot become a second undo entry.
    if (event.phase === "end" || event.phase === "cancel")
      requestAnimationFrame(editGesture.endGesture);
    return true;
  };
  return {
    applyGesture,
    gestureAccepted: () => activeRef.current !== null,
  };
}
