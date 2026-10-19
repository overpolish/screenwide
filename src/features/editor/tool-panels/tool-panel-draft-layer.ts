// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToolPanelPatch } from "./tool-panel-patch";
import { ToolPanelLayerSelection } from "./tool-panel-selection";

/**
 * The selected layer as the panel shows it while `patch` waits for the
 * editor's answer. A dragged inset or radius holds its own value until the
 * editor acknowledges it, so the knob stays under the pointer rather than
 * snapping back a frame; the switches flip and a dialled turn shows at once,
 * as a scene's does; the camera's effects change at once, as the
 * microphone's switches do, the bars arriving with the editor's answer.
 */
export const draftedLayer = (
  placed: ToolPanelLayerSelection | null | undefined,
  {
    cameraEffects,
    selectionDropShadow,
    selectionFollowFace,
    selectionInset,
    selectionRadius,
    selectionRotation,
  }: Pick<
    ToolPanelPatch,
    | "cameraEffects"
    | "selectionDropShadow"
    | "selectionFollowFace"
    | "selectionInset"
    | "selectionRadius"
    | "selectionRotation"
  >,
) => {
  if (!placed) return placed;
  const layer = {
    ...placed,
    ...(selectionInset === undefined ? {} : { inset: selectionInset }),
    ...(selectionRadius === undefined ? {} : { radius: selectionRadius }),
    ...(selectionDropShadow === undefined
      ? {}
      : { dropShadow: selectionDropShadow }),
    ...(selectionFollowFace === undefined
      ? {}
      : { followsFace: selectionFollowFace }),
  };
  const turned =
    layer.rotation && selectionRotation
      ? { ...layer, rotation: { ...layer.rotation, ...selectionRotation } }
      : layer;
  return turned.cameraEffects && cameraEffects
    ? {
        ...turned,
        cameraEffects: { ...turned.cameraEffects, ...cameraEffects },
      }
    : turned;
};
