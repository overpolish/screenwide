// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { mappedAnnotationShape } from "../annotations/annotation-mapping";
import {
  arrangedGroup,
  Arrangement,
  availableGroupArrangements,
} from "../annotations/annotation-order";
import { Annotation } from "../annotations/annotations";

import { withScreenshotCountersNumbered } from "./screenshot-counters";
import {
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";

/** Each layer's source width in pixels, by layer id. */
export type ScreenshotSourceWidths = ReadonlyMap<number, number>;

// A still draws every annotation on a layer together, so any two meet.
const meet = () => true;

/**
 * An annotation belongs to the layer it is drawn on, and its place in the
 * stacking is how it changes layer. Order is relative to each layer: its
 * annotations stack above its own picture, and the layers stack in workspace
 * order. A step forward from the top of a layer's list carries the
 * annotation past the next picture up, to the bottom of that layer's list; a
 * step backward from the bottom carries it to the top of the layer below.
 * Moving to the front or back never leaves the layer.
 */
export const screenshotAnnotationArrangements = (
  workspace: ScreenshotWorkspaceOutputSettings,
  itemId: number,
  isMember: (annotation: Annotation) => boolean,
) => {
  const index = workspace.items.findIndex((item) => item.id === itemId);
  if (index === -1)
    return {
      canBringForward: false,
      canMoveToBack: false,
      canMoveToFront: false,
      canSendBackward: false,
    };
  const own = availableGroupArrangements(
    workspace.items[index].output.annotations,
    isMember,
    meet,
  );
  return {
    canBringForward: own.canBringForward || index < workspace.items.length - 1,
    canMoveToBack: own.canSendBackward,
    canMoveToFront: own.canBringForward,
    canSendBackward: own.canSendBackward || index > 0,
  };
};

/**
 * Where a layer's source pixels fall on the canvas: its source's top-left
 * corner and how many canvas pixels one source pixel spans.
 */
const sourceOnCanvas = (output: ScreenshotOutputSettings, width: number) => {
  const scale = output.imageWidth / width;
  return width > 0 && Number.isFinite(scale) && scale > 0
    ? { scale, x: output.imageX, y: output.imageY }
    : null;
};

/**
 * The workspace with the annotations `isMember` picks on layer `itemId`
 * moved one step, or to an end, through the stacking, and the layer they end
 * up on; null where nothing moves. Annotations carried to another layer keep
 * their place on the canvas. Their sizes are points, which carry over as
 * they are, as a size does between every space an annotation travels.
 */
export const arrangedScreenshotAnnotations = ({
  arrangement,
  isMember,
  itemId,
  sourceWidths,
  workspace,
}: {
  arrangement: Arrangement;
  isMember: (annotation: Annotation) => boolean;
  itemId: number;
  sourceWidths: ScreenshotSourceWidths;
  workspace: ScreenshotWorkspaceOutputSettings;
}): { itemId: number; workspace: ScreenshotWorkspaceOutputSettings } | null => {
  const index = workspace.items.findIndex((item) => item.id === itemId);
  if (index === -1) return null;
  const from = workspace.items[index];
  const list = from.output.annotations;
  if (!list.some(isMember)) return null;
  const within = arrangedGroup(list, isMember, { arrangement, meets: meet });
  if (within !== list)
    return {
      itemId,
      workspace: withLayerAnnotations(workspace, [[itemId, within]]),
    };
  if (arrangement !== "forward" && arrangement !== "backward") return null;
  const to = workspace.items[index + (arrangement === "forward" ? 1 : -1)] as
    (typeof workspace.items)[number] | undefined;
  if (!to) return null;
  const source = sourceOnCanvas(from.output, sourceWidths.get(from.id) ?? 0);
  const target = sourceOnCanvas(to.output, sourceWidths.get(to.id) ?? 0);
  if (!source || !target) return null;
  const carried = list.filter(isMember).map((annotation) => ({
    ...annotation,
    shape: mappedAnnotationShape(annotation.shape, (point) => ({
      x: (source.x + point.x * source.scale - target.x) / target.scale,
      y: (source.y + point.y * source.scale - target.y) / target.scale,
    })),
  }));
  const landed =
    arrangement === "forward"
      ? [...carried, ...to.output.annotations]
      : [...to.output.annotations, ...carried];
  return {
    itemId: to.id,
    workspace: withLayerAnnotations(workspace, [
      [from.id, list.filter((annotation) => !isMember(annotation))],
      [to.id, landed],
    ]),
  };
};

const withLayerAnnotations = (
  workspace: ScreenshotWorkspaceOutputSettings,
  lists: [number, Annotation[]][],
) => {
  const byId = new Map(lists);
  return withScreenshotCountersNumbered({
    ...workspace,
    items: workspace.items.map((item) => {
      const annotations = byId.get(item.id);
      return annotations
        ? { ...item, output: { ...item.output, annotations } }
        : item;
    }),
  });
};
