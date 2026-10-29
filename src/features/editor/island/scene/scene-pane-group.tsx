// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { NumberField } from "../../../../primitives/input-fields/number-field";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { IslandFigure } from "../island-figure";
import { IslandRotation } from "../island-rotation";

import type { SceneFramingProps, ScenePane } from "./scene-framing";

// A pane's group: where it sits in its box, which is a scene's own. Exact
// figures for what is mostly done by hand on the picture (how far it zooms
// in, which part of it sits in the middle of its box) and the screen's turn
// in depth, named, since a bare figure does not say what it measures. A
// figure off its default shows a reset at its start, which goes again once
// it is back; while the camera follows the face, the middle of its box is
// the face, so Position's reset centres the face. A pane's corners and the
// camera following the face are its layer's, set on the Screen and Camera
// islands, which edit a scene's own while the playhead is in it.

const MAX_ZOOM_PERCENT = 800;

const percent = (share: number) => Math.round(share * 100);

export function ScenePaneGroup(
  props: SceneFramingProps & { pane: "camera" | "screen" },
) {
  const { camera, onCameraChange, onPaneReset, onScreenChange, pane, screen } =
    props;
  const framing: ScenePane | null = pane === "camera" ? camera : screen;
  if (!framing) return null;
  const change = pane === "camera" ? onCameraChange : onScreenChange;
  return (
    // As wide as its rows need. Rows sit as close as the island's own parts
    // do.
    <IslandFlow className="flex flex-col gap-inline">
      <ControlRow title={t("editor-island-scene-zoom")}>
        {(controlProps) => (
          <IslandFigure
            controlProps={controlProps}
            isDefault={framing.zoom === 1}
            onReset={() => {
              onPaneReset(pane, "zoom");
            }}
            resetLabel={t("editor-island-scene-reset-zoom")}
          >
            <NumberField
              aria-label={t("editor-island-scene-zoom")}
              maxValue={MAX_ZOOM_PERCENT}
              minValue={100}
              onChange={(value) => {
                change({ zoom: value / 100 });
              }}
              rightSection="%"
              step={10}
              value={percent(framing.zoom)}
            />
          </IslandFigure>
        )}
      </ControlRow>
      <ControlRow title={t("editor-island-scene-position")}>
        {(controlProps) => (
          <IslandFigure
            controlProps={controlProps}
            isDefault={framing.focusX === 0.5 && framing.focusY === 0.5}
            onReset={() => {
              onPaneReset(pane, "position");
            }}
            resetLabel={t("editor-island-scene-reset-position")}
          >
            <NumberField
              aria-label={t("editor-island-scene-position-x")}
              leftSection={t("editor-island-scene-x")}
              maxValue={100}
              minValue={0}
              onChange={(value) => {
                change({ focusX: value / 100 });
              }}
              rightSection="%"
              step={1}
              value={percent(framing.focusX)}
            />
            <NumberField
              aria-label={t("editor-island-scene-position-y")}
              leftSection={t("editor-island-scene-y")}
              maxValue={100}
              minValue={0}
              onChange={(value) => {
                change({ focusY: value / 100 });
              }}
              rightSection="%"
              step={1}
              value={percent(framing.focusY)}
            />
          </IslandFigure>
        )}
      </ControlRow>
      {pane === "screen" && screen ? (
        <IslandRotation
          onChange={props.onRotationChange}
          rotation={screen.rotation}
        />
      ) : null}
    </IslandFlow>
  );
}
