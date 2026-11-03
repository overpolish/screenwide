// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  compositionBoxes,
  WHOLE_FRAME,
} from "../../recording/scenes/recording-scene-drawing";
import {
  chosenSceneTemplate,
  fittedTemplateBoxes,
} from "../../recording/scenes/recording-scene-template";
import { sceneNeedsCamera } from "../../recording/scenes/recording-scenes";
import { sceneOffer } from "../../tools/scene/scene-offer";
import { sceneIsland } from "../scene/scene-island";

import type { ToolPanelPatch } from "../../tools/tool-panel-patch";
import type { ToolPanelScenePane } from "../../tools/tool-panel-scene";
import type { ToolPanelSnapshot } from "../../tools/tool-panel-store";
import type { EditorIslandContent } from "../editor-island";
import type { SceneToolGroup } from "../scene/scene-actions";
import type { ScenePane } from "../scene/scene-framing";

// The scene under the playhead on the Scene island, or with none the
// recording's own composition, which a layout chosen there turns into a new
// scene. A layout is a saved one where the scene's boxes are one's, its own
// where they are not, else its preset; a full scene, or none, is the
// recording's own boxes. Each pane's zoom and position reach that pane
// whichever track is in hand, and every layout is drawn with the screen
// turned as the scene turns it.

const paneOf = (pane: ToolPanelScenePane | null): ScenePane | null =>
  pane ? { ...pane.framing, radius: pane.radius } : null;

export function sceneIslandContent({
  change,
  group,
  onGroupChange,
  snapshot,
}: {
  change: (values: ToolPanelPatch) => void;
  /** The group open beneath the row, kept while the island is up. */
  group: SceneToolGroup;
  onGroupChange: (group: SceneToolGroup) => void;
  snapshot: ToolPanelSnapshot;
}): EditorIslandContent | null {
  const { hasCursorData, scene, sceneTemplates } = snapshot;
  if (!scene) return null;
  const offer = sceneOffer(scene, snapshot);
  const savedId = scene.hasSceneAtPlayhead
    ? chosenSceneTemplate(offer.templates, scene)
    : null;
  const preset =
    scene.boxes === null && scene.preset !== null && scene.preset !== "full"
      ? scene.preset
      : null;
  const boxes = compositionBoxes(scene.boxes ?? scene.composition);
  // A scene that places the camera stands idle while it is not in the video.
  const isPaused =
    (scene.boxes
      ? scene.boxes.camera !== null
      : scene.preset !== null && sceneNeedsCamera({ preset: scene.preset })) &&
    !offer.isCombined;
  const reframe = (
    pane: "camera" | "screen",
    { focusX, focusY, zoom }: Partial<ScenePane>,
  ) => {
    change({
      sceneFraming: {
        ...(focusX === undefined ? {} : { focusX }),
        ...(focusY === undefined ? {} : { focusY }),
        ...(zoom === undefined ? {} : { zoom }),
      },
      sceneFramingPane: pane,
    });
  };
  return sceneIsland({
    autoZoom: hasCursorData ? { hasAutoZooms: scene.hasAutoZooms } : null,
    camera: paneOf(scene.panes.camera),
    canPlaceCamera: offer.canPlaceCamera,
    group,
    layout:
      savedId !== null
        ? { boxes, id: savedId, kind: "saved" }
        : preset !== null
          ? { kind: "preset", preset }
          : { boxes, kind: "custom" },
    onCameraChange: (pane) => {
      reframe("camera", pane);
    },
    onClearAutoZooms: () => {
      change({ clearAutoZooms: true });
    },
    onDelete: () => {
      change({ deleteScene: true });
    },
    onGroupChange,
    onLayoutChange: (layout) => {
      if (layout.kind === "preset") {
        change({ scenePreset: layout.preset });
        return;
      }
      const template = sceneTemplates.find(({ id }) => id === layout.id);
      if (template) change({ chooseSceneTemplate: template });
    },
    onPaneReset: (pane, figure) => {
      reframe(
        pane,
        figure === "zoom" ? { zoom: 1 } : { focusX: 0.5, focusY: 0.5 },
      );
    },
    onRegenerateAutoZooms: () => {
      change({ autoZoomScenes: true });
    },
    onRemoveSavedLayout: (id) => {
      change({ removeSceneTemplate: id });
    },
    onSaveLayout: () => {
      change({ saveSceneTemplate: true });
    },
    onScreenChange: (pane) => {
      reframe("screen", pane);
    },
    onSwapPanes: () => {
      change({ swapScenePanes: true });
    },
    onVariantChange: (variant) => {
      change({ sceneVariant: variant });
    },
    savedLayouts: sceneTemplates.map((template) => ({
      boxes: fittedTemplateBoxes(template, scene.canvasAspect),
      id: template.id,
    })),
    scene:
      scene.hasSceneAtPlayhead && scene.sceneId !== null
        ? { id: scene.sceneId, isPaused }
        : null,
    screen: paneOf(scene.panes.screen),
    variant: scene.variant ?? {},
    video: {
      aspect: scene.canvasAspect,
      cameraAspect: scene.cameraAspect,
      rotation: scene.rotation ?? undefined,
      screen: scene.composition.screen ?? WHOLE_FRAME,
      screenAspect: scene.screenAspect,
    },
  });
}
