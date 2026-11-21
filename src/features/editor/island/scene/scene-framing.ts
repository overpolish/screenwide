// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  boxesDrawing,
  presetDrawing,
  type SceneDrawing,
  type ScenePreset,
  type SceneShape,
} from "../../glyphs/scene-layout-drawing";
import { canvasShare } from "../../layer-measure";

import type { Measure } from "../../../../bindings/Measure";
import type { SceneBoxes } from "../../../../bindings/SceneBoxes";
import type { SceneFraming } from "../../../../bindings/SceneFraming";
import type { SceneVariant } from "../../../../bindings/SceneVariant";

// How a stretch of the recording is framed, which a scene sets over its
// stretch and the recording sets everywhere else: the same layout, edited
// on the scene island, with a scene in hand or none. Where each pane sits
// and how far it zooms in is done on the picture.

/** How the panes are laid out: a preset, a layout of its own once a pane
 * is moved by hand, or a layout saved to use again. */
type SceneLayout =
  | { boxes: SceneBoxes; id: string; kind: "saved" }
  | { boxes: SceneBoxes; kind: "custom" }
  | { kind: "preset"; preset: ScenePreset };

/** A pane's framing, and how round its corners are. */
export type ScenePane = SceneFraming & {
  radius: Measure;
};

export type SceneFramingProps = {
  /** The camera's framing, where the layout places the camera. */
  camera: ScenePane | null;
  /** Whether a layout with the camera can be chosen: the recording has a
   * camera, and it is in the video. */
  canPlaceCamera: boolean;
  layout: SceneLayout;
  /** Chooses a preset or a saved layout; a layout becomes custom only by
   * moving a pane by hand. */
  onLayoutChange: (
    layout:
      { id: string; kind: "saved" } | { kind: "preset"; preset: ScenePreset },
  ) => void;
  /** Shows or hides a pane of the scene in hand: a preset's takes the
   * layout of the panes left, a custom one's keeps its place. */
  onPaneShownChange: (pane: "camera" | "screen", shown: boolean) => void;
  onRemoveSavedLayout: (id: string) => void;
  /** Saves a custom layout to use again (the app's scene templates). */
  onSaveLayout: () => void;
  /** Trades a custom layout's screen and camera boxes. */
  onSwapPanes: () => void;
  onVariantChange: (variant: SceneVariant) => void;
  /** Whether the scene in hand shows each pane; null with none in hand,
   * where the recording's own framing shows what the recording does. */
  panesShown: { camera: boolean; screen: boolean } | null;
  /** The layouts saved to use again, unnamed, drawn from their boxes as
   * they fit this video. */
  savedLayouts: { boxes: SceneBoxes; id: string }[];
  variant: SceneVariant;
  /** The video's shape and its sources', each width over height, where the
   * recording itself puts the screen, as shares of the video, and how the
   * scene turns the screen, which the drawings of the layouts are drawn
   * on. */
  video: Omit<SceneShape, "cameraRadius">;
};

/** The shape the layouts are drawn on, the camera as round as it is. */
export function sceneShape({ camera, video }: SceneFramingProps): SceneShape {
  return { ...video, cameraRadius: camera?.radius ?? canvasShare(0) };
}

export function layoutDrawing(props: SceneFramingProps): SceneDrawing {
  const { layout, variant } = props;
  return layout.kind === "preset"
    ? presetDrawing(layout.preset, variant, sceneShape(props))
    : boxesDrawing(layout.boxes, sceneShape(props));
}

/** Whether the scene shows `pane`, where it can be turned on and off: with
 * a scene in hand, and for the camera only where one is in the video to
 * show. Null where it cannot. */
export function paneShownChoice(
  { canPlaceCamera, panesShown }: SceneFramingProps,
  pane: "camera" | "screen",
): boolean | null {
  if (!panesShown || (pane === "camera" && !canPlaceCamera)) return null;
  return panesShown[pane];
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
