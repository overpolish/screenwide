// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import type { SceneFramingProps, ScenePane } from "./scene-framing";
import type { SceneBoxes } from "../../../../bindings/SceneBoxes";

// The framing behind the scene island's stories, and handlers that write
// each change back into the story's args, as the editor would.

export const CUSTOM: SceneBoxes = {
  camera: { height: 0.4, width: 0.25, x: 0.7, y: 0.5 },
  screen: { height: 0.8, width: 0.6, x: 0.05, y: 0.1 },
};

const SAVED_LAYOUTS = [
  {
    boxes: {
      camera: { height: 0.5, width: 0.3, x: 0.05, y: 0.25 },
      screen: { height: 0.9, width: 0.6, x: 0.38, y: 0.05 },
    },
    id: "saved-1",
  },
];

export const SAVED_LAYOUT = SAVED_LAYOUTS[0];

/** Eleven saved layouts, each drawn differently: the camera in each corner
 * and the middle, at two sizes, and a few with the screen alone. */
export const MANY_SAVED_LAYOUTS = Array.from({ length: 11 }, (_, index) => {
  const size = index % 2 === 0 ? 0.25 : 0.4;
  const spots = [
    { x: 0.05, y: 0.05 },
    { x: 0.95 - size, y: 0.05 },
    { x: 0.05, y: 0.95 - size },
    { x: 0.95 - size, y: 0.95 - size },
    { x: 0.5 - size / 2, y: 0.5 - size / 2 },
  ];
  const spot = spots[index % spots.length];
  return {
    boxes:
      index % 4 === 3
        ? { screen: { height: 0.8, width: 0.8, x: 0.1, y: 0.1 } }
        : {
            camera: { height: size, width: size, ...spot },
            screen: { height: 0.9, width: 0.9, x: 0.05, y: 0.05 },
          },
    id: `saved-${String(index + 1)}`,
  };
});

// A 16:10 screen and a 16:9 camera, on a 16:9 video, the screen inset as
// the recording's frame pads it.
const WIDE_VIDEO = {
  aspect: 16 / 9,
  cameraAspect: 16 / 9,
  screen: { height: 0.88, width: 0.792, x: 0.104, y: 0.06 },
  screenAspect: 16 / 10,
};

/** The same sources on a 1080 by 1920 video. */
export const TALL_VIDEO = {
  ...WIDE_VIDEO,
  aspect: 9 / 16,
  screen: { height: 0.316, width: 0.9, x: 0.05, y: 0.342 },
};

export const FRAMING_ARGS = {
  camera: {
    focusX: 0.5,
    focusY: 0.5,
    radius: 50,
    zoom: 1,
  },
  canPlaceCamera: true,
  layout: { kind: "preset", preset: "split-two-thirds" },
  onCameraChange: fn(),
  onLayoutChange: fn(),
  onPaneReset: fn(),
  onRemoveSavedLayout: fn(),
  onRotationChange: fn(),
  onSaveLayout: fn(),
  onScreenChange: fn(),
  onSwapPanes: fn(),
  onVariantChange: fn(),
  savedLayouts: SAVED_LAYOUTS,
  screen: {
    focusX: 0.5,
    focusY: 0.5,
    radius: 12,
    rotation: { x: 0, y: 0, z: 0 },
    zoom: 1,
  },
  variant: {},
  video: WIDE_VIDEO,
} satisfies SceneFramingProps;

const changed = <Pane extends ScenePane>(
  pane: Pane,
  patch: Partial<ScenePane>,
): Pane => ({ ...pane, ...patch });

const reset = <Pane extends ScenePane>(
  pane: Pane,
  figure: "position" | "zoom",
): Pane =>
  figure === "zoom"
    ? { ...pane, zoom: 1 }
    : { ...pane, focusX: 0.5, focusY: 0.5 };

/** The framing's handlers for a story's render: each logs through the arg
 * and writes its change back with `updateArgs`. */
export function framingHandlers(
  args: SceneFramingProps,
  updateArgs: (patch: Partial<SceneFramingProps>) => void,
): Partial<SceneFramingProps> {
  const { camera, screen } = args;
  return {
    onCameraChange: (pane) => {
      args.onCameraChange(pane);
      if (camera) updateArgs({ camera: changed(camera, pane) });
    },
    onLayoutChange: (layout) => {
      args.onLayoutChange(layout);
      updateArgs({
        layout:
          layout.kind === "preset"
            ? layout
            : {
                boxes:
                  args.savedLayouts.find(({ id }) => id === layout.id)?.boxes ??
                  CUSTOM,
                id: layout.id,
                kind: "saved",
              },
      });
    },
    onPaneReset: (pane, figure) => {
      args.onPaneReset(pane, figure);
      if (pane === "camera" && camera)
        updateArgs({ camera: reset(camera, figure) });
      if (pane === "screen" && screen)
        updateArgs({ screen: reset(screen, figure) });
    },
    onRotationChange: (rotation) => {
      args.onRotationChange(rotation);
      if (screen) updateArgs({ screen: { ...screen, rotation } });
    },
    onScreenChange: (pane) => {
      args.onScreenChange(pane);
      if (screen) updateArgs({ screen: changed(screen, pane) });
    },
    onSwapPanes: () => {
      args.onSwapPanes();
      const { layout } = args;
      if (layout.kind !== "custom" || !layout.boxes.camera) return;
      updateArgs({
        layout: {
          ...layout,
          boxes: { camera: layout.boxes.screen, screen: layout.boxes.camera },
        },
      });
    },
    onVariantChange: (variant) => {
      args.onVariantChange(variant);
      updateArgs({ variant: { ...args.variant, ...variant } });
    },
  };
}
