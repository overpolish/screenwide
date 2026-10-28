// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { AngleDial } from "../../../primitives/angle-dial/angle-dial";

import { IslandFigure } from "./island-figure";

import type { SceneRotation } from "../../../bindings/SceneRotation";

const AXES = [
  { axis: "x", color: "red", label: () => t("editor-island-rotation-x") },
  { axis: "y", color: "green", label: () => t("editor-island-rotation-y") },
  { axis: "z", color: "blue", label: () => t("editor-island-rotation-z") },
] as const;

/** How something on the picture is turned, in depth and flat, a row of its
 * own: a dial for each axis in the colour that names it, after the reset
 * that stands it upright again. The screen in a scene turns this way, and
 * so does a placed image. */
export function IslandRotation({
  onChange,
  rotation,
}: {
  onChange: (rotation: SceneRotation) => void;
  rotation: SceneRotation;
}) {
  return (
    <ControlRow title={t("editor-island-rotation")}>
      {(controlProps) => (
        <IslandFigure
          controlProps={controlProps}
          isDefault={rotation.x === 0 && rotation.y === 0 && rotation.z === 0}
          onReset={() => {
            onChange({ x: 0, y: 0, z: 0 });
          }}
          resetLabel={t("editor-island-reset-rotation")}
        >
          {AXES.map(({ axis, color, label }) => (
            <AngleDial
              aria-label={label()}
              color={color}
              key={axis}
              onChange={(degrees) => {
                onChange({ ...rotation, [axis]: degrees });
              }}
              value={rotation[axis]}
            />
          ))}
        </IslandFigure>
      )}
    </ControlRow>
  );
}
