// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  boxesDrawing,
  presetDrawing,
  type SceneDrawing,
  type ScenePreset,
  type SceneShape,
} from "../../glyphs/scene-layout-drawing";

import type { SceneBoxes } from "../../../../bindings/SceneBoxes";
import type { SceneFraming } from "../../../../bindings/SceneFraming";
import type { SceneRotation } from "../../../../bindings/SceneRotation";
import type { SceneVariant } from "../../../../bindings/SceneVariant";

// How a stretch of the recording is framed, which a scene sets over its
// stretch and the recording sets everywhere else: the same layout and the
// same panes, edited by the same groups on the scene island, with a scene
// in hand or none.

/** How the panes are laid out: a preset, a layout of its own once a pane
 * is moved by hand, or a layout saved to use again. */
type SceneLayout =
  | { boxes: SceneBoxes; id: string; kind: "saved" }
  | { boxes: SceneBoxes; kind: "custom" }
  | { kind: "preset"; preset: ScenePreset };

/** A pane's framing, and how round its corners are, in percent of its
 * shorter side; `hasOwnRadius` where it sets one rather than taking the
 * recording's. */
export type ScenePane = SceneFraming & {
  hasOwnRadius: boolean;
  radius: number;
};

/** The group of options open beneath the row, one at a time. */
export type SceneGroup = "camera" | "layout" | "screen";

export type SceneFramingProps = {
  /** The camera's framing, where the layout places the camera; it follows
   * the face when `followFace`, and `isFindingFace` while it looks. */
  camera: (ScenePane & { followFace: boolean; isFindingFace: boolean }) | null;
  /** Whether a layout with the camera can be chosen: the recording has a
   * camera, and it is in the video. */
  canPlaceCamera: boolean;
  layout: SceneLayout;
  onCameraChange: (pane: Partial<ScenePane>) => void;
  onFollowFaceChange: (followFace: boolean) => void;
  /** Chooses a preset or a saved layout; a layout becomes custom only by
   * moving a pane by hand. */
  onLayoutChange: (
    layout:
      { id: string; kind: "saved" } | { kind: "preset"; preset: ScenePreset },
  ) => void;
  /** Puts one of a pane's figures back: its zoom to the whole picture, its
   * position to the middle, its radius to the recording's. */
  onPaneReset: (
    pane: "camera" | "screen",
    figure: "position" | "radius" | "zoom",
  ) => void;
  onRecenter: () => void;
  onRemoveSavedLayout: (id: string) => void;
  onRotationChange: (rotation: SceneRotation) => void;
  /** Saves a custom layout to use again (the app's scene templates). */
  onSaveLayout: () => void;
  onScreenChange: (pane: Partial<ScenePane>) => void;
  /** Trades a custom layout's screen and camera boxes. */
  onSwapPanes: () => void;
  onVariantChange: (variant: SceneVariant) => void;
  /** The layouts saved to use again, unnamed, drawn from their boxes as
   * they fit this video. */
  savedLayouts: { boxes: SceneBoxes; id: string }[];
  /** The screen's framing, where the layout shows the screen. */
  screen: (ScenePane & { rotation: SceneRotation }) | null;
  variant: SceneVariant;
  /** The video's shape and its sources', each width over height, and where
   * the recording itself puts the screen, as shares of the video, which
   * the drawings of the layouts are drawn on. */
  video: Omit<SceneShape, "cameraRadius">;
};

/** The shape the layouts are drawn on, the camera as round as it is. */
export function sceneShape({ camera, video }: SceneFramingProps): SceneShape {
  return { ...video, cameraRadius: camera?.radius ?? 0 };
}

export function layoutDrawing(props: SceneFramingProps): SceneDrawing {
  const { layout, variant } = props;
  return layout.kind === "preset"
    ? presetDrawing(layout.preset, variant, sceneShape(props))
    : boxesDrawing(layout.boxes, sceneShape(props));
}

/** The groups the framing has: the camera's only where the layout places
 * the camera, the screen's only where it shows the screen. */
export function framingGroups(props: SceneFramingProps): SceneGroup[] {
  return [
    "layout",
    ...(props.camera ? (["camera"] as const) : []),
    ...(props.screen ? (["screen"] as const) : []),
  ];
}

/** Which choices a layout has under its tiles, so a change of layout that
 * changes them swaps one set for the other; null where it has none. A
 * pair's are told apart by its preset, since a stacked pair names its
 * places differently. */
export function layoutOptionsKind({ layout }: SceneFramingProps) {
  if (layout.kind !== "preset") return layout.kind;
  return layout.preset === "picture-in-picture" ||
    layout.preset === "split-two-thirds" ||
    layout.preset === "stacked"
    ? layout.preset
    : null;
}
